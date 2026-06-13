use std::collections::HashMap;

use crate::config::{Alias, HarvConfig};
use crate::project_config::{NoteTemplate, ProjectConfig};

/// The effective configuration after merging global (`HarvConfig`) and
/// project-level (`ProjectConfig`) settings.
///
/// All values are owned (cloned from the source configs) so that
/// `ResolvedConfig` has no lifetime ties to its inputs.
#[derive(Debug, Clone, Default)]
pub struct ResolvedConfig {
    /// The default project ID to pre-select. Project config takes
    /// priority over the global `last_project_id`.
    pub default_project_id: Option<u64>,

    /// The default task ID to pre-select. Project config takes
    /// priority over the global `last_task_id`.
    pub default_task_id: Option<u64>,

    /// Merged aliases from both global and project configs.
    /// Project aliases take priority when there is a name conflict.
    pub aliases: HashMap<String, Alias>,

    /// Note templates from the project config only. Empty if no
    /// project config is present.
    pub templates: HashMap<String, NoteTemplate>,
}

impl ResolvedConfig {
    /// Build a resolved configuration by merging global and project configs.
    ///
    /// # Priority (highest to lowest)
    ///
    /// 1. Project config `default_project_id` / `default_task_id`
    /// 2. Global config `last_project_id` / `last_task_id`
    ///
    /// For aliases: project aliases override global aliases with the
    /// same name. Non-conflicting aliases from both sources are
    /// included.
    ///
    /// Templates come exclusively from the project config.
    pub fn resolve(global: &HarvConfig, project: Option<&ProjectConfig>) -> Self {
        let default_project_id = project
            .and_then(|p| p.default_project_id)
            .or(global.last_project_id);

        let default_task_id = project
            .and_then(|p| p.default_task_id)
            .or(global.last_task_id);

        // Merge aliases: start with global, then overlay project
        // (project entries win on name conflict).
        let mut aliases = global.aliases.clone();
        if let Some(proj) = project {
            for (name, alias) in &proj.aliases {
                aliases.insert(name.clone(), alias.clone());
            }
        }

        let templates = project.map(|p| p.templates.clone()).unwrap_or_default();

        Self {
            default_project_id,
            default_task_id,
            aliases,
            templates,
        }
    }

    /// Resolve an alias by name. Checks the merged alias map.
    /// Returns `None` if the alias is not found.
    pub fn resolve_alias(&self, name: &str) -> Option<&Alias> {
        self.aliases.get(name)
    }

    /// Get a named template from the project config.
    /// Returns `None` if no template with that name exists.
    pub fn resolve_template(&self, name: &str) -> Option<&NoteTemplate> {
        self.templates.get(name)
    }

    /// Get the default template (named `"default"`), if one exists.
    pub fn default_template(&self) -> Option<&NoteTemplate> {
        self.resolve_template("default")
    }

