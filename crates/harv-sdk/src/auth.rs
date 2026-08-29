use axum::{Router, extract::Query, http::StatusCode, response::Html, routing::get};
use chrono::{DateTime, Duration, Utc};
use harv_core::HarvError;
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::Notify;
use uuid::Uuid;

/// Default Harvest OAuth2 client ID used by the convenient browser sign-in.
pub const OAUTH_CLIENT_ID: &str = match option_env!("HARV_CLIENT_ID") {
    Some(id) => id,
    None => "I4jYaGkAYUyfrlcmJBFilpCF",
};
const OAUTH_BASE_URL: &str = "https://id.getharvest.com";
pub(crate) const TOKEN_URL: &str = "https://id.getharvest.com/api/v2/oauth2/token";
const CALLBACK_PORT: u16 = 5006;
const CALLBACK_URL: &str = "http://localhost:5006";

const SUCCESS_HTML: &str = r#"<!DOCTYPE html><html><head><meta charset="utf-8"><title>harv</title></head><body><p>Authentication completed. You may close this window and return to Harv.</p></body></html>"#;

#[derive(Debug, Clone)]
pub struct OAuthCredentials {
    pub access_token: String,
    pub account_id: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

type CallbackResult = Arc<Mutex<Option<Result<HashMap<String, String>, HarvError>>>>;

/// Runs the OAuth2 implicit grant used by Harv's built-in application.
pub async fn authenticate() -> Result<OAuthCredentials, HarvError> {
    let params = wait_for_callback(&format!(
        "{OAUTH_BASE_URL}/oauth2/authorize?client_id={OAUTH_CLIENT_ID}&response_type=token"
    ))
    .await?;
    let (access_token, account_id) = parse_implicit_callback(&params)?;
    Ok(OAuthCredentials {
        access_token,
        account_id,
        expires_at: expires_at(&params),
        refresh_token: None,
    })
}

/// Runs authorization-code OAuth for a user-owned Harvest application.
pub async fn authenticate_refreshable(
    client_id: &str,
    client_secret: &str,
) -> Result<OAuthCredentials, HarvError> {
    let state = Uuid::new_v4().to_string();
    let auth_url = format!(
        "{OAUTH_BASE_URL}/oauth2/authorize?client_id={client_id}&response_type=code&redirect_uri={CALLBACK_URL}&state={state}"
    );
    let params = wait_for_callback(&auth_url).await?;
    let code = parse_code_callback(&params, &state)?;
    let account_id = account_id_from_scope(&params)?;
    let token = exchange_code(
        &reqwest::Client::new(),
        TOKEN_URL,
        &code,
        client_id,
        client_secret,
    )
    .await?;
    Ok(OAuthCredentials {
        access_token: token.access_token,
        account_id,
        expires_at: Some(Utc::now() + Duration::seconds(token.expires_in)),
        refresh_token: Some(token.refresh_token),
    })
}

pub(crate) async fn refresh_access_token(
    http: &reqwest::Client,
    token_url: &str,
    refresh_token: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<(String, String, DateTime<Utc>), HarvError> {
    let response = http
        .post(token_url)
        .form(&[
            ("refresh_token", refresh_token),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .map_err(|error| HarvError::Http(error.to_string()))?;
    let token = parse_token_response(response).await?;
    Ok((
        token.access_token,
        token.refresh_token,
        Utc::now() + Duration::seconds(token.expires_in),
    ))
}

async fn exchange_code(
    http: &reqwest::Client,
    token_url: &str,
    code: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<TokenResponse, HarvError> {
    let response = http
        .post(token_url)
        .form(&[
            ("code", code),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", CALLBACK_URL),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|error| HarvError::Http(error.to_string()))?;
    parse_token_response(response).await
}

async fn parse_token_response(response: reqwest::Response) -> Result<TokenResponse, HarvError> {
    if !response.status().is_success() {
        return Err(HarvError::Other(format!(
            "OAuth token request failed with status {}",
            response.status()
        )));
    }
    response
        .json()
        .await
        .map_err(|error| HarvError::Other(format!("Invalid OAuth token response: {error}")))
}

async fn wait_for_callback(auth_url: &str) -> Result<HashMap<String, String>, HarvError> {
    let result: CallbackResult = Arc::new(Mutex::new(None));
    let result_handler = result.clone();
    let notify = Arc::new(Notify::new());
    let notify_handler = notify.clone();
    let app = Router::new().route(
        "/",
        get(move |Query(params): Query<HashMap<String, String>>| {
            let result = result_handler.clone();
            let notify = notify_handler.clone();
            async move {
                *result.lock().expect("OAuth callback mutex poisoned") = Some(Ok(params));
                notify.notify_one();
                (StatusCode::OK, Html(String::from(SUCCESS_HTML)))
            }
        }),
    );
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], CALLBACK_PORT)))
        .await
        .map_err(|error| {
            HarvError::Other(format!(
                "Failed to bind OAuth callback port {CALLBACK_PORT}: {error}"
            ))
        })?;
    if let Err(error) = open::that(auth_url) {
        tracing::warn!("Failed to open browser: {error}. Open this URL manually: {auth_url}");
    }
    let handle = tokio::spawn(async move { axum::serve(listener, app).await });
    tokio::select! {
        _ = notify.notified() => handle.abort(),
        _ = tokio::time::sleep(std::time::Duration::from_secs(120)) => {
            handle.abort();
            return Err(HarvError::Other("OAuth login timed out after 120 seconds".into()));
        }
    }
    result
        .lock()
        .expect("OAuth callback mutex poisoned")
        .take()
        .ok_or(HarvError::OAuthFailed)?
}

fn parse_implicit_callback(query: &HashMap<String, String>) -> Result<(String, String), HarvError> {
    check_callback_error(query)?;
    let access_token = query
        .get("access_token")
        .ok_or(HarvError::OAuthFailed)?
        .clone();
    Ok((access_token, account_id_from_scope(query)?))
}

fn parse_code_callback(
    query: &HashMap<String, String>,
    expected_state: &str,
) -> Result<String, HarvError> {
    check_callback_error(query)?;
    if query.get("state").map(String::as_str) != Some(expected_state) {
        return Err(HarvError::Other(
            "OAuth callback state did not match the login request".into(),
        ));
    }
    query.get("code").cloned().ok_or(HarvError::OAuthFailed)
}

fn check_callback_error(query: &HashMap<String, String>) -> Result<(), HarvError> {
    match query.get("error").map(String::as_str) {
        Some("access_denied") => Err(HarvError::OAuthDenied),
        Some(error) => Err(HarvError::Other(format!("OAuth error: {error}"))),
        None => Ok(()),
    }
}

fn account_id_from_scope(query: &HashMap<String, String>) -> Result<String, HarvError> {
    let scope = query.get("scope").ok_or(HarvError::OAuthFailed)?;
    scope
        .split(':')
        .nth(1)
        .filter(|id| id.chars().all(|character| character.is_ascii_digit()))
        .map(str::to_owned)
        .ok_or_else(|| {
            HarvError::Other(format!(
                "Invalid scope format. Expected 'harvest:ACCOUNT_ID', got '{scope}'"
            ))
        })
}

fn expires_at(query: &HashMap<String, String>) -> Option<DateTime<Utc>> {
    query
        .get("expires_in")
        .and_then(|seconds| seconds.parse::<i64>().ok())
        .map(|seconds| Utc::now() + Duration::seconds(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn implicit_params() -> HashMap<String, String> {
        HashMap::from([
            ("access_token".into(), "abc123".into()),
            ("scope".into(), "harvest:1234567".into()),
            ("expires_in".into(), "1209599".into()),
        ])
    }

    #[test]
    fn parses_implicit_callback_and_expiry() {
        let params = implicit_params();
        assert_eq!(
            parse_implicit_callback(&params).unwrap(),
            ("abc123".into(), "1234567".into())
        );
        assert!(expires_at(&params).is_some());
    }

    #[test]
    fn rejects_denied_or_invalid_implicit_callback() {
        let mut denied = HashMap::new();
        denied.insert("error".into(), "access_denied".into());
        assert!(matches!(
            parse_implicit_callback(&denied),
            Err(HarvError::OAuthDenied)
        ));
        assert!(matches!(
            parse_implicit_callback(&HashMap::new()),
            Err(HarvError::OAuthFailed)
        ));
    }

    #[test]
    fn validates_code_callback_state_and_scope() {
        let params = HashMap::from([
            ("code".into(), "code-1".into()),
            ("state".into(), "expected".into()),
            ("scope".into(), "harvest:42".into()),
        ]);
        assert_eq!(parse_code_callback(&params, "expected").unwrap(), "code-1");
        assert_eq!(account_id_from_scope(&params).unwrap(), "42");
        assert!(parse_code_callback(&params, "other").is_err());
    }

    #[tokio::test]
    async fn code_exchange_uses_the_authorization_redirect_uri() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_string_contains(
                "redirect_uri=http%3A%2F%2Flocalhost%3A5006",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "access", "refresh_token": "refresh", "expires_in": 3600
            })))
            .mount(&server)
            .await;
        let token = exchange_code(
            &reqwest::Client::new(),
            &server.uri(),
            "code",
            "client-id",
            "client-secret",
        )
        .await
        .unwrap();
        assert_eq!(token.access_token, "access");
    }
}
