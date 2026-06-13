use chrono::NaiveDate;
use harv_core::text::format_project_display;
use harv_core::{ProjectAssignment, Reference, TaskAssignment};
use harv_sdk::HarvConfig;
use inquire::{CustomType, Select, Text, validator::Validation};

/// Prompt for an alias name (non-empty, no whitespace).
pub fn prompt_alias_name() -> color_eyre::eyre::Result<String> {
    let validator = |input: &str| {
        if input.trim().is_empty() {
            return Ok(Validation::Invalid("Alias name cannot be empty".into()));
        }
        if input.contains(char::is_whitespace) {
            return Ok(Validation::Invalid(
                "Alias name cannot contain spaces".into(),
            ));
        }
        Ok(Validation::Valid)
    };
    Text::new("Alias name:")
        .with_validator(validator)
        .prompt()
        .map_err(Into::into)
}

use crate::OutputFormat;

/// Project choice for the prompt.
pub struct ProjectChoice {
    pub display: String,
    pub project_id: u64,
    pub task_assignments: Vec<TaskAssignment>,
}

/// Build project choices from assignments. If `last_project_id` is set,
/// the matching project is moved to the top with a ● prefix.
pub fn build_project_choices(
    assignments: &[ProjectAssignment],
    last_project_id: Option<u64>,
) -> Vec<ProjectChoice> {
    let mut choices: Vec<ProjectChoice> = assignments
        .iter()
        .filter(|a| !a.task_assignments.is_empty())
        .map(|a| ProjectChoice {
            display: format!(
                "{} => {}",
                a.client
                    .as_ref()
                    .map(|c| c.name.as_str())
                    .unwrap_or("No client"),
                format_project_display(&a.project.name, a.project_code.as_deref())
            ),
            project_id: a.project.id,
            task_assignments: a.task_assignments.clone(),
        })
        .collect();
    choices.sort_by(|a, b| a.display.cmp(&b.display));

    #[allow(clippy::collapsible_if)]
    if let Some(pid) = last_project_id {
        if let Some(idx) = choices.iter().position(|c| c.project_id == pid) {
            let mut choice = choices.remove(idx);
            choice.display = format!("● {}", choice.display);
            choices.insert(0, choice);
        }
    }

    choices
}

/// Interactive prompt to select a project.
/// `starting_cursor` sets which item is pre-selected (0-based).
pub fn pick_project(
    choices: &[ProjectChoice],
    starting_cursor: usize,
) -> color_eyre::eyre::Result<&ProjectChoice> {
    let choice_items: Vec<&str> = choices.iter().map(|c| c.display.as_str()).collect();
    let selection = Select::new("Project:", choice_items.clone())
        .with_starting_cursor(starting_cursor)
        .prompt()?;

    let idx = choice_items
        .iter()
        .position(|&c| c == selection)
        .expect("selected item should exist");
    Ok(&choices[idx])
}

/// Interactive prompt to select a task for a given project choice.
pub fn pick_task(choice: &ProjectChoice) -> color_eyre::eyre::Result<&TaskAssignment> {
    let items: Vec<&str> = choice
        .task_assignments
        .iter()
        .map(|t| t.task.name.as_str())
        .collect();
    let selection = Select::new("Task:", items.clone()).prompt()?;

    let idx = items
        .iter()
        .position(|&c| c == selection)
        .expect("selected item should exist");
    Ok(&choice.task_assignments[idx])
}

/// Find a project choice by alias name. Returns None if not found.
#[allow(dead_code)]
pub fn resolve_alias<'a>(
    config: &HarvConfig,
    choices: &'a [ProjectChoice],
    alias_name: &str,
) -> Option<&'a ProjectChoice> {
    let alias = config.alias(alias_name)?;
    choices.iter().find(|c| c.project_id == alias.project_id)
}

/// Prompt for a date, defaulting to today.
pub fn ask_date(default: NaiveDate) -> color_eyre::eyre::Result<NaiveDate> {
    let default_str = default.format("%Y-%m-%d").to_string();
    let validator = |input: &str| {
        if input.is_empty() {
            return Ok(Validation::Valid);
        }
        match NaiveDate::parse_from_str(input, "%Y-%m-%d") {
            Ok(d) if d <= harv_core::datetime::today() => Ok(Validation::Valid),
            Ok(_) => Ok(Validation::Invalid("Date cannot be in the future".into())),
            Err(_) => Ok(Validation::Invalid(
                "Invalid date format (YYYY-MM-DD)".into(),
            )),
        }
    };

    let input = Text::new("Date:")
        .with_default(&default_str)
        .with_validator(validator)
        .prompt()?;

    if input.is_empty() || input == default_str {
        return Ok(default);
    }
    NaiveDate::parse_from_str(&input, "%Y-%m-%d")
        .map_err(|_| color_eyre::eyre::eyre!("Invalid date: {}", input))
}

