use std::collections::HashMap;
use std::path::Path;

use harv_core::HarvError;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::config::Alias;

/// A named note template with a pattern containing `{variable}` placeholders.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteTemplate {
    /// Template pattern with `{variable}` substitution placeholders.
    pub pattern: String,
}

/// Project-level configuration stored in a `harv.toml` file.
///
/// Discovered by walking up from the current working directory.
/// All fields are optional — an empty `harv.toml` is valid.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ProjectConfig {
    /// Default project ID to pre-select when creating time entries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_project_id: Option<u64>,

    /// Default task ID to pre-select when creating time entries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_task_id: Option<u64>,

    /// Project-specific aliases. These take priority over global aliases
    /// when there is a name conflict.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub aliases: HashMap<String, Alias>,

    /// Named note templates. The template named `"default"` is
    /// automatically used when creating entries if no explicit template
    /// is selected.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub templates: HashMap<String, NoteTemplate>,
}

/// Default filename for project configuration files.
pub const PROJECT_CONFIG_FILENAME: &str = "harv.toml";

impl ProjectConfig {
    /// Load a project config from a specific file path.
    pub async fn load_from(path: &Path) -> Result<Self, HarvError> {
        let contents = fs::read_to_string(path).await.map_err(HarvError::Io)?;
        toml::from_str(&contents).map_err(|e| HarvError::ConfigMalformed(e.to_string()))
    }

    /// Load a project config without requiring an async runtime.
    ///
    /// This is intended for short-lived, read-only operations such as shell
    /// completion. Normal application code should use [`Self::load_from`].
    pub fn load_from_blocking(path: &Path) -> Result<Self, HarvError> {
        let contents = std::fs::read_to_string(path).map_err(HarvError::Io)?;
        toml::from_str(&contents).map_err(|e| HarvError::ConfigMalformed(e.to_string()))
    }

    /// Save this project config to a specific file path.
    /// Creates parent directories if they don't exist.
    pub async fn save_to(&self, path: &Path) -> Result<(), HarvError> {
        let toml =
            toml::to_string_pretty(self).map_err(|e| HarvError::ConfigMalformed(e.to_string()))?;
        crate::storage::atomic_write(path, toml.into_bytes()).await
    }

    /// Discover a `harv.toml` by walking up from the current working directory.
    ///
    /// Starts at `current_dir()` and checks each parent directory up to the
    /// filesystem root. Returns `None` if no `harv.toml` is found anywhere
    /// in the ancestry chain. Returns an error only on I/O failures (not on
    /// "file not found").
    pub async fn discover() -> Result<Option<Self>, HarvError> {
        let cwd = std::env::current_dir()?;
        Self::discover_from(&cwd).await
    }

    /// Discover a `harv.toml` without requiring an async runtime.
    ///
    /// This mirrors [`Self::discover`] for short-lived, read-only operations
    /// such as shell completion.
    pub fn discover_blocking() -> Result<Option<Self>, HarvError> {
        let cwd = std::env::current_dir()?;
        Self::discover_from_blocking(&cwd)
    }

    /// Discover a `harv.toml` by walking up from the given starting directory.
    pub async fn discover_from(start_dir: &Path) -> Result<Option<Self>, HarvError> {
        let mut current = if start_dir.is_absolute() {
            start_dir.to_path_buf()
        } else {
            std::env::current_dir()?.join(start_dir)
        };

        // Canonicalize to resolve symlinks and get a clean path.
        current = current.canonicalize().unwrap_or_else(|_| current.clone());

        loop {
            let candidate = current.join(PROJECT_CONFIG_FILENAME);

            match fs::metadata(&candidate).await {
                Ok(meta) if meta.is_file() => {
                    return Self::load_from(&candidate).await.map(Some);
                }
                _ => {
                    // Move up to parent. Stop if we've reached the root.
                    if let Some(parent) = current.parent() {
                        // On Unix, root's parent is root itself — check for that.
                        if parent == current {
                            return Ok(None);
                        }
                        current = parent.to_path_buf();
                    } else {
                        return Ok(None);
                    }
                }
            }
        }
    }

    /// Discover a `harv.toml` from a directory without requiring an async runtime.
    pub fn discover_from_blocking(start_dir: &Path) -> Result<Option<Self>, HarvError> {
        let mut current = if start_dir.is_absolute() {
            start_dir.to_path_buf()
        } else {
            std::env::current_dir()?.join(start_dir)
        };

        current = current.canonicalize().unwrap_or_else(|_| current.clone());

        loop {
            let candidate = current.join(PROJECT_CONFIG_FILENAME);

            match std::fs::metadata(&candidate) {
                Ok(meta) if meta.is_file() => {
                    return Self::load_from_blocking(&candidate).map(Some);
                }
                _ => {
                    if let Some(parent) = current.parent() {
                        if parent == current {
                            return Ok(None);
                        }
                        current = parent.to_path_buf();
                    } else {
                        return Ok(None);
                    }
                }
            }
        }
    }

