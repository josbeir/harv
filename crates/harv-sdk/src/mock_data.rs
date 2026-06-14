//! Shared mock data factories used by both tests and the mock-mode dev server.
//!
//! This module is always compiled — it has no dependency on `wiremock`.
//! Tests and the mock server both import from here, avoiding duplication.

use std::collections::HashMap;

use crate::config::HarvConfig;
use serde_json::json;

/// Creates a minimal valid config suitable for testing or mock mode.
/// Token and account_id are dummy values accepted by a mock server.
pub fn test_config() -> HarvConfig {
    HarvConfig {
        access_token: "t".into(),
        account_id: "1".into(),
        cache_ttl_hours: 0, // never cache mock data
        last_project_id: None,
        last_task_id: None,
        locale: None,
        aliases: HashMap::new(),
    }
}

/// Creates a config with pre-set last-used project/task IDs.
pub fn config_with_last_used(pid: u64, tid: u64) -> HarvConfig {
    HarvConfig {
        last_project_id: Some(pid),
        last_task_id: Some(tid),
        ..test_config()
    }
}

// ---------------------------------------------------------------------------
// Pagination wrappers
// ---------------------------------------------------------------------------

pub fn paginated(key: &str, items: Vec<serde_json::Value>) -> serde_json::Value {
    let count = items.len();
    json!({
        key: items,
        "total_pages": 1,
        "page": 1,
        "total_entries": count,
        "per_page": 100,
    })
}

// ---------------------------------------------------------------------------
// Users
// ---------------------------------------------------------------------------

pub fn user_json() -> serde_json::Value {
    json!({
        "id": 1,
        "first_name": "Test",
        "last_name": "User",
        "email": "test@harv.dev",
        "is_active": true,
        "created_at": null,
        "updated_at": null,
        "access_roles": ["member"]
    })
}

// ---------------------------------------------------------------------------
// Company
// ---------------------------------------------------------------------------

pub fn company_json() -> serde_json::Value {
    json!({
        "name": "Acme Corp",
        "base_uri": "https://acme.harvestapp.com",
        "full_domain": "acme.harvestapp.com",
        "is_active": true,
        "week_start_day": "Monday",
        "wants_timestamp_timers": false,
        "time_format": "hours_minutes",
        "clock": "12h",
        "plan_type": "simple-v4"
    })
}

// ---------------------------------------------------------------------------
// Clients
// ---------------------------------------------------------------------------

pub fn client_a_json() -> serde_json::Value {
    json!({"id": 1, "name": "Acme Corp"})
}

pub fn client_b_json() -> serde_json::Value {
    json!({"id": 2, "name": "Initech"})
}

// ---------------------------------------------------------------------------
// Projects
// ---------------------------------------------------------------------------

pub fn project_alpha_json() -> serde_json::Value {
    json!({
        "id": 100,
        "name": "Website Redesign",
        "code": "WEB",
        "client": client_a_json(),
        "is_active": true,
        "notes": null,
        "starts_on": null,
        "ends_on": null,
        "created_at": null,
        "updated_at": null
    })
}

pub fn project_beta_json() -> serde_json::Value {
    json!({
        "id": 101,
        "name": "Mobile App",
        "code": "MOB",
        "client": client_b_json(),
        "is_active": true,
        "notes": null,
        "starts_on": null,
        "ends_on": null,
        "created_at": null,
        "updated_at": null
    })
}

pub fn project_gamma_json() -> serde_json::Value {
    json!({
        "id": 102,
        "name": "Internal Tools",
        "code": null,
        "client": null,
        "is_active": true,
        "notes": null,
        "starts_on": null,
        "ends_on": null,
        "created_at": null,
        "updated_at": null
    })
}

pub fn all_projects_json() -> Vec<serde_json::Value> {
    vec![
        project_alpha_json(),
        project_beta_json(),
        project_gamma_json(),
    ]
}

// ---------------------------------------------------------------------------
// Tasks
// ---------------------------------------------------------------------------

fn task_dev_json() -> serde_json::Value {
    json!({"id": 200, "name": "Development"})
}

fn task_design_json() -> serde_json::Value {
    json!({"id": 201, "name": "Design"})
}

fn task_meeting_json() -> serde_json::Value {
    json!({"id": 202, "name": "Meetings"})
}