/// Prompt for hours. Empty or 0 = start a running timer.
/// Accepts decimal (1.5) or HH:MM (1:30) format.
pub fn ask_hours() -> color_eyre::eyre::Result<Option<f64>> {
    let input = CustomType::<String>::new("Hours (0 to start timer, e.g. 1.5 or 1:30):")
        .with_default("".into())
        .with_validator(|input: &String| {
            if input.is_empty() {
                return Ok(Validation::Valid);
            }
            match harv_core::datetime::parse_hours(input) {
                Ok(h) if h >= 0.0 => Ok(Validation::Valid),
                Ok(_) => Ok(Validation::Invalid("Hours must be non-negative".into())),
                Err(e) => Ok(Validation::Invalid(
                    inquire::validator::ErrorMessage::Custom(e),
                )),
            }
        })
        .prompt()?;

    if input.is_empty() {
        return Ok(None);
    }
    let hours =
        harv_core::datetime::parse_hours(&input).map_err(|e| color_eyre::eyre::eyre!("{}", e))?;
    if hours == 0.0 {
        Ok(None)
    } else {
        Ok(Some(hours))
    }
}

/// Prompt for optional notes. Returns None if empty.
pub fn ask_notes(use_editor: bool) -> color_eyre::eyre::Result<Option<String>> {
    if use_editor {
        let notes = Text::new("Notes (opens $EDITOR, empty to skip):")
            .prompt_skippable()?
            .filter(|s| !s.trim().is_empty());
        return Ok(notes);
    }

    let notes = Text::new("Notes (empty to skip):")
        .prompt_skippable()?
        .filter(|s| !s.trim().is_empty());
    Ok(notes)
}

/// Prompt for notes with a default value pre-filled (from a template).
/// Returns None if the user clears the input. Returns the default if
/// the user accepts without changes.
pub fn ask_notes_with_default(default: &str) -> color_eyre::eyre::Result<Option<String>> {
    let notes = Text::new("Notes (template expanded, edit or press enter to accept):")
        .with_default(default)
        .prompt_skippable()?
        .filter(|s| !s.trim().is_empty());
    Ok(notes)
}

/// Interactive prompt to select a project, pre-selecting a default.
/// Returns (project_id, task_assignments).
pub fn pick_project_with_default(
    choices: &[ProjectChoice],
    default_project_id: u64,
) -> color_eyre::eyre::Result<(u64, Vec<TaskAssignment>)> {
    let choice_items: Vec<&str> = choices.iter().map(|c| c.display.as_str()).collect();
    let starting = choices
        .iter()
        .position(|c| c.project_id == default_project_id)
        .unwrap_or(0);
    let selection = Select::new("Project:", choice_items.clone())
        .with_starting_cursor(starting)
        .prompt()?;

    let idx = choice_items
        .iter()
        .position(|&c| c == selection)
        .expect("selected item should exist");
    Ok((
        choices[idx].project_id,
        choices[idx].task_assignments.clone(),
    ))
}

/// Interactive prompt to select a task, pre-selecting a default.
pub fn pick_task_with_default(
    task_assignments: &[TaskAssignment],
    default_task_id: u64,
) -> color_eyre::eyre::Result<TaskAssignment> {
    let items: Vec<&str> = task_assignments
        .iter()
        .map(|t| t.task.name.as_str())
        .collect();
    let starting = items
        .iter()
        .position(|&name| {
            task_assignments
                .iter()
                .any(|t| t.task.name == name && t.task.id == default_task_id)
        })
        .unwrap_or(0);
    let selection = Select::new("Task:", items.clone())
        .with_starting_cursor(starting)
        .prompt()?;

    let idx = items
        .iter()
        .position(|&c| c == selection)
        .expect("selected item should exist");
    Ok(task_assignments[idx].clone())
}

