use harv_core::{ProjectAssignment, TaskAssignment, TimeEntry};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Action {
    Quit,
    Tick,
    SwitchView(ViewId),
    NavigateUp,
    NavigateDown,
    Select,
    Refresh,
    TimerUpdate(Vec<TimeEntry>),
    TodayEntriesUpdate(Vec<TimeEntry>, f64),
    OpenForm {
        last_project_id: Option<u64>,
        last_task_id: Option<u64>,
        project_name: Option<String>,
        log_mode: bool,
    },
    FormAssignmentsUpdate(Vec<ProjectAssignment>),
    FormTasksUpdate(Vec<TaskAssignment>),
    FormSelectProject(u64),
    CreateEntry {
        project_id: u64,
        task_id: u64,
        spent_date: String,
        hours: Option<f64>,
        notes: Option<String>,
    },
    StartTimer {
        project_id: u64,
        task_id: u64,
    },
    StopTimer {
        entry_id: u64,
    },
    DeleteEntry {
        entry_id: u64,
    },
    ConfirmDelete {
        entry_id: u64,
        entry_desc: String,
    },
    Error(String),
    ToggleHelp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewId {
    Dashboard,
}
