use harv_core::HarvError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;

/// Configuration stored at `~/.config/harv/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvConfig {
    pub access_token: String,
    pub account_id: String,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_hours: u64,
    #[serde(default)]
    pub last_project_id: Option<u64>,
    #[serde(default)]
    pub last_task_id: Option<u64>,
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default)]
    pub aliases: HashMap<String, Alias>,
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
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let toml =
            toml::to_string_pretty(self).map_err(|e| HarvError::ConfigMalformed(e.to_string()))?;
        fs::write(&path, toml).await?;
        Ok(())
    }

    /// Returns the path to the config file: `~/.config/harv/config.toml`.
    pub fn path() -> PathBuf {
        dirs::config_dir()
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
        self.aliases.insert(name.to_string(), alias);
        self.save().await
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::const_new(());

    fn test_config() -> HarvConfig {
        HarvConfig {
            access_token: "test-token".into(),
            account_id: "1234567".into(),
            cache_ttl_hours: 24,
            last_project_id: None,
            last_task_id: None,
            locale: None,
            aliases: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_load_nonexistent() {
        let _guard = ENV_MUTEX.lock().await;
        let dir = tempdir().unwrap();
        unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
        unsafe { std::env::set_var("HOME", dir.path()) };
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

    #[test]
    fn test_path_ends_with_config_toml() {
        let path = HarvConfig::path();
        assert!(path.to_string_lossy().contains("harv"));
        assert!(path.ends_with("config.toml"));
    }

    #[tokio::test]
    async fn test_save_load_with_tempdir() {
        let _guard = ENV_MUTEX.lock().await;
        let tmp = tempdir().unwrap();
        unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
        unsafe { std::env::set_var("HOME", tmp.path()) };
        let harv_dir = tmp.path().join(".config").join("harv");
        std::fs::create_dir_all(&harv_dir).unwrap();

        let config = test_config();
        config.save().await.unwrap();

        let loaded = HarvConfig::load().await.unwrap();
        assert_eq!(loaded.access_token, "test-token");
        assert_eq!(loaded.account_id, "1234567");
    }

    #[tokio::test]
    async fn test_save_set_and_remove_alias() {
        let _guard = ENV_MUTEX.lock().await;
        let tmp = tempdir().unwrap();
        unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
        unsafe { std::env::set_var("HOME", tmp.path()) };
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
        let _guard = ENV_MUTEX.lock().await;
        let tmp = tempdir().unwrap();
        unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
        unsafe { std::env::set_var("HOME", tmp.path()) };
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
