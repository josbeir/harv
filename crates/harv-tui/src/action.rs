use chrono::NaiveDate;
use harv_core::{ProjectAssignment, TimeEntry, User};

use crate::theme::ThemeMode;

#[derive(Debug, Clone)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum Action {
    Quit,
    Tick,
    SwitchView(ViewId),
    Refresh,
    RefreshEntries,
    TimerUpdate(Vec<TimeEntry>),
    TodayEntriesUpdate(Vec<TimeEntry>, f64, usize),
    UserLoaded(User),
    NavigateDayPrev,
    NavigateDayNext,
    NavigateDayToday,
    OpenDatePicker,
    CloseDatePicker,
    SelectDate(NaiveDate),
    OpenForm {
        last_project_id: Option<u64>,
        last_task_id: Option<u64>,
        project_name: Option<String>,
        mode: FormMode,
        entry_id: Option<u64>,
        entry_date: Option<String>,
        entry_hours: Option<String>,
        entry_notes: Option<String>,
        is_running: bool,
    },
    FormAssignmentsUpdate(Vec<ProjectAssignment>),
    CreateEntry {
        project_id: u64,
        task_id: u64,
        spent_date: String,
        hours: Option<f64>,
        notes: Option<String>,
    },
    EditEntry {
        entry_id: u64,
        project_id: u64,
        task_id: u64,
        spent_date: String,
        hours: Option<f64>,
        notes: Option<String>,
    },
    StopTimer {
        entry_id: u64,
    },
    DeleteEntry {
        entry_id: u64,
    },
    StopAndStartNew {
        entry_id: u64,
    },
    ConfirmStopAndStart {
        entry_id: u64,
        entry_desc: String,
    },
    ConfirmDelete {
        entry_id: u64,
        entry_desc: String,
    },
    SetLoadingMessage(String),
    Error(String),
    ToggleHelp,
    ThemeChanged(ThemeMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormMode {
    Start,
    Create,
    Edit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewId {
    Dashboard,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_mode_equality() {
        assert_eq!(FormMode::Start, FormMode::Start);
        assert_ne!(FormMode::Start, FormMode::Create);
        assert_ne!(FormMode::Create, FormMode::Edit);
    }

    #[test]
    fn test_view_id_equality() {
        assert_eq!(ViewId::Dashboard, ViewId::Dashboard);
    }

    #[test]
    fn test_open_form_action_construction() {
        let action = Action::OpenForm {
            last_project_id: Some(1),
            last_task_id: Some(2),
            project_name: Some("Test".into()),
            mode: FormMode::Create,
            entry_id: None,
            entry_date: None,
            entry_hours: None,
            entry_notes: None,
            is_running: false,
        };
        assert!(matches!(
            action,
            Action::OpenForm {
                mode: FormMode::Create,
                ..
            }
        ));
    }

    #[test]
    fn test_create_entry_construction() {
        let action = Action::CreateEntry {
            project_id: 1,
            task_id: 2,
            spent_date: "2026-06-09".into(),
            hours: Some(1.5),
            notes: Some("test".into()),
        };
        assert!(matches!(
            action,
            Action::CreateEntry {
                project_id: 1,
                task_id: 2,
                ..
            }
        ));
    }

    #[test]
    fn test_edit_entry_construction() {
        let action = Action::EditEntry {
            entry_id: 42,
            project_id: 1,
            task_id: 2,
            spent_date: "2026-06-09".into(),
            hours: Some(2.0),
            notes: None,
        };
        assert!(matches!(action, Action::EditEntry { entry_id: 42, .. }));
    }
}