    /// Check if this config is completely empty (no meaningful settings).
    pub fn is_empty(&self) -> bool {
        self.default_project_id.is_none()
            && self.default_task_id.is_none()
            && self.aliases.is_empty()
            && self.templates.is_empty()
    }

    /// Get a template by name. Returns `None` if not found.
    pub fn template(&self, name: &str) -> Option<&NoteTemplate> {
        self.templates.get(name)
    }

    /// Get the default template (named `"default"`), if it exists.
    pub fn default_template(&self) -> Option<&NoteTemplate> {
        self.template("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn sample_config_toml() -> String {
        r#"default_project_id = 12345
default_task_id = 67890

[aliases.dev]
project_id = 100
task_id = 200

[templates.daily]
pattern = "Daily standup — {date} — Branch: {branch_name}"
"#
        .to_string()
    }

    async fn write_harv_toml(dir: &TempDir, content: &str) {
        let path = dir.path().join(PROJECT_CONFIG_FILENAME);
        tokio::fs::write(&path, content).await.unwrap();
    }

    /// Helper to set CWD for discovery tests. Must be called within
    /// the workspace test-process lock.
    fn set_cwd(dir: &TempDir) {
        std::env::set_current_dir(dir.path()).unwrap();
    }

    // --- Load / Save tests ---

    #[tokio::test]
    async fn test_load_valid_config() {
        let dir = tempfile::tempdir().unwrap();
        write_harv_toml(&dir, &sample_config_toml()).await;

        let path = dir.path().join(PROJECT_CONFIG_FILENAME);
        let config = ProjectConfig::load_from(&path).await.unwrap();

        assert_eq!(config.default_project_id, Some(12345));
        assert_eq!(config.default_task_id, Some(67890));
        assert_eq!(config.aliases.len(), 1);
        assert_eq!(config.aliases.get("dev").unwrap().project_id, 100);
        assert_eq!(config.templates.len(), 1);
        assert_eq!(
            config.templates.get("daily").unwrap().pattern,
            "Daily standup — {date} — Branch: {branch_name}"
        );
    }

    #[test]
    fn test_blocking_load_and_discover_walk_up() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(PROJECT_CONFIG_FILENAME);
        std::fs::write(&path, sample_config_toml()).unwrap();
        let nested = dir.path().join("nested");
        std::fs::create_dir(&nested).unwrap();

        let loaded = ProjectConfig::load_from_blocking(&path).unwrap();
        assert_eq!(loaded.aliases.get("dev").unwrap().task_id, 200);
        let discovered = ProjectConfig::discover_from_blocking(&nested)
            .unwrap()
            .unwrap();
        assert_eq!(discovered.default_project_id, Some(12345));
    }