/// Prompt for a date with a specific default, skippable (returns None to keep current).
pub fn ask_date_with_default(default: NaiveDate) -> color_eyre::eyre::Result<Option<NaiveDate>> {
    let default_str = default.format("%Y-%m-%d").to_string();
    let validator = |input: &str| {
        if input.is_empty() {
            return Ok(Validation::Valid);
        }
        match NaiveDate::parse_from_str(input, "%Y-%m-%d") {
            Ok(d) if d <= harv_core::datetime::today() => Ok(Validation::Valid),
            Ok(_) => Ok(Validation::Invalid("Date cannot be in the future".into())),
            Err(_) => Ok(Validation::Invalid(
                "Invalid date format (YYYY-MM-DD)".into(),
            )),
        }
    };

    let input = Text::new("Date (empty to keep current):")
        .with_default(&default_str)
        .with_validator(validator)
        .prompt()?;

    if input.is_empty() || input == default_str {
        return Ok(None);
    }
    NaiveDate::parse_from_str(&input, "%Y-%m-%d")
        .map(Some)
        .map_err(|_| color_eyre::eyre::eyre!("Invalid date: {}", input))
}

/// Prompt for hours with the current value as default.
/// Returns None to keep existing hours, Some(0.0) to clear, Some(h) to change.
pub fn ask_hours_with_default(current: Option<f64>) -> color_eyre::eyre::Result<Option<f64>> {
    let default_str = current.map(|h| format!("{:.2}", h)).unwrap_or_default();
    let input = CustomType::<String>::new("Hours (empty to keep, 0 to clear):")
        .with_default(default_str.clone())
        .with_validator(|input: &String| {
            if input.is_empty() {
                return Ok(Validation::Valid);
            }
            match harv_core::datetime::parse_hours(input) {
                Ok(h) if h >= 0.0 => Ok(Validation::Valid),
                Ok(_) => Ok(Validation::Invalid("Hours must be non-negative".into())),
                Err(e) => Ok(Validation::Invalid(
                    inquire::validator::ErrorMessage::Custom(e),
                )),
            }
        })
        .prompt()?;

    if input.is_empty() || input == default_str {
        return Ok(None);
    }
    let h =
        harv_core::datetime::parse_hours(&input).map_err(|e| color_eyre::eyre::eyre!("{}", e))?;
    Ok(Some(h))
}
pub fn format_entry_confirmation(
    hours: Option<f64>,
    project: &Reference,
    task: &Reference,
    date: NaiveDate,
    is_running: bool,
    id: u64,
) -> String {
    let hours_str = hours
        .map(harv_core::text::format_hours)
        .unwrap_or_else(|| "Running...".into());
    let status = if is_running { "Running" } else { "Not running" };
    format!(
        "{} — {} → {} → {}\n#{}\t| {}\t| {}",
        hours_str,
        harv_core::datetime::format_date(date),
        project.name,
        task.name,
        id,
        date.format("%b %d, %Y"),
        status,
    )
}

/// Get a formatted display for the current running timer.
pub fn format_timer_display(
    timer: &harv_core::TimeEntry,
    elapsed_minutes: Option<i64>,
    format: &OutputFormat,
) -> String {
    match format {
        OutputFormat::Table => {
            let elapsed = elapsed_minutes
                .map(|m| format!("{}h {}m", m / 60, m % 60))
                .unwrap_or_else(|| "unknown".into());
            format!(
                "Timer: {} → {} → {}\nStarted: {}\nElapsed: {}",
                timer
                    .client
                    .as_ref()
                    .map(|c| c.name.as_str())
                    .unwrap_or("No client"),
                timer.project.name,
                timer.task.name,
                timer
                    .timer_started_at
                    .map(|t| harv_core::datetime::format_local(t, true))
                    .unwrap_or_default(),
                elapsed
            )
        }
        OutputFormat::Json => serde_json::json!({
            "id": timer.id,
            "project": timer.project.name,
            "task": timer.task.name,
            "client": timer.client.as_ref().map(|c| &c.name),
            "started_at": timer.timer_started_at.map(|t| t.to_rfc3339()),
            "elapsed_minutes": elapsed_minutes,
        })
        .to_string(),
    }
}

