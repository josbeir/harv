//! In-process mock Harvest API server used when `HARV_MOCK=1` is set.
//!
//! Only compiled when the `mock-mode` feature is enabled.
//! Uses `wiremock` to serve realistic but hardcoded responses for all
//! API endpoints needed by the CLI and TUI.

use crate::mock_data;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn json_response(body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200)
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
    Mock::given(method("GET"))
        .and(path("/projects/100"))
        .respond_with(json_response(mock_data::project_alpha_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/projects/101"))
        .respond_with(json_response(mock_data::project_beta_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/projects/102"))
        .respond_with(json_response(mock_data::project_gamma_json()))
        .mount(&server)
        .await;

    // ── Task assignments (per project) ─────────────────────────
    Mock::given(method("GET"))
        .and(path("/projects/100/task_assignments"))
        .respond_with(json_response(mock_data::paginated(
            "task_assignments",
            vec![
                serde_json::json!({"id": 10, "task": {"id": 200, "name": "Development"}}),
                serde_json::json!({"id": 11, "task": {"id": 201, "name": "Design"}}),
                serde_json::json!({"id": 12, "task": {"id": 203, "name": "Code Review"}}),
            ],
        )))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/projects/101/task_assignments"))
        .respond_with(json_response(mock_data::paginated(
            "task_assignments",
            vec![
                serde_json::json!({"id": 20, "task": {"id": 200, "name": "Development"}}),
                serde_json::json!({"id": 21, "task": {"id": 201, "name": "Design"}}),
            ],
        )))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/projects/102/task_assignments"))
        .respond_with(json_response(mock_data::paginated(
            "task_assignments",
            vec![
                serde_json::json!({"id": 30, "task": {"id": 200, "name": "Development"}}),
                serde_json::json!({"id": 31, "task": {"id": 202, "name": "Meetings"}}),
            ],
        )))
        .mount(&server)
        .await;

    // ── Tasks ──────────────────────────────────────────────────
    Mock::given(method("GET"))
        .and(path("/tasks"))
        .respond_with(json_response(mock_data::paginated(
            "tasks",
            vec![
                serde_json::json!({"id": 200, "name": "Development", "billable_by_default": true, "default_hourly_rate": null, "is_default": false, "is_active": true, "created_at": null, "updated_at": null}),
                serde_json::json!({"id": 201, "name": "Design", "billable_by_default": true, "default_hourly_rate": null, "is_default": false, "is_active": true, "created_at": null, "updated_at": null}),
                serde_json::json!({"id": 202, "name": "Meetings", "billable_by_default": true, "default_hourly_rate": null, "is_default": false, "is_active": true, "created_at": null, "updated_at": null}),
                serde_json::json!({"id": 203, "name": "Code Review", "billable_by_default": true, "default_hourly_rate": null, "is_default": false, "is_active": true, "created_at": null, "updated_at": null}),
            ],
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
                .set_body_json(serde_json::json!({
                    "id": id,
                    "spent_date": "2026-06-14",
                    "hours": 1.0,
                    "notes": null,
                    "is_running": false,
                    "timer_started_at": null,
                    "started_time": null,
                    "ended_time": null,
                    "project": {"id": 100, "name": "Website Redesign"},
                    "task": {"id": 200, "name": "Development"},
                    "user": {"id": 1, "name": "Test User"},
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
            "project": {"id": 100, "name": "Website Redesign"},
            "task": {"id": 200, "name": "Development"},
            "user": {"id": 1, "name": "Test User"},
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
            "project": {"id": 100, "name": "Website Redesign"},
            "task": {"id": 200, "name": "Development"},
            "user": {"id": 1, "name": "Test User"},
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
            vec![mock_data::client_a_json(), mock_data::client_b_json()],
        )))
        .mount(&server)
        .await;

    server.uri()
}