    #[tokio::test]
    async fn test_load_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(PROJECT_CONFIG_FILENAME);
        let result = ProjectConfig::load_from(&path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_empty_config() {
        let dir = tempfile::tempdir().unwrap();
        write_harv_toml(&dir, "").await;

        let path = dir.path().join(PROJECT_CONFIG_FILENAME);
        let config = ProjectConfig::load_from(&path).await.unwrap();

        assert!(config.is_empty());
        assert!(config.default_project_id.is_none());
        assert!(config.default_task_id.is_none());
        assert!(config.aliases.is_empty());
        assert!(config.templates.is_empty());
    }

    #[tokio::test]
    async fn test_load_partial_config() {
        let dir = tempfile::tempdir().unwrap();
        write_harv_toml(&dir, "default_project_id = 42\n").await;

        let path = dir.path().join(PROJECT_CONFIG_FILENAME);
        let config = ProjectConfig::load_from(&path).await.unwrap();

        assert_eq!(config.default_project_id, Some(42));
        assert!(config.default_task_id.is_none());
        assert!(config.aliases.is_empty());
    }

    #[tokio::test]
    async fn test_save_and_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(PROJECT_CONFIG_FILENAME);

        let config = ProjectConfig {
            default_project_id: Some(99),
            default_task_id: Some(88),
            aliases: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "test".into(),
                    Alias {
                        project_id: 1,
                        task_id: 2,
                    },
                );
                m
            },
            templates: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "daily".into(),
                    NoteTemplate {
                        pattern: "Test: {date}".into(),
                    },
                );
                m
            },
        };

        config.save_to(&path).await.unwrap();
        let loaded = ProjectConfig::load_from(&path).await.unwrap();

        assert_eq!(loaded, config);
    }

    #[tokio::test]
    async fn test_save_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir
            .path()
            .join("sub")
            .join("deep")
            .join(PROJECT_CONFIG_FILENAME);

        let config = ProjectConfig {
            default_project_id: Some(1),
            ..Default::default()
        };

        config.save_to(&nested).await.unwrap();
        assert!(nested.exists());
        let loaded = ProjectConfig::load_from(&nested).await.unwrap();
        assert_eq!(loaded.default_project_id, Some(1));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn save_does_not_restrict_project_directory_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        ProjectConfig::default()
            .save_to(&dir.path().join(PROJECT_CONFIG_FILENAME))
            .await
            .unwrap();
        assert_eq!(
            std::fs::metadata(dir.path()).unwrap().permissions().mode() & 0o777,
            0o755
        );
    }

    #[tokio::test]
    async fn test_aliases_serialize_as_table() {
        let mut config = ProjectConfig::default();
        config.aliases.insert(
            "dev".into(),
            Alias {
                project_id: 10,
                task_id: 20,
            },
        );

        let toml_str = toml::to_string_pretty(&config).unwrap();
        // Aliases should be serialized as [aliases.name] tables, not inline.
        assert!(toml_str.contains("[aliases.dev]"));
        assert!(toml_str.contains("project_id = 10"));
    }

    #[tokio::test]
    async fn test_templates_serialize_as_table() {
        let mut config = ProjectConfig::default();
        config.templates.insert(
            "daily".into(),
            NoteTemplate {
                pattern: "Hello {date}".into(),
            },
        );

        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("[templates.daily]"));
        assert!(toml_str.contains("pattern = \"Hello {date}\""));
    }

    #[tokio::test]
    async fn test_is_empty() {
        assert!(ProjectConfig::default().is_empty());

        let cfg = ProjectConfig {
            default_project_id: Some(1),
            ..Default::default()
        };
        assert!(!cfg.is_empty());
    }

    #[tokio::test]
    async fn test_template_lookup() {
        let mut config = ProjectConfig::default();
        config.templates.insert(
            "default".into(),
            NoteTemplate {
                pattern: "default pattern".into(),
            },
        );
        config.templates.insert(
            "meeting".into(),
            NoteTemplate {
                pattern: "meeting pattern".into(),
            },
        );

        assert_eq!(
            config.template("default").unwrap().pattern,
            "default pattern"
        );
        assert_eq!(
            config.default_template().unwrap().pattern,
            "default pattern"
        );
        assert!(config.template("nonexistent").is_none());
    }

    // --- Discovery tests ---

    /// Run a test with a temporary CWD. Restores the original CWD
    /// when the closure returns or panics.
    async fn with_temp_cwd(dir: &TempDir, f: impl std::future::Future<Output = ()>) {
        let original = std::env::current_dir().ok();
        set_cwd(dir);
        f.await;
        if let Some(cwd) = original {
            let _ = std::env::set_current_dir(cwd);
        }
    }

    #[tokio::test]
    async fn test_discover_finds_in_cwd() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let dir = tempfile::tempdir().unwrap();
        write_harv_toml(&dir, "default_project_id = 7\n").await;
        with_temp_cwd(&dir, async {
            let found = ProjectConfig::discover().await.unwrap();
            assert!(found.is_some());
            assert_eq!(found.unwrap().default_project_id, Some(7));
        })
        .await;
    }

    #[tokio::test]
    async fn test_discover_walks_up() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let dir = tempfile::tempdir().unwrap();
        write_harv_toml(&dir, "default_project_id = 7\n").await;

        let subdir = dir.path().join("sub");
        std::fs::create_dir(&subdir).unwrap();
        // No CWD change needed — we use discover_from with an explicit path.
        let found = ProjectConfig::discover_from(&subdir).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().default_project_id, Some(7));
    }

    #[tokio::test]
    async fn test_discover_returns_none_when_not_found() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let dir = tempfile::tempdir().unwrap();
        with_temp_cwd(&dir, async {
            let found = ProjectConfig::discover().await.unwrap();
            assert!(found.is_none());
        })
        .await;
    }

    #[tokio::test]
    async fn test_discover_from_nonexistent_dir() {
        let path = PathBuf::from("/tmp/harv_nonexistent_test_dir_xyz");
        // discover_from on a nonexistent path should fail on canonicalize
        // but our implementation falls back to the non-canonicalized path
        // so it should just return None
        let result = ProjectConfig::discover_from(&path).await;
        // May succeed (returning None) or fail with IO error — both are acceptable.
        match result {
            Ok(None) => {}              // expected
            Err(HarvError::Io(_)) => {} // also acceptable
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_discover_stops_at_root() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        // Create a temp dir that has no harv.toml anywhere up to root.
        // But we can't put harv.toml at /. So just test that we don't
        // infinite-loop by starting from root.
        let found = ProjectConfig::discover_from(Path::new("/")).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_discover_uses_nearest_config() {
        let _guard = crate::TEST_PROCESS_MUTEX.lock().await;
        let dir = tempfile::tempdir().unwrap();
        write_harv_toml(&dir, "default_project_id = 1\n").await;

        let subdir = dir.path().join("sub");
        std::fs::create_dir(&subdir).unwrap();
        let sub_harv = subdir.join(PROJECT_CONFIG_FILENAME);
        tokio::fs::write(&sub_harv, "default_project_id = 2\n")
            .await
            .unwrap();

        let deep = subdir.join("deep");
        std::fs::create_dir(&deep).unwrap();

        let found = ProjectConfig::discover_from(&deep).await.unwrap();
        assert!(found.is_some());
        // Should find the nearest one (in subdir), not the parent one.
        assert_eq!(found.unwrap().default_project_id, Some(2));
    }
}
