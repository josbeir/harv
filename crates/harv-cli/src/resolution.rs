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
