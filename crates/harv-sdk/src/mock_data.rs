//! Shared mock data factories used by both tests and the mock-mode dev server.
//!
//! This module is always compiled — it has no dependency on `wiremock`.
//! Tests and the mock server both import from here, avoiding duplication.

use std::collections::HashMap;

use crate::config::HarvConfig;
use serde_json::json;

// ── Config helpers ──────────────────────────────────────────

pub fn test_config() -> HarvConfig {
    HarvConfig {
        access_token: "t".into(),
        account_id: "1".into(),
        cache_ttl_hours: 0,
        last_project_id: None,
        last_task_id: None,
        locale: None,
        aliases: HashMap::new(),
    }
}

pub fn config_with_last_used(pid: u64, tid: u64) -> HarvConfig {
    HarvConfig {
        last_project_id: Some(pid),
        last_task_id: Some(tid),
        ..test_config()
    }
}

// ── Pagination ──────────────────────────────────────────────

pub fn paginated(key: &str, items: Vec<serde_json::Value>) -> serde_json::Value {
    let count = items.len();
    json!({
        key: items,
        "total_pages": 1, "page": 1,
        "total_entries": count, "per_page": 100,
    })
}

// ── User / Company ──────────────────────────────────────────

pub fn user_json() -> serde_json::Value {
    json!({
        "id": 1, "first_name": "Marcus", "last_name": "Aurelius",
        "email": "marcus@rome.emp", "is_active": true,
        "created_at": null, "updated_at": null,
        "access_roles": ["member"]
    })
}

pub fn company_json() -> serde_json::Value {
    json!({
        "name": "Senatus Populusque Romanus",
        "base_uri": "https://spqr.harvestapp.com",
        "full_domain": "spqr.harvestapp.com", "is_active": true,
        "week_start_day": "Monday", "wants_timestamp_timers": false,
        "time_format": "hours_minutes", "clock": "12h", "plan_type": "simple-v4"
    })
}

// ── Clients ─────────────────────────────────────────────────

pub fn client_a_json() -> serde_json::Value {
    json!({"id": 1, "name": "Legio X Fretensis"})
}
pub fn client_b_json() -> serde_json::Value {
    json!({"id": 2, "name": "Collegium Fabrorum"})
}
pub fn client_c_json() -> serde_json::Value {
    json!({"id": 3, "name": "Templum Iovis"})
}
pub fn client_d_json() -> serde_json::Value {
    json!({"id": 4, "name": "Via Appia Consortium"})
}

// ── Projects (7) ────────────────────────────────────────────

fn project(
    id: u64,
    name: &str,
    code: Option<&str>,
    client: serde_json::Value,
) -> serde_json::Value {
    json!({
        "id": id, "name": name, "code": code,
        "client": client, "is_active": true,
        "notes": null, "starts_on": null, "ends_on": null,
        "created_at": null, "updated_at": null
    })
}

pub fn project_alpha_json() -> serde_json::Value {
    project(100, "Aqueduct Restoration", Some("AQV"), client_a_json())
}
pub fn project_beta_json() -> serde_json::Value {
    project(101, "Colosseum Upgrades", Some("COL"), client_b_json())
}
pub fn project_gamma_json() -> serde_json::Value {
    project(102, "Senate Records", None, json!(null))
}
pub fn project_delta_json() -> serde_json::Value {
    project(103, "Triumphal Arch", Some("ARC"), client_c_json())
}
pub fn project_epsilon_json() -> serde_json::Value {
    project(104, "Gladius Forge", Some("GLD"), client_a_json())
}
pub fn project_zeta_json() -> serde_json::Value {
    project(105, "Campus Martius", Some("CMP"), client_d_json())
}
pub fn project_eta_json() -> serde_json::Value {
    project(106, "Castra Praetoria", None, client_b_json())
}

pub fn all_projects_json() -> Vec<serde_json::Value> {
    vec![
        project_alpha_json(),
        project_beta_json(),
        project_gamma_json(),
        project_delta_json(),
        project_epsilon_json(),
        project_zeta_json(),
        project_eta_json(),
    ]
}

// ── Tasks (8) ───────────────────────────────────────────────

fn task(id: u64, name: &str) -> serde_json::Value {
    json!({"id": id, "name": name})
}
fn task_full(id: u64, name: &str) -> serde_json::Value {
    json!({
        "id": id, "name": name, "billable_by_default": true,
        "default_hourly_rate": null, "is_default": false,
        "is_active": true, "created_at": null, "updated_at": null
    })
}

