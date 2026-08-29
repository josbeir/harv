use chrono::{DateTime, Utc};
use harv_core::HarvError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// Configuration stored at `~/.config/harv/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvConfig {
    pub(crate) access_token: String,
    pub(crate) account_id: String,
    /// How the access token was acquired. Older configurations used the
    /// implicit OAuth flow and therefore deserialize as `QuickOAuth`.
    #[serde(default)]
    pub(crate) auth_method: AuthMethod,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) access_token_expires_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) refresh_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) oauth_client_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) oauth_client_secret: Option<String>,
    #[serde(default = "default_cache_ttl")]
    pub(crate) cache_ttl_hours: u64,
    #[serde(default)]
    pub(crate) last_project_id: Option<u64>,
    #[serde(default)]
    pub(crate) last_task_id: Option<u64>,
    #[serde(default)]
    pub(crate) locale: Option<String>,
    #[serde(default = "default_check_updates")]
    pub(crate) check_updates: bool,
    #[serde(default)]
    pub(crate) aliases: HashMap<String, Alias>,
}

/// The credential strategy used to authenticate requests to Harvest.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    /// Harv's built-in OAuth application using the implicit grant.
    #[default]
    #[serde(rename = "quick-oauth")]
    QuickOAuth,
    /// A user-created Harvest personal access token.
    #[serde(rename = "personal-access-token")]
    PersonalAccessToken,
    /// A user-owned OAuth application using refresh tokens.
    #[serde(rename = "refreshable-oauth")]
    RefreshableOAuth,
}

/// Credentials and metadata to persist after an authentication flow.
#[derive(Debug, Clone)]
pub struct Authentication {
    method: AuthMethod,
    access_token: String,
    account_id: String,
    expires_at: Option<DateTime<Utc>>,
    refresh_token: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
}

impl Authentication {
    pub fn new(
        method: AuthMethod,
        access_token: String,
        account_id: String,
        expires_at: Option<DateTime<Utc>>,
        refresh_token: Option<String>,
        client_id: Option<String>,
        client_secret: Option<String>,
    ) -> Self {
        Self {
            method,
            access_token,
            account_id,
            expires_at,
            refresh_token,
            client_id,
            client_secret,
        }
    }
}

fn default_check_updates() -> bool {
    true
}

fn default_cache_ttl() -> u64 {
    24
}

/// A named shortcut mapping an alias to a project + task pair.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Alias {
    pub project_id: u64,
    pub task_id: u64,
}

impl HarvConfig {
    /// Create a new config with the given credentials and default values.
    pub fn new(access_token: String, account_id: String) -> Self {
        Self {
            access_token,
            account_id,
            auth_method: AuthMethod::QuickOAuth,
            access_token_expires_at: None,
            refresh_token: None,
            oauth_client_id: None,
            oauth_client_secret: None,
            cache_ttl_hours: 24,
            last_project_id: None,
            last_task_id: None,
            locale: None,
            check_updates: true,
            aliases: HashMap::new(),
        }
    }

    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    pub fn auth_method(&self) -> AuthMethod {
        self.auth_method
    }

    pub fn access_token_expires_at(&self) -> Option<DateTime<Utc>> {
        self.access_token_expires_at
    }