fn task_review_json() -> serde_json::Value {
    json!({"id": 203, "name": "Code Review"})
}

// ---------------------------------------------------------------------------
// Project Assignments
// ---------------------------------------------------------------------------

pub fn project_assignments_json() -> serde_json::Value {
    paginated(
        "project_assignments",
        vec![
            json!({
                "id": 1,
                "project": {"id": 100, "name": "Website Redesign"},
                "client": client_a_json(),
                "task_assignments": [
                    {"id": 10, "task": task_dev_json()},
                    {"id": 11, "task": task_design_json()},
                    {"id": 12, "task": task_review_json()},
                ],
                "is_active": true
            }),
            json!({
                "id": 2,
                "project": {"id": 101, "name": "Mobile App"},
                "client": client_b_json(),
                "task_assignments": [
                    {"id": 20, "task": task_dev_json()},
                    {"id": 21, "task": task_design_json()},
                ],
                "is_active": true
            }),
            json!({
                "id": 3,
                "project": {"id": 102, "name": "Internal Tools"},
                "client": null,
                "task_assignments": [
                    {"id": 30, "task": task_dev_json()},
                    {"id": 31, "task": task_meeting_json()},
                ],
                "is_active": true
            }),
        ],
    )
}

// ---------------------------------------------------------------------------
// Time Entries
// ---------------------------------------------------------------------------

/// A running timer entry.
pub fn running_timer_json() -> serde_json::Value {
    json!({
        "id": 5001,
        "spent_date": null,
        "hours": null,
        "notes": "Working on the login screen",
        "is_running": true,
        "timer_started_at": "2026-06-14T09:30:00Z",
        "started_time": null,
        "ended_time": null,
        "project": {"id": 100, "name": "Website Redesign"},
        "task": {"id": 200, "name": "Development"},
        "user": {"id": 1, "name": "Test User"},
        "client": client_a_json(),
        "is_billed": false,
        "billable": true,
        "billable_rate": null,
        "cost_rate": null,
        "created_at": null,
        "updated_at": null
    })
}

/// Stopped entries for today.
pub fn today_entries_json() -> Vec<serde_json::Value> {
    vec![
        json!({
            "id": 5002,
            "spent_date": "2026-06-14",
            "hours": 1.5,
            "notes": "Fixed navigation bug",
            "is_running": false,
            "timer_started_at": null,
            "started_time": "08:00",
            "ended_time": "09:30",
            "project": {"id": 100, "name": "Website Redesign"},
            "task": {"id": 200, "name": "Development"},
            "user": {"id": 1, "name": "Test User"},
            "client": client_a_json(),
            "is_billed": false,
            "billable": true,
            "billable_rate": null,
            "cost_rate": null,
            "created_at": null,
            "updated_at": null
        }),
        json!({
            "id": 5003,
            "spent_date": "2026-06-14",
            "hours": 2.0,
            "notes": "Designed new dashboard layout",
            "is_running": false,
            "timer_started_at": null,
            "started_time": "10:00",
            "ended_time": "12:00",
            "project": {"id": 100, "name": "Website Redesign"},
            "task": {"id": 201, "name": "Design"},
            "user": {"id": 1, "name": "Test User"},
            "client": client_a_json(),
            "is_billed": false,
            "billable": true,
            "billable_rate": null,
            "cost_rate": null,
            "created_at": null,
            "updated_at": null
        }),
    ]
}

/// Minimal entry for tests that only need 1 project + 1-2 tasks.
pub fn project_assignments_minimal_json() -> serde_json::Value {
    paginated(
        "project_assignments",
        vec![json!({
            "id": 1,
            "project": {"id": 100, "name": "Test Project"},
            "client": client_a_json(),
            "task_assignments": [
                {"id": 10, "task": {"id": 200, "name": "Development"}},
                {"id": 11, "task": {"id": 201, "name": "Design"}},
            ],
            "is_active": true
        })],
    )
}

pub fn project_minimal_json() -> serde_json::Value {
    json!({
        "id": 100,
        "name": "Test Project",
        "code": "TEST",
        "client": client_a_json(),
        "is_active": true,
        "notes": null,
        "starts_on": null,
        "ends_on": null,
        "created_at": null,
        "updated_at": null
    })
}