fn task_dev() -> serde_json::Value {
    task(200, "Stone Cutting")
}
fn task_design() -> serde_json::Value {
    task(201, "Mosaic Design")
}
fn task_meetings() -> serde_json::Value {
    task(202, "Senate Hearings")
}
fn task_review() -> serde_json::Value {
    task(203, "Inscription Review")
}
fn task_testing() -> serde_json::Value {
    task(204, "Siege Testing")
}
fn task_deploy() -> serde_json::Value {
    task(205, "Triumphal Parade")
}
fn task_docs() -> serde_json::Value {
    task(206, "Scroll Writing")
}
fn task_planning() -> serde_json::Value {
    task(207, "Augury & Omens")
}

pub fn all_tasks_json() -> Vec<serde_json::Value> {
    vec![
        task_full(200, "Stone Cutting"),
        task_full(201, "Mosaic Design"),
        task_full(202, "Senate Hearings"),
        task_full(203, "Inscription Review"),
        task_full(204, "Siege Testing"),
        task_full(205, "Triumphal Parade"),
        task_full(206, "Scroll Writing"),
        task_full(207, "Augury & Omens"),
    ]
}

// ── Project Assignments (7 projects) ────────────────────────

pub fn project_assignments_json() -> serde_json::Value {
    paginated(
        "project_assignments",
        vec![
            json!({"id": 1, "project": {"id": 100, "name": "Aqueduct Restoration"},
            "client": client_a_json(), "is_active": true,
            "task_assignments": [
                {"id": 10, "task": task_dev()},
                {"id": 11, "task": task_design()},
                {"id": 12, "task": task_review()},
                {"id": 13, "task": task_testing()},
            ]}),
            json!({"id": 2, "project": {"id": 101, "name": "Colosseum Upgrades"},
            "client": client_b_json(), "is_active": true,
            "task_assignments": [
                {"id": 20, "task": task_dev()},
                {"id": 21, "task": task_design()},
                {"id": 22, "task": task_testing()},
            ]}),
            json!({"id": 3, "project": {"id": 102, "name": "Senate Records"},
            "client": null, "is_active": true,
            "task_assignments": [
                {"id": 30, "task": task_dev()},
                {"id": 31, "task": task_meetings()},
            ]}),
            json!({"id": 4, "project": {"id": 103, "name": "Triumphal Arch"},
            "client": client_c_json(), "is_active": true,
            "task_assignments": [
                {"id": 40, "task": task_dev()},
                {"id": 41, "task": task_review()},
                {"id": 42, "task": task_testing()},
                {"id": 43, "task": task_docs()},
            ]}),
            json!({"id": 5, "project": {"id": 104, "name": "Gladius Forge"},
            "client": client_a_json(), "is_active": true,
            "task_assignments": [
                {"id": 50, "task": task_dev()},
                {"id": 51, "task": task_deploy()},
                {"id": 52, "task": task_docs()},
            ]}),
            json!({"id": 6, "project": {"id": 105, "name": "Campus Martius"},
            "client": client_d_json(), "is_active": true,
            "task_assignments": [
                {"id": 60, "task": task_dev()},
                {"id": 61, "task": task_design()},
                {"id": 62, "task": task_planning()},
            ]}),
            json!({"id": 7, "project": {"id": 106, "name": "Castra Praetoria"},
            "client": client_b_json(), "is_active": true,
            "task_assignments": [
                {"id": 70, "task": task_meetings()},
                {"id": 71, "task": task_docs()},
            ]}),
        ],
    )
}

// ── Time Entries ────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn time_entry(
    id: u64,
    spent_date: &str,
    hours: f64,
    notes: &str,
    project_id: u64,
    project_name: &str,
    task_id: u64,
    task_name: &str,
    client: serde_json::Value,
    start: &str,
    end: &str,
) -> serde_json::Value {
    json!({
        "id": id, "spent_date": spent_date, "hours": hours,
        "notes": notes, "is_running": false, "timer_started_at": null,
        "started_time": start, "ended_time": end,
        "project": {"id": project_id, "name": project_name},
        "task": {"id": task_id, "name": task_name},
        "user": {"id": 1, "name": "Marcus Aurelius"},
        "client": client,
        "is_billed": false, "billable": true,
        "billable_rate": null, "cost_rate": null,
        "created_at": null, "updated_at": null
    })
}