    /// Returns `true` if no meaningful configuration has been resolved
    /// (no defaults, no aliases, no templates).
    pub fn is_empty(&self) -> bool {
        self.default_project_id.is_none()
            && self.default_task_id.is_none()
            && self.aliases.is_empty()
            && self.templates.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn global_config() -> HarvConfig {
        HarvConfig {
            access_token: "tok".into(),
            account_id: "1".into(),
            cache_ttl_hours: 24,
            last_project_id: Some(100),
            last_task_id: Some(200),
            locale: None,
            aliases: {
                let mut m = HashMap::new();
                m.insert(
                    "global-only".into(),
                    Alias {
                        project_id: 1,
                        task_id: 2,
                    },
                );
                m.insert(
                    "conflict".into(),
                    Alias {
                        project_id: 10,
                        task_id: 20,
                    },
                );
                m
            },
        }
    }

    fn project_config() -> ProjectConfig {
        let mut aliases = HashMap::new();
        aliases.insert(
            "project-only".into(),
            Alias {
                project_id: 5,
                task_id: 6,
            },
        );
        // "conflict" alias — should override the global one
        aliases.insert(
            "conflict".into(),
            Alias {
                project_id: 99,
                task_id: 88,
            },
        );

        let mut templates = HashMap::new();
        templates.insert(
            "default".into(),
            NoteTemplate {
                pattern: "Daily: {date}".into(),
            },
        );

        ProjectConfig {
            default_project_id: Some(300),
            default_task_id: Some(400),
            aliases,
            templates,
        }
    }

    #[test]
    fn test_resolve_project_overrides_global() {
        let resolved = ResolvedConfig::resolve(&global_config(), Some(&project_config()));

        // Project defaults take priority
        assert_eq!(resolved.default_project_id, Some(300));
        assert_eq!(resolved.default_task_id, Some(400));
    }

    #[test]
    fn test_resolve_global_fallback() {
        let resolved = ResolvedConfig::resolve(&global_config(), None);

        // Falls back to global last_used
        assert_eq!(resolved.default_project_id, Some(100));
        assert_eq!(resolved.default_task_id, Some(200));
    }

    #[test]
    fn test_resolve_project_partial() {
        // Project has project_id but not task_id
        let pc = ProjectConfig {
            default_project_id: Some(300),
            ..Default::default()
        };

        let resolved = ResolvedConfig::resolve(&global_config(), Some(&pc));
        assert_eq!(resolved.default_project_id, Some(300));
        // Falls back to global for task
        assert_eq!(resolved.default_task_id, Some(200));
    }

    #[test]
    fn test_resolve_no_configs() {
        let empty_global = HarvConfig {
            access_token: "tok".into(),
            account_id: "1".into(),
            cache_ttl_hours: 24,
            last_project_id: None,
            last_task_id: None,
            locale: None,
            aliases: HashMap::new(),
        };

        let resolved = ResolvedConfig::resolve(&empty_global, None);

        assert!(resolved.default_project_id.is_none());
        assert!(resolved.default_task_id.is_none());
        assert!(resolved.aliases.is_empty());
        assert!(resolved.templates.is_empty());
        assert!(resolved.is_empty());
    }

    #[test]
    fn test_alias_merge_project_wins_conflict() {
        let resolved = ResolvedConfig::resolve(&global_config(), Some(&project_config()));

        // "conflict" should have the project's values (99/88), not global's (10/20)
        let alias = resolved.resolve_alias("conflict").unwrap();
        assert_eq!(alias.project_id, 99);
        assert_eq!(alias.task_id, 88);
    }

    #[test]
    fn test_alias_merge_includes_both_sources() {
        let resolved = ResolvedConfig::resolve(&global_config(), Some(&project_config()));

        // Both global-only and project-only aliases should be present
        assert!(resolved.resolve_alias("global-only").is_some());
        assert!(resolved.resolve_alias("project-only").is_some());
        assert_eq!(resolved.aliases.len(), 3); // global-only, project-only, conflict
    }

    #[test]
    fn test_alias_resolve_not_found() {
        let resolved = ResolvedConfig::resolve(&global_config(), None);
        assert!(resolved.resolve_alias("nonexistent").is_none());
    }

    #[test]
    fn test_template_from_project_only() {
        let resolved = ResolvedConfig::resolve(&global_config(), Some(&project_config()));

        let tpl = resolved.default_template().unwrap();
        assert_eq!(tpl.pattern, "Daily: {date}");
    }

    #[test]
    fn test_template_empty_without_project() {
        let resolved = ResolvedConfig::resolve(&global_config(), None);
        assert!(resolved.default_template().is_none());
        assert!(resolved.templates.is_empty());
    }

    #[test]
    fn test_is_empty() {
        let resolved = ResolvedConfig::resolve(&global_config(), None);
        assert!(!resolved.is_empty()); // has defaults and aliases

        let empty = ResolvedConfig::default();
        assert!(empty.is_empty());
    }
}
