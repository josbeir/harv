use harv_core::{ProjectAssignment, TaskAssignment, TimeEntry, User};

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
    UserLoaded(User),
    OpenForm {
        last_project_id: Option<u64>,
        last_task_id: Option<u64>,
        project_name: Option<String>,
        mode: FormMode,
        entry_id: Option<u64>,
        entry_date: Option<String>,
        entry_hours: Option<String>,
        entry_notes: Option<String>,
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
    EditEntry {
        entry_id: u64,
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
pub enum FormMode {
    Start,
    Create,
    Edit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewId {
    Dashboard,
}