    pub fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
    }

    pub fn oauth_client_id(&self) -> Option<&str> {
        self.oauth_client_id.as_deref()
    }

    pub fn oauth_client_secret(&self) -> Option<&str> {
        self.oauth_client_secret.as_deref()
    }

    /// Replace only credentials, retaining the user's non-auth preferences.
    pub fn set_authentication(&mut self, authentication: Authentication) {
        self.auth_method = authentication.method;
        self.access_token = authentication.access_token;
        self.account_id = authentication.account_id;
        self.access_token_expires_at = authentication.expires_at;
        self.refresh_token = authentication.refresh_token;
        self.oauth_client_id = authentication.client_id;
        self.oauth_client_secret = authentication.client_secret;
    }

    pub fn cache_ttl_hours(&self) -> u64 {
        self.cache_ttl_hours
    }

    pub fn last_project_id(&self) -> Option<u64> {
        self.last_project_id
    }

    pub fn last_task_id(&self) -> Option<u64> {
        self.last_task_id
    }

    pub fn locale(&self) -> Option<&str> {
        self.locale.as_deref()
    }

    pub fn aliases(&self) -> &HashMap<String, Alias> {
        &self.aliases
    }

    pub fn check_updates(&self) -> bool {
        self.check_updates
    }

    pub fn set_cache_ttl_hours(&mut self, hours: u64) {
        self.cache_ttl_hours = hours;
    }

    pub fn set_locale(&mut self, locale: Option<String>) {
        self.locale = locale;
    }

    pub fn set_check_updates(&mut self, enabled: bool) {
        self.check_updates = enabled;
    }

    /// Load config from `~/.config/harv/config.toml`.
    pub async fn load() -> Result<Self, HarvError> {
        let path = Self::path();
        let contents = fs::read_to_string(&path)
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => HarvError::ConfigNotFound(path),
                _ => HarvError::Io(e),
            })?;
        toml::from_str(&contents).map_err(|e| HarvError::ConfigMalformed(e.to_string()))
    }

    /// Save config to `~/.config/harv/config.toml`. Creates the directory if needed.
    pub async fn save(&self) -> Result<(), HarvError> {
        let path = Self::path();
        let toml =
            toml::to_string_pretty(self).map_err(|e| HarvError::ConfigMalformed(e.to_string()))?;
        crate::storage::atomic_write_private(&path, toml.into_bytes()).await
    }

    /// Returns the path to the config file: `~/.config/harv/config.toml`.
    ///
    /// `HARV_CONFIG_DIR` overrides the platform config directory when set.
    pub fn path() -> PathBuf {
        std::env::var_os("HARV_CONFIG_DIR")
            .map(PathBuf::from)
            .or_else(dirs::config_dir)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("harv")
            .join("config.toml")
    }

    /// Look up an alias by name. Returns `None` if not found.
    pub fn alias(&self, name: &str) -> Option<&Alias> {
        self.aliases.get(name)
    }

    /// Insert or update an alias and persist to disk.
    pub async fn set_alias(&mut self, name: &str, alias: Alias) -> Result<(), HarvError> {
        self.insert_alias(name, alias);
        self.save().await
    }

    /// Insert or update an alias without persisting to disk.
    pub fn insert_alias(&mut self, name: &str, alias: Alias) {
        self.aliases.insert(name.to_string(), alias);
    }

    /// Remove an alias and persist to disk.
    pub async fn remove_alias(&mut self, name: &str) -> Result<(), HarvError> {
        self.aliases.remove(name);
        self.save().await
    }

    /// Record the last used project and task IDs.
    pub fn set_last_used(&mut self, project_id: u64, task_id: u64) {
        self.last_project_id = Some(project_id);
        self.last_task_id = Some(task_id);
    }

    /// Record the last used project and task IDs and persist to disk.
    pub async fn save_last_used(&mut self, project_id: u64, task_id: u64) -> Result<(), HarvError> {
        let mut latest = Self::load().await.unwrap_or_else(|_| self.clone());
        latest.set_last_used(project_id, task_id);
        latest.save().await?;
        *self = latest;
        Ok(())
    }

    /// Persist newly refreshed credentials without replacing unrelated settings
    /// that may have changed since this client was created.
    pub async fn save_refreshed_credentials(
        access_token: String,
        refresh_token: String,
        expires_at: DateTime<Utc>,
    ) -> Result<(), HarvError> {
        let mut latest = Self::load().await?;
        latest.access_token = access_token;
        latest.refresh_token = Some(refresh_token);
        latest.access_token_expires_at = Some(expires_at);
        latest.save().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn set_test_config_dir(dir: &std::path::Path) {
        unsafe { std::env::set_var("HARV_CONFIG_DIR", dir.join(".config")) };
    }

    fn test_config() -> HarvConfig {
        HarvConfig {
            access_token: "test-token".into(),
            account_id: "1234567".into(),
            auth_method: AuthMethod::QuickOAuth,
            access_token_expires_at: None,
            refresh_token: None,
            oauth_client_id: None,
            oauth_client_secret: None,
            cache_ttl_hours: 24,
            last_project_id: None,
            last_task_id: None,
            locale: None,
            check_updates: true,
            aliases: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_load_nonexistent() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let dir = tempdir().unwrap();
        set_test_config_dir(dir.path());
        let _ = dirs::config_dir();

        let path = HarvConfig::path();
        assert!(path.ends_with("config.toml"));
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.toml");
        let config = test_config();

        let toml = toml::to_string_pretty(&config).unwrap();
        tokio::fs::write(&file_path, &toml).await.unwrap();

        let contents = tokio::fs::read_to_string(&file_path).await.unwrap();
        let loaded: HarvConfig = toml::from_str(&contents).unwrap();
        assert_eq!(loaded.access_token, "test-token");
        assert_eq!(loaded.account_id, "1234567");
    }

    #[tokio::test]
    async fn test_alias_operations() {
        let mut config = test_config();
        let alias = Alias {
            project_id: 1,
            task_id: 2,
        };
        config.aliases.insert("dev".into(), alias.clone());

        assert!(config.alias("dev").is_some());
        assert_eq!(config.alias("dev").unwrap().project_id, 1);

        config.aliases.remove("dev");
        assert!(config.alias("dev").is_none());
    }

    #[tokio::test]
    async fn test_alias_not_found() {
        let config = test_config();
        assert!(config.alias("nonexistent").is_none());
    }

    #[test]
    fn test_serialize_with_aliases() {
        let mut config = test_config();
        config.aliases.insert(
            "dev".into(),
            Alias {
                project_id: 10,
                task_id: 20,
            },
        );
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("dev"));
        assert!(toml.contains("10"));
    }

    #[test]
    fn test_deserialize_with_aliases() {
        let toml = r#"
access_token = "tok"
account_id = "1"

[aliases.dev]
project_id = 10
task_id = 20
"#;
        let config: HarvConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.access_token, "tok");
        assert!(config.alias("dev").is_some());
    }

    #[test]
    fn test_deserialize_without_aliases() {
        let toml = r#"
access_token = "tok"
account_id = "1"
"#;
        let config: HarvConfig = toml::from_str(toml).unwrap();
        assert!(config.aliases.is_empty());
    }

    #[tokio::test]
    async fn test_path_ends_with_config_toml() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let path = HarvConfig::path();
        assert!(path.to_string_lossy().contains("harv"));
        assert!(path.ends_with("config.toml"));
    }

    #[tokio::test]
    async fn test_path_uses_config_dir_override() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let dir = tempdir().unwrap();
        set_test_config_dir(dir.path());
        assert_eq!(
            HarvConfig::path(),
            dir.path().join(".config").join("harv").join("config.toml")
        );
    }

    #[tokio::test]
    async fn test_save_load_with_tempdir() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let tmp = tempdir().unwrap();
        set_test_config_dir(tmp.path());
        let harv_dir = tmp.path().join(".config").join("harv");
        std::fs::create_dir_all(&harv_dir).unwrap();

        let config = test_config();
        config.save().await.unwrap();

        let loaded = HarvConfig::load().await.unwrap();
        assert_eq!(loaded.access_token, "test-token");
        assert_eq!(loaded.account_id, "1234567");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn save_restricts_config_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let tmp = tempdir().unwrap();
        set_test_config_dir(tmp.path());
        test_config().save().await.unwrap();
        let config_path = HarvConfig::path();
        assert_eq!(
            std::fs::metadata(&config_path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(config_path.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }

    #[tokio::test]
    async fn test_save_set_and_remove_alias() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let tmp = tempdir().unwrap();
        set_test_config_dir(tmp.path());
        let harv_dir = tmp.path().join(".config").join("harv");
        std::fs::create_dir_all(&harv_dir).unwrap();

        let mut config = test_config();
        config.save().await.unwrap();

        config
            .set_alias(
                "dev",
                Alias {
                    project_id: 1,
                    task_id: 2,
                },
            )
            .await
            .unwrap();
        let loaded = HarvConfig::load().await.unwrap();
        assert!(loaded.alias("dev").is_some());

        let mut loaded = loaded;
        loaded.remove_alias("dev").await.unwrap();
        let after = HarvConfig::load().await.unwrap();
        assert!(after.alias("dev").is_none());
    }

    #[tokio::test]
    async fn test_load_malformed_config() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let tmp = tempdir().unwrap();
        set_test_config_dir(tmp.path());
        let harv_dir = tmp.path().join(".config").join("harv");
        std::fs::create_dir_all(&harv_dir).unwrap();
        std::fs::write(harv_dir.join("config.toml"), "not valid toml = = =").unwrap();

        let result = HarvConfig::load().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_default_cache_ttl() {
        let toml = r#"
access_token = "tok"
account_id = "1"
"#;
        let config: HarvConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.cache_ttl_hours, 24);
    }

    #[test]
    fn legacy_config_defaults_to_quick_oauth() {
        let config: HarvConfig = toml::from_str(
            r#"
access_token = "tok"
account_id = "1"
"#,
        )
        .unwrap();
        assert_eq!(config.auth_method(), AuthMethod::QuickOAuth);
        assert_eq!(config.refresh_token(), None);
        assert_eq!(config.access_token_expires_at(), None);
    }

    #[test]
    fn serializes_refreshable_credentials_without_omitting_them() {
        let mut config = test_config();
        config.set_authentication(Authentication::new(
            AuthMethod::RefreshableOAuth,
            "access".into(),
            "1".into(),
            Some(Utc::now()),
            Some("refresh".into()),
            Some("client".into()),
            Some("secret".into()),
        ));
        let serialized = toml::to_string(&config).unwrap();
        assert!(serialized.contains("auth_method = \"refreshable-oauth\""));
        assert!(serialized.contains("refresh_token = \"refresh\""));
        let loaded: HarvConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(loaded.auth_method(), AuthMethod::RefreshableOAuth);
        assert_eq!(loaded.oauth_client_secret(), Some("secret"));
    }

    #[test]
    fn test_deserialize_custom_cache_ttl() {
        let toml = r#"
access_token = "tok"
account_id = "1"
cache_ttl_hours = 48
"#;
        let config: HarvConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.cache_ttl_hours, 48);
    }

    #[test]
    fn test_deserialize_last_used_default_none() {
        let toml = r#"
access_token = "tok"
account_id = "1"
"#;
        let config: HarvConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.last_project_id, None);
        assert_eq!(config.last_task_id, None);
    }

    #[test]
    fn test_set_last_used() {
        let mut config = test_config();
        config.set_last_used(42, 99);
        assert_eq!(config.last_project_id, Some(42));
        assert_eq!(config.last_task_id, Some(99));
    }
}
