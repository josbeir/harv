use axum::{Router, extract::Query, http::StatusCode, response::Html, routing::get};
use harv_core::HarvError;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::Notify;

/// Default Harvest OAuth2 client ID. Override at compile time by setting
/// `HARV_CLIENT_ID` in the environment:
/// ```bash
/// HARV_CLIENT_ID="your-app-id" cargo build --release
/// ```
///
/// Custom OAuth2 applications can be created at:
/// https://id.getharvest.com/developers
///
/// When registering your app, set the redirect URI to `http://localhost:5006`.
pub const OAUTH_CLIENT_ID: &str = match option_env!("HARV_CLIENT_ID") {
    Some(id) => id,
    None => "I4jYaGkAYUyfrlcmJBFilpCF",
};
const OAUTH_BASE_URL: &str = "https://id.getharvest.com";
const CALLBACK_PORT: u16 = 5006;

const SUCCESS_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>harv-cli</title>
</head>
<body style="background: #eee; font-family: Arial, Helvetica, sans-serif">
    <main style="display: flex; flex-grow: 1; justify-content: center; margin: 80px auto;">
        <div style="background: #fff; background-clip: padding-box; border: 1px solid rgba(0,0,0,0.2);
                    border-radius: 6px; box-shadow: 0 2px 10px rgb(0 0 0 / 10%); padding: 1em; text-align: center;">
            <h1 style="font-weight: 600; margin: 0.25em 0">harv-cli</h1>
            <p>You are now authenticated. You may close this window and return to the CLI.</p>
        </div>
    </main>
</body>
</html>"#;

type CallbackResult = Arc<Mutex<Option<Result<(String, String), HarvError>>>>;

/// Runs the OAuth2 implicit grant authentication flow.
pub async fn authenticate() -> Result<(String, String), HarvError> {
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
                let parsed = parse_callback(&params);
                *result.lock().unwrap() = Some(parsed);
                notify.notify_one();
                (StatusCode::OK, Html(String::from(SUCCESS_HTML)))
            }
        }),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], CALLBACK_PORT));
    let listener = TcpListener::bind(addr).await.map_err(|e| {
        HarvError::Other(format!("Failed to bind to port {}: {}", CALLBACK_PORT, e))
    })?;

    let auth_url = format!(
        "{}/oauth2/authorize?client_id={}&response_type=token",
        OAUTH_BASE_URL, OAUTH_CLIENT_ID
    );

    let open_result = open::that(&auth_url);
    if let Err(e) = &open_result {
        tracing::warn!(
            "Failed to open browser: {}. Open this URL manually:\n{}",
            e,
            auth_url
        );
    }

    let server = axum::serve(listener, app);
    let handle = tokio::spawn(async move { server.await });

    // Wait until we get the callback or the timeout fires
    tokio::select! {
        _ = notify.notified() => {
            handle.abort();
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(120)) => {
            handle.abort();
            return Err(HarvError::Other("OAuth login timed out after 120 seconds".into()));
        }
    }

    result
        .lock()
        .unwrap()
        .take()
        .ok_or(HarvError::OAuthFailed)?
}

fn parse_callback(query: &HashMap<String, String>) -> Result<(String, String), HarvError> {
    if let Some(error) = query.get("error") {
        if error == "access_denied" {
            return Err(HarvError::OAuthDenied);
        }
        return Err(HarvError::Other(format!("OAuth error: {}", error)));
    }

    let access_token = query
        .get("access_token")
        .ok_or(HarvError::OAuthFailed)?
        .clone();

    let scope = query.get("scope").ok_or(HarvError::OAuthFailed)?;

    let account_id = scope
        .split(':')
        .nth(1)
        .filter(|s| s.chars().all(|c| c.is_ascii_digit()))
        .ok_or_else(|| {
            HarvError::Other(format!(
                "Invalid scope format. Expected 'harvest:ACCOUNT_ID', got '{}'",
                scope
            ))
        })?
        .to_string();

    Ok((access_token, account_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_callback_success() {
        let mut params = HashMap::new();
        params.insert("access_token".into(), "abc123".into());
        params.insert("scope".into(), "harvest:1234567".into());
        params.insert("token_type".into(), "bearer".into());
        params.insert("expires_in".into(), "1209599".into());

        let result = parse_callback(&params).unwrap();
        assert_eq!(result.0, "abc123");
        assert_eq!(result.1, "1234567");
    }

    #[test]
    fn test_parse_callback_access_denied() {
        let mut params = HashMap::new();
        params.insert("error".into(), "access_denied".into());

        let err = parse_callback(&params).unwrap_err();
        assert!(matches!(err, HarvError::OAuthDenied));
    }

    #[test]
    fn test_parse_callback_missing_token() {
        let mut params = HashMap::new();
        params.insert("scope".into(), "harvest:1234567".into());

        let err = parse_callback(&params).unwrap_err();
        assert!(matches!(err, HarvError::OAuthFailed));
    }

    #[test]
    fn test_parse_callback_missing_scope() {
        let mut params = HashMap::new();
        params.insert("access_token".into(), "abc123".into());

        let err = parse_callback(&params).unwrap_err();
        assert!(matches!(err, HarvError::OAuthFailed));
    }

    #[test]
    fn test_parse_callback_invalid_scope() {
        let mut params = HashMap::new();
        params.insert("access_token".into(), "abc123".into());
        params.insert("scope".into(), "invalid_format".into());

        let err = parse_callback(&params).unwrap_err();
        assert!(err.to_string().contains("Invalid scope"));
    }
}
