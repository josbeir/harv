//! In-process mock Harvest API server used when `HARV_MOCK=1` is set.
//!
//! Only compiled when the `mock-mode` feature is enabled.
//! Uses `wiremock` to serve realistic but hardcoded responses for all
//! API endpoints needed by the CLI and TUI.
//!
//! ## Configuration
//!
//! | Env var | Default | Description |
//! |---------|---------|-------------|
//! | `HARV_MOCK_DELAY_MS` | `0` | Simulated API latency in milliseconds |

use std::time::Duration;

use crate::mock_data;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn delay() -> Duration {
    Duration::from_millis(
        std::env::var("HARV_MOCK_DELAY_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
    )
}

fn json_response(body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200)
        .set_delay(delay())
        .set_body_json(body)
        .insert_header("Content-Type", "application/json")
}

/// Start a mock Harvest API server and return its base URL.
///
/// The server responds to all endpoints used by harv with realistic
/// data: multiple projects, task assignments, users, company info,
/// time entries (including a running timer), and CRUD operations.
pub async fn start() -> String {
    let server = MockServer::start().await;

    // ── Users ──────────────────────────────────────────────────
    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(mock_data::user_json()))
        .mount(&server)
        .await;

    // ── Company ────────────────────────────────────────────────
    Mock::given(method("GET"))
        .and(path("/company"))
        .respond_with(json_response(mock_data::company_json()))
        .mount(&server)
        .await;

    // ── Project assignments ────────────────────────────────────
    Mock::given(method("GET"))
        .and(path("/users/me/project_assignments"))
        .respond_with(json_response(mock_data::project_assignments_json()))
        .mount(&server)
        .await;

    // ── Projects ───────────────────────────────────────────────
    Mock::given(method("GET"))
        .and(path("/projects"))
        .respond_with(json_response(mock_data::paginated(
            "projects",
            mock_data::all_projects_json(),
        )))
        .mount(&server)
        .await;

    // GET /projects/{id}
    for (id, project_fn) in [
        (100, mock_data::project_alpha_json as fn() -> _),
        (101, mock_data::project_beta_json),
        (102, mock_data::project_gamma_json),
        (103, mock_data::project_delta_json),
        (104, mock_data::project_epsilon_json),
        (105, mock_data::project_zeta_json),
        (106, mock_data::project_eta_json),
    ] {
        Mock::given(method("GET"))
            .and(path(format!("/projects/{}", id)))
            .respond_with(json_response(project_fn()))
            .mount(&server)
            .await;
    }

    // ── Task assignments (per project) ─────────────────────────
    let ta = |tasks: &[(u64, u64, &str)]| -> Vec<serde_json::Value> {
        tasks
            .iter()
            .map(|(aid, tid, name)| {
                serde_json::json!({"id": aid, "task": {"id": tid, "name": name}})
            })
            .collect()
    };
    Mock::given(method("GET"))
        .and(path("/projects/100/task_assignments"))
        .respond_with(json_response(mock_data::paginated(
            "task_assignments",
            ta(&[
                (10, 200, "Stone Cutting"),
                (11, 201, "Mosaic Design"),
                (12, 203, "Inscription Review"),
                (13, 204, "Siege Testing"),
            ]),
        )))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/projects/101/task_assignments"))
        .respond_with(json_response(mock_data::paginated(
            "task_assignments",
            ta(&[
                (20, 200, "Stone Cutting"),
                (21, 201, "Mosaic Design"),
                (22, 204, "Siege Testing"),
            ]),
        )))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/projects/102/task_assignments"))
        .respond_with(json_response(mock_data::paginated(
            "task_assignments",
            ta(&[(30, 200, "Stone Cutting"), (31, 202, "Senate Hearings")]),
        )))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/projects/103/task_assignments"))
        .respond_with(json_response(mock_data::paginated(
            "task_assignments",
            ta(&[
                (40, 200, "Stone Cutting"),
                (41, 203, "Inscription Review"),
                (42, 204, "Siege Testing"),
                (43, 206, "Scroll Writing"),
            ]),
        )))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/projects/104/task_assignments"))
        .respond_with(json_response(mock_data::paginated(
            "task_assignments",
            ta(&[
                (50, 200, "Stone Cutting"),
                (51, 205, "Triumphal Parade"),
                (52, 206, "Scroll Writing"),
            ]),
        )))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/projects/105/task_assignments"))
        .respond_with(json_response(mock_data::paginated(
            "task_assignments",
            ta(&[
                (60, 200, "Stone Cutting"),
                (61, 201, "Mosaic Design"),
                (62, 207, "Augury & Omens"),
            ]),
        )))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/projects/106/task_assignments"))
        .respond_with(json_response(mock_data::paginated(
            "task_assignments",
            ta(&[(70, 202, "Senate Hearings"), (71, 206, "Scroll Writing")]),
        )))
        .mount(&server)
        .await;

    // ── Tasks ──────────────────────────────────────────────────
    Mock::given(method("GET"))
        .and(path("/tasks"))
        .respond_with(json_response(mock_data::paginated(
            "tasks",
            mock_data::all_tasks_json(),
        )))
        .mount(&server)
        .await;

    // ── Time entries ───────────────────────────────────────────
    // List today's entries + running timer
    Mock::given(method("GET"))
        .and(path("/time_entries"))
        .respond_with(json_response(mock_data::paginated("time_entries", {
            let mut entries = mock_data::today_entries_json();
            entries.push(mock_data::running_timer_json());
            entries
        })))
        .mount(&server)
        .await;

    // Running time entries (for the timer check)
    Mock::given(method("GET"))
        .and(path("/time_entries"))
        .respond_with(json_response(mock_data::paginated(
            "time_entries",
            vec![mock_data::running_timer_json()],
        )))
        .mount(&server)
        .await;

    // Create time entry
    let next_id = std::sync::atomic::AtomicU64::new(6000);
    Mock::given(method("POST"))
        .and(path("/time_entries"))
        .respond_with(move |_req: &wiremock::Request| {
            let id = next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            ResponseTemplate::new(200)
                .set_delay(delay())
                .set_body_json(serde_json::json!({
                    "id": id,
                    "spent_date": "2026-06-14",
                    "hours": 1.0,
                    "notes": null,
                    "is_running": false,
                    "timer_started_at": null,
                    "started_time": null,
                    "ended_time": null,
                    "project": {"id": 100, "name": "Aqueduct Restoration"},
                    "task": {"id": 200, "name": "Stone Cutting"},
                    "user": {"id": 1, "name": "Marcus Aurelius"},
                    "client": mock_data::client_a_json(),
                    "is_billed": false,
                    "billable": true,
                    "billable_rate": null,
                    "cost_rate": null,
                    "created_at": null,
                    "updated_at": null
                }))
                .insert_header("Content-Type", "application/json")
        })
        .mount(&server)
        .await;

    // Update time entry
    Mock::given(method("PATCH"))
        .and(path("/time_entries/5002"))
        .respond_with(json_response(serde_json::json!({
            "id": 5002,
            "spent_date": "2026-06-14",
            "hours": 1.5,
            "notes": "Updated notes",
            "is_running": false,
            "timer_started_at": null,
            "project": {"id": 100, "name": "Aqueduct Restoration"},
            "task": {"id": 200, "name": "Stone Cutting"},
            "user": {"id": 1, "name": "Marcus Aurelius"},
            "client": mock_data::client_a_json(),
            "is_billed": false,
            "billable": true,
            "billable_rate": null,
            "cost_rate": null,
            "created_at": null,
            "updated_at": null
        })))
        .mount(&server)
        .await;

    // Delete time entry
    Mock::given(method("DELETE"))
        .and(path("/time_entries/5002"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    // Stop running timer
    Mock::given(method("PATCH"))
        .and(path("/time_entries/5001/stop"))
        .respond_with(json_response(serde_json::json!({
            "id": 5001,
            "is_running": false,
            "hours": 2.0,
            "project": {"id": 100, "name": "Aqueduct Restoration"},
            "task": {"id": 200, "name": "Stone Cutting"},
            "user": {"id": 1, "name": "Marcus Aurelius"},
            "client": mock_data::client_a_json(),
            "is_billed": false,
            "billable": true,
            "billable_rate": null,
            "cost_rate": null,
            "created_at": null,
            "updated_at": null,
            "spent_date": null,
            "notes": null,
            "timer_started_at": null,
            "started_time": null,
            "ended_time": null
        })))
        .mount(&server)
        .await;

    // ── Clients ────────────────────────────────────────────────
    Mock::given(method("GET"))
        .and(path("/clients"))
        .respond_with(json_response(mock_data::paginated(
            "clients",
            vec![
                mock_data::client_a_json(),
                mock_data::client_b_json(),
                mock_data::client_c_json(),
                mock_data::client_d_json(),
            ],
        )))
        .mount(&server)
        .await;

    let d = delay();
    if d.as_millis() > 0 {
        eprintln!("🐢 HARV_MOCK delay: {} ms", d.as_millis());
    }

    server.uri()
}
