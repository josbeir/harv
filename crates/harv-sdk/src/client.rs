use crate::auth;
use crate::config::{AuthMethod, HarvConfig};
use crate::resources::clients::ClientsApi;
use crate::resources::company::CompanyApi;
use crate::resources::projects::ProjectsApi;
use crate::resources::tasks::TasksApi;
use crate::resources::time_entries::TimeEntriesApi;
use crate::resources::users::UsersApi;
use chrono::{DateTime, Duration, Utc};
use harv_core::HarvError;
use reqwest::header::{
    ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, RETRY_AFTER, USER_AGENT,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

const BASE_URL: &str = "https://api.harvestapp.com/v2";
const USER_AGENT_STRING: &str = "harv-cli (https://github.com/josbeir/harv)";
const REFRESH_LEEWAY: Duration = Duration::minutes(5);

#[derive(Debug, Clone)]
struct AuthState {
    method: AuthMethod,
    access_token: String,
    expires_at: Option<DateTime<Utc>>,
    refresh_token: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
}

impl AuthState {
    fn from_config(config: &HarvConfig) -> Self {
        Self {
            method: config.auth_method(),
            access_token: config.access_token().to_owned(),
            expires_at: config.access_token_expires_at(),
            refresh_token: config.refresh_token().map(str::to_owned),
            client_id: config.oauth_client_id().map(str::to_owned),
            client_secret: config.oauth_client_secret().map(str::to_owned),
        }
    }

    fn needs_refresh(&self) -> bool {
        self.method == AuthMethod::RefreshableOAuth
            && self
                .expires_at
                .is_none_or(|expires_at| expires_at <= Utc::now() + REFRESH_LEEWAY)
    }
}

/// The main entry point for interacting with the Harvest API v2.
#[derive(Clone)]
pub struct HarvClient {
    http: reqwest::Client,
    config: HarvConfig,
    auth: Arc<RwLock<AuthState>>,
    refresh_lock: Arc<Mutex<()>>,
    persist_refreshed_credentials: bool,
    base_url: String,
    token_url: String,
}

impl HarvClient {
    /// Create a new client from a config.
    pub fn new(config: HarvConfig) -> Result<Self, HarvError> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|error| HarvError::Http(error.to_string()))?;
        Ok(Self {
            http,
            auth: Arc::new(RwLock::new(AuthState::from_config(&config))),
            refresh_lock: Arc::new(Mutex::new(())),
            persist_refreshed_credentials: false,
            config,
            base_url: BASE_URL.to_string(),
            token_url: auth::TOKEN_URL.to_string(),
        })
    }

    /// Override the base URL (for testing with local mock servers).
    pub fn with_base_url(mut self, base_url: &str) -> Self {
        self.base_url = base_url.to_string();
        self
    }

    #[cfg(test)]
    fn with_token_url(mut self, token_url: &str) -> Self {
        self.token_url = token_url.to_string();
        self
    }

    /// Load config from `~/.config/harv/config.toml` and create a client.
    pub async fn from_config_file() -> Result<Self, HarvError> {
        let mut client = Self::new(HarvConfig::load().await?)?;
        client.persist_refreshed_credentials = true;
        Ok(client)
    }

    #[cfg(feature = "mock-mode")]
    pub async fn from_config_or_mock() -> Result<Self, HarvError> {
        if std::env::var("HARV_MOCK").as_deref() == Ok("1") {
            let mock_url = crate::mock_server::start().await;
            return Ok(Self::new(crate::mock_data::test_config())?.with_base_url(&mock_url));
        }
        Self::from_config_file().await
    }

    #[cfg(not(feature = "mock-mode"))]
    pub async fn from_config_or_mock() -> Result<Self, HarvError> {
        Self::from_config_file().await
    }

    /// Returns the configuration snapshot loaded with this client.
    pub fn config(&self) -> &HarvConfig {
        &self.config
    }

    async fn headers(&self) -> Result<HeaderMap, HarvError> {
        let token = self.access_token().await?;
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|error| HarvError::Http(format!("Invalid bearer token: {error}")))?,
        );
        headers.insert(
            "Harvest-Account-Id",
            HeaderValue::from_str(self.config.account_id())
                .map_err(|error| HarvError::Http(format!("Invalid account ID: {error}")))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STRING));
        Ok(headers)
    }

    async fn access_token(&self) -> Result<String, HarvError> {
        let state = self.auth.read().await.clone();
        if !state.needs_refresh() {
            return Ok(state.access_token);
        }
        self.refresh_access_token(None).await
    }

    async fn refresh_access_token(
        &self,
        rejected_token: Option<&str>,
    ) -> Result<String, HarvError> {
        let _refresh_guard = self.refresh_lock.lock().await;
        let state = self.auth.read().await.clone();
        if rejected_token.is_some_and(|token| token != state.access_token) {
            return Ok(state.access_token);
        }
        if rejected_token.is_none() && !state.needs_refresh() {
            return Ok(state.access_token);
        }
        let (previous_refresh_token, client_id, client_secret) = match (
            state.refresh_token.as_deref(),
            state.client_id.as_deref(),
            state.client_secret.as_deref(),
        ) {
            (Some(refresh_token), Some(client_id), Some(client_secret)) => (
                refresh_token.to_owned(),
                client_id.to_owned(),
                client_secret.to_owned(),
            ),
            _ => return Err(HarvError::NotAuthenticated),
        };
        let _file_lock = if self.persist_refreshed_credentials {
            Some(HarvConfig::acquire_refresh_lock().await.map_err(|error| {
                HarvError::Other(format!(
                    "Authentication refresh could not acquire its lock; run `harv connect`: {error}"
                ))
            })?)
        } else {
            None
        };
        if self.persist_refreshed_credentials
            && let Ok(latest) = HarvConfig::load().await
            && latest.auth_method() == AuthMethod::RefreshableOAuth
            && latest.account_id() == self.config.account_id()
            && latest.oauth_client_id() == Some(client_id.as_str())
            && latest.refresh_token() != Some(previous_refresh_token.as_str())
        {
            let refreshed = AuthState::from_config(&latest);
            let access_token = refreshed.access_token.clone();
            *self.auth.write().await = refreshed;
            return Ok(access_token);
        }
        let (access_token, refresh_token, expires_at) = auth::refresh_access_token(
            &self.http,
            &self.token_url,
            &previous_refresh_token,
            &client_id,
            &client_secret,
        )
        .await
        .map_err(|error| {
            HarvError::Other(format!(
                "Authentication refresh failed; run `harv connect`: {error}"
            ))
        })?;
        {
            let mut state = self.auth.write().await;
            state.access_token = access_token.clone();
            state.refresh_token = Some(refresh_token.clone());
            state.expires_at = Some(expires_at);
        }
        if self.persist_refreshed_credentials {
            HarvConfig::save_refreshed_credentials(
                self.config.account_id(),
                &client_id,
                &previous_refresh_token,
                access_token.clone(),
                refresh_token.clone(),
                expires_at,
            )
            .await
            .map_err(|error| {
                HarvError::Other(format!(
                    "Authentication refresh could not be saved; run `harv connect`: {error}"
                ))
            })?;
        }
        Ok(access_token)
    }

    async fn send<F>(&self, request: F) -> Result<reqwest::Response, HarvError>
    where
        F: Fn(HeaderMap) -> reqwest::RequestBuilder,
    {
        let headers = self.headers().await?;
        let rejected_token = bearer_token(&headers);
        let mut response = request(headers)
            .send()
            .await
            .map_err(|error| HarvError::Http(error.to_string()))?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED
            && self.auth.read().await.method == AuthMethod::RefreshableOAuth
        {
            self.refresh_access_token(rejected_token.as_deref()).await?;
            response = request(self.headers().await?)
                .send()
                .await
                .map_err(|error| HarvError::Http(error.to_string()))?;
        }
        Ok(response)
    }

    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, HarvError> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .send(|headers| self.http.get(&url).headers(headers).query(query))
            .await?;
        self.handle_response(response).await
    }

    pub(crate) async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, HarvError> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .send(|headers| self.http.post(&url).headers(headers).json(body))
            .await?;
        self.handle_response(response).await
    }

    pub(crate) async fn patch<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, HarvError> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .send(|headers| self.http.patch(&url).headers(headers).json(body))
            .await?;
        self.handle_response(response).await
    }

    pub(crate) async fn delete(&self, path: &str) -> Result<(), HarvError> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .send(|headers| self.http.delete(&url).headers(headers))
            .await?;
        if response.status().is_success() {
            Ok(())
        } else if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            Err(HarvError::RateLimited {
                retry_after_secs: retry_after(&response),
            })
        } else {
            let status = response.status().as_u16();
            Err(HarvError::Api {
                status,
                message: response.text().await.unwrap_or_default(),
            })
        }
    }

    async fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, HarvError> {
        let status = response.status();
        if status.is_success() {
            response
                .json()
                .await
                .map_err(|error| HarvError::Http(format!("Failed to parse response: {error}")))
        } else if status == reqwest::StatusCode::UNAUTHORIZED {
            Err(HarvError::NotAuthenticated)
        } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            Err(HarvError::RateLimited {
                retry_after_secs: retry_after(&response),
            })
        } else {
            Err(HarvError::Api {
                status: status.as_u16(),
                message: response.text().await.unwrap_or_default(),
            })
        }
    }

    pub fn clients(&self) -> ClientsApi<'_> {
        ClientsApi::new(self)
    }
    pub fn company(&self) -> CompanyApi<'_> {
        CompanyApi::new(self)
    }
    pub fn projects(&self) -> ProjectsApi<'_> {
        ProjectsApi::new(self)
    }
    pub fn tasks(&self) -> TasksApi<'_> {
        TasksApi::new(self)
    }
    pub fn time_entries(&self) -> TimeEntriesApi<'_> {
        TimeEntriesApi::new(self)
    }
    pub fn users(&self) -> UsersApi<'_> {
        UsersApi::new(self)
    }
}

