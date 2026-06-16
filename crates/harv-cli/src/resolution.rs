//! Shared project/task resolution helpers used across CLI commands.

use harv_sdk::ResolvedConfig;

use crate::prompts::ProjectChoice;

/// Result of resolving a project and task for a time entry.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ResolvedSelection {
    pub project_id: u64,
    pub task_id: u64,
}

/// Resolve a project ID from either a CLI-provided ID or the resolved
/// config defaults. Returns `None` if the caller should fall back to
/// interactive selection.
#[allow(dead_code)]
pub fn resolve_project_id(cli_project_id: Option<u64>, resolved: &ResolvedConfig) -> Option<u64> {
    cli_project_id.or(resolved.default_project_id)
}

/// Look up a project choice by project ID.
#[allow(dead_code)]
pub fn find_choice_by_id(choices: &[ProjectChoice], project_id: u64) -> Option<&ProjectChoice> {
    choices.iter().find(|c| c.project_id == project_id)
}

/// Resolve a project + task from an alias name using the merged
/// (global + project) alias map.
#[allow(dead_code)]
pub fn resolve_alias(resolved: &ResolvedConfig, alias_name: &str) -> Option<ResolvedSelection> {
    resolved
        .resolve_alias(alias_name)
        .map(|a| ResolvedSelection {
            project_id: a.project_id,
            task_id: a.task_id,
        })
}

/// Build the starting cursor position for the project picker.
///
/// If the resolved default project matches one of the choices, return
/// its index. Otherwise return 0.
pub fn starting_cursor_for_default(
    choices: &[ProjectChoice],
    default_project_id: Option<u64>,
) -> usize {
    default_project_id
        .and_then(|pid| choices.iter().position(|c| c.project_id == pid))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_choice(pid: u64, display: &str) -> ProjectChoice {
        ProjectChoice {
            project_id: pid,
            display: display.into(),
            task_assignments: vec![],
        }
    }

    #[test]
    fn test_starting_cursor_finds_default() {
        let choices = vec![
            make_choice(10, "A"),
            make_choice(20, "B"),
            make_choice(30, "C"),
        ];
        assert_eq!(starting_cursor_for_default(&choices, Some(20)), 1);
    }

    #[test]
    fn test_starting_cursor_default_not_found() {
        let choices = vec![make_choice(10, "A"), make_choice(20, "B")];
        assert_eq!(starting_cursor_for_default(&choices, Some(999)), 0);
    }

    #[test]
    fn test_starting_cursor_no_default() {
        let choices = vec![make_choice(10, "A")];
        assert_eq!(starting_cursor_for_default(&choices, None), 0);
    }

    #[test]
    fn test_resolve_project_id_cli_takes_priority() {
        let config = harv_sdk::mock_data::test_config();
        let resolved = harv_sdk::ResolvedConfig::resolve(&config, None);
        assert_eq!(resolve_project_id(Some(42), &resolved), Some(42));
    }

    #[test]
    fn test_resolve_project_id_falls_back_to_default() {
        use harv_sdk::ProjectConfig;
        let mut config = harv_sdk::mock_data::test_config();
        config.last_project_id = Some(99);
        let pc = ProjectConfig {
            default_project_id: Some(55),
            default_task_id: Some(10),
            aliases: std::collections::HashMap::new(),
            templates: std::collections::HashMap::new(),
        };
        let resolved = harv_sdk::ResolvedConfig::resolve(&config, Some(&pc));
        assert_eq!(resolve_project_id(None, &resolved), Some(55));
    }

    #[test]
    fn test_find_choice_by_id_found() {
        let choices = vec![make_choice(10, "A"), make_choice(20, "B")];
        let found = find_choice_by_id(&choices, 20);
        assert!(found.is_some());
        assert_eq!(found.unwrap().project_id, 20);
    }

    #[test]
    fn test_find_choice_by_id_not_found() {
        let choices = vec![make_choice(10, "A")];
        assert!(find_choice_by_id(&choices, 999).is_none());
    }

    #[test]
    fn test_resolve_alias_found() {
        use harv_sdk::Alias;

        let mut config = harv_sdk::mock_data::test_config();
        config.aliases.insert(
            "dev".into(),
            Alias {
                project_id: 100,
                task_id: 200,
            },
        );
        let resolved = harv_sdk::ResolvedConfig::resolve(&config, None);
        let result = resolve_alias(&resolved, "dev");
        assert!(result.is_some());
        let sel = result.unwrap();
        assert_eq!(sel.project_id, 100);
        assert_eq!(sel.task_id, 200);
    }

    #[test]
    fn test_resolve_alias_not_found() {
        let config = harv_sdk::mock_data::test_config();
        let resolved = harv_sdk::ResolvedConfig::resolve(&config, None);
        assert!(resolve_alias(&resolved, "nonexistent").is_none());
    }
}
