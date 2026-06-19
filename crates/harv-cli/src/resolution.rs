//! Shared project/task resolution helpers used across CLI commands.

use crate::prompts::ProjectChoice;

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
}
