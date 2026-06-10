use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// A lightweight reference to a Harvest resource.
/// Used pervasively in Harvest API responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reference {
    pub id: u64,
    pub name: String,
}

/// A time entry from the Harvest API.
///
/// <https://help.getharvest.com/api-v2/timesheets-api/timesheets/time-entries/>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimeEntry {
    pub id: u64,
    pub spent_date: Option<NaiveDate>,
    pub hours: Option<f64>,
    pub notes: Option<String>,
    #[serde(default)]
    pub is_running: bool,
    pub timer_started_at: Option<DateTime<Utc>>,
    pub started_time: Option<String>,
    pub ended_time: Option<String>,
    pub project: Reference,
    pub task: Reference,
    pub user: Reference,
    pub client: Option<Reference>,
    #[serde(default)]
    pub is_billed: bool,
    #[serde(default)]
    pub billable: bool,
    pub billable_rate: Option<f64>,
    pub cost_rate: Option<f64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Request body for creating a time entry via POST /v2/time_entries.
///
/// Omit `hours`, `started_time`, and `ended_time` to start a running timer.
/// Set `hours` to a positive value to create a completed time entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTimeEntry {
    pub project_id: u64,
    pub task_id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spent_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_time: Option<String>,
}

/// Request body for updating a time entry via PATCH /v2/time_entries/{id}.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTimeEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spent_date: Option<NaiveDate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hours: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_time: Option<String>,
}

/// A Harvest project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub id: u64,
    pub name: String,
    pub client: Option<Reference>,
    #[serde(default)]
    pub is_active: bool,
    pub code: Option<String>,
    pub notes: Option<String>,
    pub starts_on: Option<NaiveDate>,
    pub ends_on: Option<NaiveDate>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// A Harvest task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub billable_by_default: bool,
    pub default_hourly_rate: Option<f64>,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub is_active: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// A Harvest user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: u64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    #[serde(default)]
    pub is_active: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub timezone: Option<String>,
    pub weekly_capacity: Option<u64>,
    pub is_admin: Option<bool>,
    pub is_project_manager: Option<bool>,
}

/// A Harvest client.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Client {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub is_active: bool,
    pub address: Option<String>,
    pub currency: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// A Harvest company.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Company {
    pub name: String,
}

/// A project assignment for the authenticated user.
/// Returned by GET /v2/users/me/project_assignments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectAssignment {
    pub id: u64,
    pub project: Reference,
    pub client: Option<Reference>,
    pub task_assignments: Vec<TaskAssignment>,
    #[serde(default)]
    pub is_active: bool,
}