fn retry_after(response: &reqwest::Response) -> Option<u64> {
    response
        .headers()
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok())
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::tempdir;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

    struct UnauthorizedThenOk(AtomicUsize);

    impl Respond for UnauthorizedThenOk {
        fn respond(&self, _request: &Request) -> ResponseTemplate {
            if self.0.fetch_add(1, Ordering::SeqCst) == 0 {
                ResponseTemplate::new(401)
            } else {
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true}))
            }
        }
    }

    struct CountingTokenResponder(Arc<AtomicUsize>);

    impl Respond for CountingTokenResponder {
        fn respond(&self, _request: &Request) -> ResponseTemplate {
            self.0.fetch_add(1, Ordering::SeqCst);
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "new-token", "refresh_token": "new-refresh", "expires_in": 3600
            }))
        }
    }

    #[tokio::test]
    async fn refreshes_expiring_token_before_request_and_persists_it() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let directory = tempdir().unwrap();
        unsafe { std::env::set_var("HARV_CONFIG_DIR", directory.path()) };
        let mut config = HarvConfig::new("old-token".into(), "1".into());
        config.set_locale(Some("nl".into()));
        config.set_authentication(crate::config::Authentication::new(
            AuthMethod::RefreshableOAuth,
            "old-token".into(),
            "1".into(),
            Some(Utc::now() - Duration::seconds(1)),
            Some("old-refresh".into()),
            Some("client-id".into()),
            Some("client-secret".into()),
        ));
        config.save().await.unwrap();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "new-token", "refresh_token": "new-refresh", "expires_in": 3600
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/data"))
            .and(header("authorization", "Bearer new-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
            .mount(&server)
            .await;
        let client = HarvClient::from_config_file()
            .await
            .unwrap()
            .with_base_url(&server.uri())
            .with_token_url(&format!("{}/token", server.uri()));
        let value: serde_json::Value = client.get("/data", &[]).await.unwrap();
        assert_eq!(value["ok"], true);
        let saved = HarvConfig::load().await.unwrap();
        assert_eq!(saved.access_token(), "new-token");
        assert_eq!(saved.refresh_token(), Some("new-refresh"));
        assert_eq!(saved.locale(), Some("nl"));
    }

    #[tokio::test]
    async fn direct_client_refresh_does_not_overwrite_global_config() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let directory = tempdir().unwrap();
        unsafe { std::env::set_var("HARV_CONFIG_DIR", directory.path()) };
        HarvConfig::new("global-token".into(), "99".into())
            .save()
            .await
            .unwrap();
        let mut config = HarvConfig::new("old-token".into(), "1".into());
        config.set_authentication(crate::config::Authentication::new(
            AuthMethod::RefreshableOAuth,
            "old-token".into(),
            "1".into(),
            Some(Utc::now() - Duration::seconds(1)),
            Some("old-refresh".into()),
            Some("client-id".into()),
            Some("client-secret".into()),
        ));
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "new-token", "refresh_token": "new-refresh", "expires_in": 3600
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/data"))
            .and(header("authorization", "Bearer new-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
            .mount(&server)
            .await;
        let client = HarvClient::new(config)
            .unwrap()
            .with_base_url(&server.uri())
            .with_token_url(&format!("{}/token", server.uri()));
        let _: serde_json::Value = client.get("/data", &[]).await.unwrap();
        let saved = HarvConfig::load().await.unwrap();
        assert_eq!(saved.access_token(), "global-token");
        assert_eq!(saved.account_id(), "99");
    }

    #[tokio::test]
    async fn refresh_does_not_overwrite_a_newer_login() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let directory = tempdir().unwrap();
        unsafe { std::env::set_var("HARV_CONFIG_DIR", directory.path()) };
        let mut original = HarvConfig::new("old-token".into(), "1".into());
        original.set_authentication(crate::config::Authentication::new(
            AuthMethod::RefreshableOAuth,
            "old-token".into(),
            "1".into(),
            Some(Utc::now() - Duration::seconds(1)),
            Some("old-refresh".into()),
            Some("client-id".into()),
            Some("client-secret".into()),
        ));
        original.save().await.unwrap();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "refreshed-token", "refresh_token": "refreshed-refresh", "expires_in": 3600
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/data"))
            .and(header("authorization", "Bearer refreshed-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
            .mount(&server)
            .await;
        let client = HarvClient::from_config_file()
            .await
            .unwrap()
            .with_base_url(&server.uri())
            .with_token_url(&format!("{}/token", server.uri()));
        let mut replacement = HarvConfig::new("new-login-token".into(), "2".into());
        replacement.set_authentication(crate::config::Authentication::new(
            AuthMethod::RefreshableOAuth,
            "new-login-token".into(),
            "2".into(),
            Some(Utc::now() + Duration::hours(1)),
            Some("new-login-refresh".into()),
            Some("other-client-id".into()),
            Some("other-client-secret".into()),
        ));
        replacement.save().await.unwrap();
        let _: serde_json::Value = client.get("/data", &[]).await.unwrap();
        let saved = HarvConfig::load().await.unwrap();
        assert_eq!(saved.account_id(), "2");
        assert_eq!(saved.access_token(), "new-login-token");
        assert_eq!(saved.refresh_token(), Some("new-login-refresh"));
    }

    #[tokio::test]
    async fn independently_loaded_clients_share_a_refresh() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let directory = tempdir().unwrap();
        unsafe { std::env::set_var("HARV_CONFIG_DIR", directory.path()) };
        let mut config = HarvConfig::new("old-token".into(), "1".into());
        config.set_authentication(crate::config::Authentication::new(
            AuthMethod::RefreshableOAuth,
            "old-token".into(),
            "1".into(),
            Some(Utc::now() - Duration::seconds(1)),
            Some("old-refresh".into()),
            Some("client-id".into()),
            Some("client-secret".into()),
        ));
        config.save().await.unwrap();
        let server = MockServer::start().await;
        let refreshes = Arc::new(AtomicUsize::new(0));
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(CountingTokenResponder(refreshes.clone()))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/data"))
            .and(header("authorization", "Bearer new-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
            .mount(&server)
            .await;
        let first = HarvClient::from_config_file()
            .await
            .unwrap()
            .with_base_url(&server.uri())
            .with_token_url(&format!("{}/token", server.uri()));
        let second = HarvClient::from_config_file()
            .await
            .unwrap()
            .with_base_url(&server.uri())
            .with_token_url(&format!("{}/token", server.uri()));
        let (first_result, second_result) = tokio::join!(
            first.get::<serde_json::Value>("/data", &[]),
            second.get::<serde_json::Value>("/data", &[])
        );
        assert!(first_result.is_ok());
        assert!(second_result.is_ok());
        assert_eq!(refreshes.load(Ordering::SeqCst), 1);
        assert_eq!(
            HarvConfig::load().await.unwrap().refresh_token(),
            Some("new-refresh")
        );
    }

    #[tokio::test]
    async fn refresh_retains_credentials_when_persistence_fails() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let directory = tempdir().unwrap();
        unsafe { std::env::set_var("HARV_CONFIG_DIR", directory.path()) };
        let mut config = HarvConfig::new("old-token".into(), "1".into());
        config.set_authentication(crate::config::Authentication::new(
            AuthMethod::RefreshableOAuth,
            "old-token".into(),
            "1".into(),
            Some(Utc::now() - Duration::seconds(1)),
            Some("old-refresh".into()),
            Some("client-id".into()),
            Some("client-secret".into()),
        ));
        config.save().await.unwrap();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "new-token", "refresh_token": "new-refresh", "expires_in": 3600
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/data"))
            .and(header("authorization", "Bearer new-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})))
            .mount(&server)
            .await;
        let client = HarvClient::from_config_file()
            .await
            .unwrap()
            .with_base_url(&server.uri())
            .with_token_url(&format!("{}/token", server.uri()));
        tokio::fs::remove_file(HarvConfig::path()).await.unwrap();
        assert!(client.get::<serde_json::Value>("/data", &[]).await.is_err());
        let value: serde_json::Value = client.get("/data", &[]).await.unwrap();
        assert_eq!(value["ok"], true);
    }

    #[tokio::test]
    async fn personal_tokens_are_not_refreshed() {
        let mut config = HarvConfig::new("personal-token".into(), "1".into());
        config.set_authentication(crate::config::Authentication::new(
            AuthMethod::PersonalAccessToken,
            "personal-token".into(),
            "1".into(),
            None,
            None,
            None,
            None,
        ));
        let client = HarvClient::new(config).unwrap();
        assert_eq!(client.access_token().await.unwrap(), "personal-token");
    }

    #[tokio::test]
    async fn refreshable_oauth_retries_one_unauthorized_request() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let directory = tempdir().unwrap();
        unsafe { std::env::set_var("HARV_CONFIG_DIR", directory.path()) };
        let mut config = HarvConfig::new("old-token".into(), "1".into());
        config.set_authentication(crate::config::Authentication::new(
            AuthMethod::RefreshableOAuth,
            "old-token".into(),
            "1".into(),
            Some(Utc::now() + Duration::hours(1)),
            Some("old-refresh".into()),
            Some("client-id".into()),
            Some("client-secret".into()),
        ));
        config.save().await.unwrap();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "new-token", "refresh_token": "new-refresh", "expires_in": 3600
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/data"))
            .respond_with(UnauthorizedThenOk(AtomicUsize::new(0)))
            .mount(&server)
            .await;
        let client = HarvClient::new(config)
            .unwrap()
            .with_base_url(&server.uri())
            .with_token_url(&format!("{}/token", server.uri()));
        let value: serde_json::Value = client.get("/data", &[]).await.unwrap();
        assert_eq!(value["ok"], true);
    }
}