#[allow(dead_code)]
fn fuzzy_score(pattern: &str, text: &str) -> i32 {
    harv_core::text::fuzzy_score(pattern, text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use harv_core::Reference;

    #[test]
    fn test_fuzzy_score_exact() {
        assert!(fuzzy_score("dev", "Development") > 0);
    }

    #[test]
    fn test_fuzzy_score_no_match() {
        assert_eq!(fuzzy_score("xyz", "Development"), -1);
    }

    #[test]
    fn test_fuzzy_score_substring() {
        assert!(fuzzy_score("De", "Development") > 0);
    }

    #[test]
    fn test_build_project_choices_sorts() {
        let assignments = vec![
            ProjectAssignment {
                id: 1,
                project: Reference {
                    id: 100,
                    name: "Beta".into(),
                },
                project_code: None,
                client: Some(Reference {
                    id: 1,
                    name: "Client".into(),
                }),
                task_assignments: vec![TaskAssignment {
                    id: 1,
                    task: Reference {
                        id: 200,
                        name: "Dev".into(),
                    },
                    billable: true,
                    hourly_rate: None,
                    is_active: true,
                    budget: None,
                }],
                is_active: true,
            },
            ProjectAssignment {
                id: 2,
                project: Reference {
                    id: 101,
                    name: "Alpha".into(),
                },
                project_code: None,
                client: Some(Reference {
                    id: 1,
                    name: "Client".into(),
                }),
                task_assignments: vec![],
                is_active: true,
            },
        ];
        let choices = build_project_choices(&assignments, None);
        assert_eq!(choices.len(), 1); // empty task_assignments filtered out
        assert_eq!(choices[0].display, "Client => Beta");
    }

    #[test]
    fn test_build_project_choices_with_last_used() {
        let assignments = vec![
            ProjectAssignment {
                id: 1,
                project: Reference {
                    id: 100,
                    name: "Beta".into(),
                },
                project_code: None,
                client: Some(Reference {
                    id: 1,
                    name: "Client".into(),
                }),
                task_assignments: vec![TaskAssignment {
                    id: 1,
                    task: Reference {
                        id: 200,
                        name: "Dev".into(),
                    },
                    billable: true,
                    hourly_rate: None,
                    is_active: true,
                    budget: None,
                }],
                is_active: true,
            },
            ProjectAssignment {
                id: 2,
                project: Reference {
                    id: 101,
                    name: "Alpha".into(),
                },
                project_code: None,
                client: None,
                task_assignments: vec![TaskAssignment {
                    id: 2,
                    task: Reference {
                        id: 201,
                        name: "Design".into(),
                    },
                    billable: false,
                    hourly_rate: None,
                    is_active: true,
                    budget: None,
                }],
                is_active: true,
            },
        ];
        let choices = build_project_choices(&assignments, Some(101));
        assert_eq!(choices.len(), 2);
        assert!(choices[0].display.starts_with("●"));
        assert!(choices[0].display.contains("Alpha"));
        assert_eq!(choices[1].display, "Client => Beta");
    }

    #[test]
    fn test_format_entry_confirmation_running() {
        let result = format_entry_confirmation(
            None,
            &Reference {
                id: 100,
                name: "Test Project".into(),
            },
            &Reference {
                id: 200,
                name: "Development".into(),
            },
            chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap(),
            true,
            42,
        );
        assert!(result.contains("Running..."));
        assert!(result.contains("Running"));
        assert!(result.contains("42"));
    }

    #[test]
    fn test_format_entry_confirmation_with_hours() {
        let result = format_entry_confirmation(
            Some(2.5),
            &Reference {
                id: 100,
                name: "Test Project".into(),
            },
            &Reference {
                id: 200,
                name: "Development".into(),
            },
            chrono::NaiveDate::from_ymd_opt(2026, 6, 8).unwrap(),
            false,
            42,
        );
        assert!(result.contains("2.50h"));
        assert!(result.contains("Not running"));
    }

    #[test]
    fn test_format_timer_display_json() {
        use crate::OutputFormat;
        use harv_core::TimeEntry;

        let timer = TimeEntry {
            id: 1,
            spent_date: None,
            hours: None,
            notes: None,
            is_running: true,
            timer_started_at: Some(chrono::Utc::now()),
            started_time: None,
            ended_time: None,
            project: Reference {
                id: 100,
                name: "Test Project".into(),
            },
            task: Reference {
                id: 200,
                name: "Development".into(),
            },
            user: Reference {
                id: 1,
                name: "User".into(),
            },
            client: Some(Reference {
                id: 1,
                name: "Test Client".into(),
            }),
            is_billed: false,
            billable: false,
            project_code: None,
            billable_rate: None,
            cost_rate: None,
            created_at: None,
            updated_at: None,
        };
        let result = format_timer_display(&timer, Some(30), &OutputFormat::Json);
        assert!(result.contains("\"project\""));
        assert!(result.contains("Test Project"));
        assert!(result.contains("30"));
    }
}