pub fn running_timer_json() -> serde_json::Value {
    json!({
        "id": 5001, "spent_date": null, "hours": null,
        "notes": "Carving inscription for Arch of Titus",
        "is_running": true, "timer_started_at": "2026-06-14T09:30:00Z",
        "started_time": null, "ended_time": null,
        "project": {"id": 100, "name": "Aqueduct Restoration"},
        "task": {"id": 200, "name": "Stone Cutting"},
        "user": {"id": 1, "name": "Marcus Aurelius"},
        "client": client_a_json(),
        "is_billed": false, "billable": true,
        "billable_rate": null, "cost_rate": null,
        "created_at": null, "updated_at": null
    })
}

pub fn today_entries_json() -> Vec<serde_json::Value> {
    vec![
        time_entry(
            5002,
            "2026-06-14",
            1.5,
            "Repaired cracked aqueduct arch #7",
            100,
            "Aqueduct Restoration",
            200,
            "Stone Cutting",
            client_a_json(),
            "08:00",
            "09:30",
        ),
        time_entry(
            5003,
            "2026-06-14",
            2.0,
            "Laid mosaic pattern in eastern gallery",
            100,
            "Aqueduct Restoration",
            201,
            "Mosaic Design",
            client_a_json(),
            "10:00",
            "12:00",
        ),
        time_entry(
            5004,
            "2026-06-14",
            1.0,
            "Inspected new iron gate hinges",
            101,
            "Colosseum Upgrades",
            203,
            "Inscription Review",
            client_b_json(),
            "13:00",
            "14:00",
        ),
        time_entry(
            5005,
            "2026-06-14",
            2.5,
            "Drafted pillar dimensions for archway",
            103,
            "Triumphal Arch",
            200,
            "Stone Cutting",
            client_c_json(),
            "14:00",
            "16:30",
        ),
        time_entry(
            5006,
            "2026-06-14",
            0.5,
            "Morning briefing with centurions",
            104,
            "Gladius Forge",
            202,
            "Senate Hearings",
            client_a_json(),
            "09:00",
            "09:30",
        ),
        time_entry(
            5007,
            "2026-06-14",
            1.0,
            "Surveyed drainage on Campus Martius",
            105,
            "Campus Martius",
            201,
            "Mosaic Design",
            client_d_json(),
            "16:30",
            "17:30",
        ),
        time_entry(
            5008,
            "2026-06-14",
            2.0,
            "Authored scroll on arch construction techniques",
            103,
            "Triumphal Arch",
            206,
            "Scroll Writing",
            client_c_json(),
            "11:00",
            "13:00",
        ),
        time_entry(
            5009,
            "2026-06-14",
            1.5,
            "Siege-tested new catapult torsion springs",
            101,
            "Colosseum Upgrades",
            204,
            "Siege Testing",
            client_b_json(),
            "15:00",
            "16:30",
        ),
    ]
}

// ── Minimal variants for tests ──────────────────────────────

pub fn project_assignments_minimal_json() -> serde_json::Value {
    paginated(
        "project_assignments",
        vec![json!({
            "id": 1, "project": {"id": 100, "name": "Test Project"},
            "client": client_a_json(), "is_active": true,
            "task_assignments": [
                {"id": 10, "task": {"id": 200, "name": "Development"}},
                {"id": 11, "task": {"id": 201, "name": "Design"}},
            ],
        })],
    )
}

