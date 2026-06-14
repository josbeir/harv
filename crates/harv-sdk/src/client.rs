use crate::config::HarvConfig;
use harv_core::HarvError;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::resources::clients::ClientsApi;
use crate::resources::company::CompanyApi;
use crate::resources::projects::ProjectsApi;
use crate::resources::tasks::TasksApi;
use crate::resources::time_entries::TimeEntriesApi;
use crate::resources::users::UsersApi;

const BASE_URL: &str = "https://api.harvestapp.com/v2";
const USER_AGENT_STRING: &str = "harv-cli (https://github.com/josbeir/harv)";

/// The main entry point for interacting with the Harvest API v2.
#[derive(Clone)]
pub struct HarvClient {
    http: reqwest::Client,
    config: HarvConfig,
    base_url: String,
}

impl HarvClient {
    /// Create a new client from a config.
    pub fn new(config: HarvConfig) -> Result<Self, HarvError> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| HarvError::Http(e.to_string()))?;

        Ok(Self {
            http,
            config,
            base_url: BASE_URL.to_string(),
        })
    }

    /// Override the base URL (for testing with local mock servers).
    pub fn with_base_url(mut self, base_url: &str) -> Self {
        self.base_url = base_url.to_string();
        self
    }

    /// Load config from `~/.config/harv/config.toml` and create a client.
    pub async fn from_config_file() -> Result<Self, HarvError> {
        let config = HarvConfig::load().await?;
        Self::new(config)
    }

    /// Like `from_config_file`, but in mock mode (`HARV_MOCK=1`) uses
    /// a local wiremock server with fake data instead of the real API.
    /// Only available when the `mock-mode` feature is enabled.
    #[cfg(feature = "mock-mode")]
    pub async fn from_config_or_mock() -> Result<Self, HarvError> {
        if std::env::var("HARV_MOCK").as_deref() == Ok("1") {
            let mock_url = crate::mock_server::start().await;
            let config = crate::mock_data::test_config();
            return Ok(Self::new(config)?.with_base_url(&mock_url));
        }
        Self::from_config_file().await
    }

    /// Fallback when `mock-mode` is not enabled: just calls `from_config_file`.
    #[cfg(not(feature = "mock-mode"))]
    pub async fn from_config_or_mock() -> Result<Self, HarvError> {
        Self::from_config_file().await
    }

    /// Returns a reference to the underlying config.
    pub fn config(&self) -> &HarvConfig {
        &self.config
    }

    /// Builds the standard request headers with auth and content type.
    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.config.access_token))
                .expect("valid bearer token"),
        );
        headers.insert(
            "Harvest-Account-Id",
            HeaderValue::from_str(&self.config.account_id).expect("valid account ID"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STRING));
        headers
    }

    /// Makes a GET request to the given path with optional query parameters.
    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, HarvError> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .get(&url)
            .headers(self.headers())
            .query(query)
            .send()
            .await
            .map_err(|e| HarvError::Http(e.to_string()))?;

        self.handle_response(response).await
    }

    /// Makes a POST request to the given path with a JSON body.
    pub(crate) async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, HarvError> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .post(&url)
            .headers(self.headers())
            .json(body)
            .send()
            .await
            .map_err(|e| HarvError::Http(e.to_string()))?;

        self.handle_response(response).await
    }

    /// Makes a PATCH request to the given path with a JSON body.
    pub(crate) async fn patch<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, HarvError> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .patch(&url)
            .headers(self.headers())
            .json(body)
            .send()
            .await
            .map_err(|e| HarvError::Http(e.to_string()))?;

        self.handle_response(response).await
    }

    /// Makes a DELETE request to the given path.
    pub(crate) async fn delete(&self, path: &str) -> Result<(), HarvError> {
        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http
            .delete(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| HarvError::Http(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            Err(HarvError::Api {
                status,
                message: body,
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
                .map_err(|e| HarvError::Http(format!("Failed to parse response: {}", e)))
        } else if status == reqwest::StatusCode::UNAUTHORIZED {
            let body = response.text().await.unwrap_or_default();
            tracing::error!("401 Unauthorized: {}", body);
            Err(HarvError::NotAuthenticated)
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(HarvError::Api {
                status: status.as_u16(),
                message: body,
            })
        }
    }

    // --- Resource accessors ---

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