/// A task assignment within a project assignment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskAssignment {
    pub id: u64,
    pub task: Reference,
    #[serde(default)]
    pub billable: bool,
    pub hourly_rate: Option<f64>,
    #[serde(default)]
    pub is_active: bool,
    pub budget: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_deserialize() {
        let json = r#"{"id": 42, "name": "Development"}"#;
        let reference: Reference = serde_json::from_str(json).unwrap();
        assert_eq!(reference.id, 42);
        assert_eq!(reference.name, "Development");
    }

    #[test]
    fn test_time_entry_deserialize() {
        let json = r#"{
            "id": 636709355,
            "spent_date": "2026-06-08",
            "hours": 2.5,
            "notes": "Refactored auth module",
            "is_running": false,
            "timer_started_at": null,
            "started_time": "9:00am",
            "ended_time": "11:30am",
            "project": {"id": 14307913, "name": "Platform Redesign"},
            "task": {"id": 8083365, "name": "Development"},
            "user": {"id": 1782959, "name": "Kim Allen"},
            "client": {"id": 5735774, "name": "HealthCorp"},
            "is_billed": false,
            "billable": true,
            "billable_rate": null,
            "cost_rate": null,
            "created_at": "2026-06-08T09:00:00Z",
            "updated_at": "2026-06-08T11:30:00Z"
        }"#;
        let entry: TimeEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.id, 636709355);
        assert_eq!(entry.hours, Some(2.5));
        assert!(!entry.is_running);
        assert_eq!(entry.project.name, "Platform Redesign");
        assert_eq!(entry.task.name, "Development");
        assert!(entry.client.is_some());
    }

    #[test]
    fn test_time_entry_running_deserialize() {
        let json = r#"{
            "id": 636709356,
            "hours": null,
            "is_running": true,
            "timer_started_at": "2026-06-08T14:00:00Z",
            "project": {"id": 14307913, "name": "Platform Redesign"},
            "task": {"id": 8083365, "name": "Development"},
            "user": {"id": 1782959, "name": "Kim Allen"},
            "client": null,
            "is_billed": false,
            "billable": true,
            "billable_rate": null,
            "cost_rate": null,
            "created_at": null,
            "updated_at": null,
            "spent_date": "2026-06-08",
            "notes": null,
            "started_time": null,
            "ended_time": null
        }"#;
        let entry: TimeEntry = serde_json::from_str(json).unwrap();
        assert!(entry.is_running);
        assert!(entry.hours.is_none());
        assert!(entry.timer_started_at.is_some());
    }

    #[test]
    fn test_create_time_entry_with_hours() {
        let entry = CreateTimeEntry {
            project_id: 14307913,
            task_id: 8083365,
            spent_date: None,
            hours: Some(2.5),
            notes: Some("Worked on feature X".into()),
            started_time: Some("9:00am".into()),
            ended_time: Some("11:30am".into()),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("project_id"));
        assert!(json.contains("2.5"));
        assert!(json.contains("feature X"));
    }

    #[test]
    fn test_create_time_entry_running() {
        let entry = CreateTimeEntry {
            project_id: 14307913,
            task_id: 8083365,
            spent_date: None,
            hours: None,
            notes: None,
            started_time: None,
            ended_time: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        // Running timer: hours, started_time, ended_time should be absent
        assert!(!json.contains("hours"));
        assert!(!json.contains("started_time"));
        assert!(!json.contains("ended_time"));
    }

    #[test]
    fn test_update_time_entry_defaults() {
        let entry = UpdateTimeEntry::default();
        let json = serde_json::to_string(&entry).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_project_assignment_deserialize() {
        let json = r#"{
            "id": 125068553,
            "project": {"id": 14307913, "name": "Platform Redesign"},
            "client": {"id": 5735774, "name": "HealthCorp"},
            "task_assignments": [
                {"id": 155502709, "task": {"id": 8083365, "name": "Development"}},
                {"id": 155502710, "task": {"id": 8083366, "name": "Code Review"}}
            ],
            "is_active": true
        }"#;
        let assignment: ProjectAssignment = serde_json::from_str(json).unwrap();
        assert_eq!(assignment.project.name, "Platform Redesign");
        assert_eq!(assignment.task_assignments.len(), 2);
        assert_eq!(assignment.task_assignments[0].task.name, "Development");
    }

    #[test]
    fn test_serialize_roundtrip_reference() {
        let original = Reference {
            id: 42,
            name: "Test".into(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let parsed: Reference = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_user_deserialize() {
        let json = r#"{
            "id": 1782959,
            "first_name": "Kim",
            "last_name": "Allen",
            "email": "kim@example.com",
            "is_active": true,
            "created_at": null,
            "updated_at": null
        }"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.first_name, "Kim");
        assert_eq!(user.email, "kim@example.com");
    }

    #[test]
    fn test_user_deserialize_with_new_fields() {
        let json = r#"{
            "id": 1,
            "first_name": "Test",
            "last_name": "User",
            "email": "test@example.com",
            "is_active": true,
            "created_at": null,
            "updated_at": null,
            "timezone": "America/New_York",
            "weekly_capacity": 144000,
            "is_admin": true,
            "is_project_manager": false
        }"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.timezone, Some("America/New_York".into()));
        assert_eq!(user.weekly_capacity, Some(144000));
        assert_eq!(user.is_admin, Some(true));
        assert_eq!(user.is_project_manager, Some(false));
    }

    #[test]
    fn test_user_deserialize_without_new_fields() {
        let json = r#"{
            "id": 1,
            "first_name": "Test",
            "last_name": "User",
            "email": "test@example.com",
            "is_active": true,
            "created_at": null,
            "updated_at": null
        }"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.timezone, None);
        assert_eq!(user.weekly_capacity, None);
        assert_eq!(user.is_admin, None);
        assert_eq!(user.is_project_manager, None);
    }

    #[test]
    fn test_client_deserialize() {
        let json = r#"{
            "id": 1,
            "name": "Test Client",
            "is_active": true,
            "address": "123 Main St",
            "currency": "USD",
            "created_at": null,
            "updated_at": null
        }"#;
        let client: Client = serde_json::from_str(json).unwrap();
        assert_eq!(client.name, "Test Client");
        assert_eq!(client.currency, Some("USD".into()));
    }

    #[test]
    fn test_company_deserialize() {
        let json = r#"{"name": "Acme Corp"}"#;
        let company: Company = serde_json::from_str(json).unwrap();
        assert_eq!(company.name, "Acme Corp");
    }

    #[test]
    fn test_project_deserialize() {
        let json = r#"{
            "id": 1,
            "name": "Test Project",
            "client": null,
            "is_active": true,
            "code": "PRJ-001",
            "notes": null,
            "starts_on": null,
            "ends_on": null,
            "created_at": null,
            "updated_at": null
        }"#;
        let project: Project = serde_json::from_str(json).unwrap();
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.code, Some("PRJ-001".into()));
    }

    #[test]
    fn test_task_deserialize() {
        let json = r#"{
            "id": 1,
            "name": "Development",
            "billable_by_default": true,
            "default_hourly_rate": null,
            "is_default": false,
            "is_active": true,
            "created_at": null,
            "updated_at": null
        }"#;
        let task: Task = serde_json::from_str(json).unwrap();
        assert_eq!(task.name, "Development");
        assert!(task.billable_by_default);
    }
}