pub fn project_minimal_json() -> serde_json::Value {
    json!({
        "id": 100, "name": "Test Project", "code": "TEST",
        "client": client_a_json(), "is_active": true,
        "notes": null, "starts_on": null, "ends_on": null,
        "created_at": null, "updated_at": null
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_config() {
        let cfg = test_config();
        assert_eq!(cfg.access_token, "t");
        assert_eq!(cfg.account_id, "1");
        assert_eq!(cfg.cache_ttl_hours, 0);
        assert!(cfg.last_project_id.is_none());
        assert!(cfg.last_task_id.is_none());
        assert!(cfg.aliases.is_empty());
    }

    #[test]
    fn test_config_with_last_used() {
        let cfg = config_with_last_used(42, 7);
        assert_eq!(cfg.last_project_id, Some(42));
        assert_eq!(cfg.last_task_id, Some(7));
    }

    #[test]
    fn test_paginated() {
        let items = vec![json!({"id": 1}), json!({"id": 2}), json!({"id": 3})];
        let result = paginated("items", items);
        assert_eq!(result["page"], json!(1));
        assert_eq!(result["total_pages"], json!(1));
        assert_eq!(result["total_entries"], json!(3));
        assert_eq!(result["per_page"], json!(100));
        assert_eq!(result["items"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn test_user_json() {
        let user = user_json();
        assert_eq!(user["id"], json!(1));
        assert_eq!(user["first_name"], json!("Marcus"));
        assert_eq!(user["last_name"], json!("Aurelius"));
        assert_eq!(user["email"], json!("marcus@rome.emp"));
    }

    #[test]
    fn test_company_json() {
        let company = company_json();
        assert_eq!(company["name"], json!("Senatus Populusque Romanus"));
        assert_eq!(company["base_uri"], json!("https://spqr.harvestapp.com"));
    }

    #[test]
    fn test_client_a_json() {
        let c = client_a_json();
        assert_eq!(c["id"], json!(1));
        assert_eq!(c["name"], json!("Legio X Fretensis"));
    }

    #[test]
    fn test_client_b_json() {
        let c = client_b_json();
        assert_eq!(c["name"], json!("Collegium Fabrorum"));
    }

    #[test]
    fn test_client_c_json() {
        let c = client_c_json();
        assert_eq!(c["name"], json!("Templum Iovis"));
    }

    #[test]
    fn test_client_d_json() {
        let c = client_d_json();
        assert_eq!(c["name"], json!("Via Appia Consortium"));
    }

    #[test]
    fn test_project_alpha_json() {
        let p = project_alpha_json();
        assert_eq!(p["id"], json!(100));
        assert_eq!(p["name"], json!("Aqueduct Restoration"));
        assert_eq!(p["code"], json!("AQV"));
    }

    #[test]
    fn test_project_beta_json() {
        let p = project_beta_json();
        assert_eq!(p["name"], json!("Colosseum Upgrades"));
        assert_eq!(p["code"], json!("COL"));
    }

    #[test]
    fn test_project_gamma_json() {
        let p = project_gamma_json();
        assert_eq!(p["name"], json!("Senate Records"));
        assert_eq!(p["code"], json!(null));
    }

    #[test]
    fn test_project_delta_json() {
        let p = project_delta_json();
        assert_eq!(p["name"], json!("Triumphal Arch"));
    }

    #[test]
    fn test_project_epsilon_json() {
        let p = project_epsilon_json();
        assert_eq!(p["name"], json!("Gladius Forge"));
    }

    #[test]
    fn test_project_zeta_json() {
        let p = project_zeta_json();
        assert_eq!(p["name"], json!("Campus Martius"));
    }

    #[test]
    fn test_project_eta_json() {
        let p = project_eta_json();
        assert_eq!(p["name"], json!("Castra Praetoria"));
    }

    #[test]
    fn test_all_projects_json() {
        let projects = all_projects_json();
        assert_eq!(projects.len(), 7);
    }

    #[test]
    fn test_all_tasks_json() {
        let tasks = all_tasks_json();
        assert_eq!(tasks.len(), 8);
        assert_eq!(tasks[0]["name"], json!("Stone Cutting"));
        assert!(tasks[0]["billable_by_default"] == json!(true));
    }

    #[test]
    fn test_project_assignments_json() {
        let result = project_assignments_json();
        let pas = result["project_assignments"].as_array().unwrap();
        assert_eq!(pas.len(), 7);
    }

    #[test]
    fn test_running_timer_json() {
        let t = running_timer_json();
        assert_eq!(t["id"], json!(5001));
        assert_eq!(t["is_running"], json!(true));
        assert!(t["timer_started_at"].is_string());
    }

    #[test]
    fn test_today_entries_json() {
        let entries = today_entries_json();
        assert_eq!(entries.len(), 8);
        assert_eq!(entries[0]["id"], json!(5002));
    }

    #[test]
    fn test_project_assignments_minimal_json() {
        let result = project_assignments_minimal_json();
        let pas = result["project_assignments"].as_array().unwrap();
        assert_eq!(pas.len(), 1);
        assert_eq!(pas[0]["project"]["name"], json!("Test Project"));
    }

    #[test]
    fn test_project_minimal_json() {
        let p = project_minimal_json();
        assert_eq!(p["id"], json!(100));
        assert_eq!(p["code"], json!("TEST"));
    }
}
