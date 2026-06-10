use harv_cli::commands;
use harv_sdk::{HarvClient, HarvConfig};
use serde_json::json;
use std::collections::HashMap;
use tokio::sync::Mutex;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

static ENV_MUTEX: Mutex<()> = Mutex::const_new(());

fn test_config() -> HarvConfig {
    HarvConfig {
        access_token: "t".into(),
        account_id: "1".into(),
        cache_ttl_hours: 24,
        last_project_id: None,
        last_task_id: None,
        aliases: HashMap::new(),
    }
}

fn client(uri: &str) -> HarvClient {
    HarvClient::new(test_config()).unwrap().with_base_url(uri)
}

fn client_with_last_used(uri: &str, pid: u64, tid: u64) -> HarvClient {
    let mut config = test_config();
    config.last_project_id = Some(pid);
    config.last_task_id = Some(tid);
    HarvClient::new(config).unwrap().with_base_url(uri)
}

fn json_response(body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200)
        .set_body_json(body)
        .insert_header("Content-Type", "application/json")
}

fn project_assignments_json() -> serde_json::Value {
    json!({
        "project_assignments": [{
            "id": 1, "project": {"id": 100, "name": "Test Project"},
            "client": {"id": 1, "name": "Test Client"},
            "task_assignments": [
                {"id": 10, "task": {"id": 200, "name": "Development"}},
                {"id": 11, "task": {"id": 201, "name": "Design"}}
            ],
            "is_active": true
        }],
        "total_pages": 1, "page": 1, "total_entries": 1, "per_page": 100
    })
}

fn user_json() -> serde_json::Value {
    json!({"id": 1, "first_name": "Test", "last_name": "User", "email": "test@test.com", "is_active": true, "created_at": null, "updated_at": null, "access_roles": ["member"]})
}

// --- Projects command ---

#[tokio::test]
async fn test_projects_execute() {
    let server = MockServer::start().await;
    let c = client(&server.uri());
    Mock::given(method("GET"))
        .and(path("/users/me/project_assignments"))
        .respond_with(json_response(project_assignments_json()))
        .mount(&server)
        .await;

    commands::projects::execute(&c, None, false, &harv_cli::OutputFormat::Table)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_projects_with_search() {
    let server = MockServer::start().await;
    let c = client(&server.uri());
    Mock::given(method("GET"))
        .and(path("/users/me/project_assignments"))
        .respond_with(json_response(project_assignments_json()))
        .mount(&server)
        .await;

    commands::projects::execute(
        &c,
        Some("Test".into()),
        false,
        &harv_cli::OutputFormat::Table,
    )
    .await
    .unwrap();
}

// --- Tasks command ---

#[tokio::test]
async fn test_tasks_execute() {
    let server = MockServer::start().await;
    let c = client(&server.uri());
    Mock::given(method("GET"))
        .and(path("/projects/100/task_assignments"))
        .respond_with(json_response(json!({
            "task_assignments": [
                {"id": 10, "task": {"id": 200, "name": "Development"}}
            ],
            "total_pages": 1, "page": 1, "total_entries": 1, "per_page": 100
        })))
        .mount(&server)
        .await;

    commands::tasks::execute(&c, 100, &harv_cli::OutputFormat::Table)
        .await
        .unwrap();
}

// --- Stop command ---

#[tokio::test]
async fn test_stop_no_timer() {
    let server = MockServer::start().await;
    let c = client(&server.uri());
    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/time_entries"))
        .respond_with(json_response(json!({"time_entries": [], "total_pages": 1, "page": 1, "total_entries": 0, "per_page": 100})))
        .mount(&server).await;

    commands::stop::execute(&c, None, false, false)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_stop_single_timer() {
    let server = MockServer::start().await;
    let c = client(&server.uri());

    let running_entry = json!({
        "id": 1, "is_running": true, "hours": null, "timer_started_at": "2026-06-08T14:00:00Z",
        "project": {"id": 100, "name": "Test Project"}, "task": {"id": 200, "name": "Development"},
        "user": {"id": 1, "name": "Test User"}, "client": {"id": 1, "name": "Test Client"},
        "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
        "created_at": null, "updated_at": null,
        "spent_date": null, "notes": null, "started_time": null, "ended_time": null
    });

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/time_entries"))
        .respond_with(json_response(json!({"time_entries": [running_entry], "total_pages": 1, "page": 1, "total_entries": 1, "per_page": 100})))
        .mount(&server).await;
    Mock::given(method("PATCH")).and(path("/time_entries/1/stop"))
        .respond_with(json_response(json!({
            "id": 1, "is_running": false, "hours": 1.5,
            "project": {"id": 100, "name": "Test Project"}, "task": {"id": 200, "name": "Development"},
            "user": {"id": 1, "name": "Test User"}, "client": {"id": 1, "name": "Test Client"},
            "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
            "created_at": null, "updated_at": null,
            "spent_date": null, "notes": null, "timer_started_at": null, "started_time": null, "ended_time": null
        })))
        .mount(&server).await;

    commands::stop::execute(&c, None, false, false)
        .await
        .unwrap();
}

// --- Note command ---

#[tokio::test]
async fn test_note_no_timer() {
    let server = MockServer::start().await;
    let c = client(&server.uri());
    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/time_entries"))
        .respond_with(json_response(json!({"time_entries": [], "total_pages": 1, "page": 1, "total_entries": 0, "per_page": 100})))
        .mount(&server).await;

    commands::note::execute(&c, None, false, false)
        .await
        .unwrap();
}

// --- Status command ---

#[tokio::test]
async fn test_status_no_timers() {
    let server = MockServer::start().await;
    let c = client(&server.uri());
    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/time_entries"))
        .respond_with(json_response(json!({"time_entries": [], "total_pages": 1, "page": 1, "total_entries": 0, "per_page": 100})))
        .mount(&server).await;

    commands::status::execute(&c, &harv_cli::OutputFormat::Table)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_status_with_timers() {
    let server = MockServer::start().await;
    let c = client(&server.uri());

    let running = json!({
        "id": 1, "is_running": true, "timer_started_at": "2026-06-08T14:00:00Z",
        "project": {"id": 100, "name": "Test Project"}, "task": {"id": 200, "name": "Development"},
        "user": {"id": 1, "name": "Test User"}, "client": {"id": 1, "name": "Test Client"},
        "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
        "created_at": null, "updated_at": null,
        "spent_date": null, "notes": null, "hours": null, "started_time": null, "ended_time": null
    });

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/time_entries"))
        .respond_with(json_response(json!({"time_entries": [running], "total_pages": 1, "page": 1, "total_entries": 1, "per_page": 100})))
        .mount(&server).await;
    Mock::given(method("GET")).and(path("/time_entries"))
        .respond_with(json_response(json!({"time_entries": [], "total_pages": 1, "page": 1, "total_entries": 0, "per_page": 100})))
        .mount(&server).await;

    commands::status::execute(&c, &harv_cli::OutputFormat::Table)
        .await
        .unwrap();
}

// --- Config command ---

#[tokio::test]
async fn test_config_execute_no_file() {
    let _guard = ENV_MUTEX.lock().await;
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    commands::config_cmd::execute(&harv_cli::ConfigArgs { action: None })
        .await
        .unwrap();
}

#[tokio::test]
async fn test_config_show_with_file() {
    let _guard = ENV_MUTEX.lock().await;
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.json"),
        r#"{"access_token":"tok","account_id":"1","cache_ttl_hours":48}"#,
    )
    .unwrap();
    commands::config_cmd::execute(&harv_cli::ConfigArgs { action: None })
        .await
        .unwrap();
}

#[tokio::test]
async fn test_config_get_cache_ttl() {
    let _guard = ENV_MUTEX.lock().await;
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.json"),
        r#"{"access_token":"tok","account_id":"1","cache_ttl_hours":48}"#,
    )
    .unwrap();
    commands::config_cmd::execute(&harv_cli::ConfigArgs {
        action: Some(harv_cli::ConfigAction::Get {
            setting: "cache-ttl".into(),
        }),
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn test_config_get_invalid() {
    let _guard = ENV_MUTEX.lock().await;
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.json"),
        r#"{"access_token":"tok","account_id":"1"}"#,
    )
    .unwrap();
    let result = commands::config_cmd::execute(&harv_cli::ConfigArgs {
        action: Some(harv_cli::ConfigAction::Get {
            setting: "bogus".into(),
        }),
    })
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_config_set_cache_ttl() {
    let _guard = ENV_MUTEX.lock().await;
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.json"),
        r#"{"access_token":"tok","account_id":"1"}"#,
    )
    .unwrap();
    commands::config_cmd::execute(&harv_cli::ConfigArgs {
        action: Some(harv_cli::ConfigAction::Set {
            setting: "cache-ttl".into(),
            value: "72".into(),
        }),
    })
    .await
    .unwrap();

    let config = HarvConfig::load().await.unwrap();
    assert_eq!(config.cache_ttl_hours, 72);
}

#[tokio::test]
async fn test_config_set_invalid() {
    let _guard = ENV_MUTEX.lock().await;
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.json"),
        r#"{"access_token":"tok","account_id":"1"}"#,
    )
    .unwrap();
    let result = commands::config_cmd::execute(&harv_cli::ConfigArgs {
        action: Some(harv_cli::ConfigAction::Set {
            setting: "bogus".into(),
            value: "1".into(),
        }),
    })
    .await;
    assert!(result.is_err());
}

// --- Track command (with provided IDs, bypasses prompts) ---

#[tokio::test]
async fn test_track_with_ids() {
    let server = MockServer::start().await;
    let c = client(&server.uri());

    Mock::given(method("GET"))
        .and(path("/users/me/project_assignments"))
        .respond_with(json_response(project_assignments_json()))
        .mount(&server)
        .await;
    Mock::given(method("POST")).and(path("/time_entries"))
        .respond_with(json_response(json!({
            "id": 99, "spent_date": "2026-06-08", "hours": 2.0, "notes": null,
            "is_running": false, "timer_started_at": null, "started_time": null, "ended_time": null,
            "project": {"id": 100, "name": "Test Project"}, "task": {"id": 200, "name": "Development"},
            "user": {"id": 1, "name": "Test User"}, "client": null,
            "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
            "created_at": null, "updated_at": null
        }))).mount(&server).await;

    commands::track::execute(
        &c,
        Some(100),
        Some(200),
        Some(2.0),
        Some("notes".into()),
        false,
        Some("2026-06-08".into()),
        false,
        None,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_track_with_last_used_auto_task() {
    let _guard = ENV_MUTEX.lock().await;
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();

    let server = MockServer::start().await;
    let c = client_with_last_used(&server.uri(), 100, 200);

    Mock::given(method("GET"))
        .and(path("/users/me/project_assignments"))
        .respond_with(json_response(project_assignments_json()))
        .mount(&server)
        .await;
    Mock::given(method("POST")).and(path("/time_entries"))
        .respond_with(json_response(json!({
            "id": 99, "spent_date": "2026-06-08", "hours": 2.0, "notes": null,
            "is_running": false, "timer_started_at": null, "started_time": null, "ended_time": null,
            "project": {"id": 100, "name": "Test Project"}, "task": {"id": 200, "name": "Development"},
            "user": {"id": 1, "name": "Test User"}, "client": null,
            "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
            "created_at": null, "updated_at": null
        }))).mount(&server).await;

    commands::track::execute(
        &c,
        Some(100),
        None,
        Some(2.0),
        None,
        false,
        Some("2026-06-08".into()),
        false,
        None,
    )
    .await
    .unwrap();
}

// --- Log command ---

#[tokio::test]
async fn test_log_with_ids() {
    let server = MockServer::start().await;
    let c = client(&server.uri());

    Mock::given(method("GET"))
        .and(path("/users/me/project_assignments"))
        .respond_with(json_response(project_assignments_json()))
        .mount(&server)
        .await;
    Mock::given(method("POST")).and(path("/time_entries"))
        .respond_with(json_response(json!({
            "id": 99, "spent_date": "2026-06-08", "hours": 2.0, "notes": null,
            "is_running": false, "timer_started_at": null, "started_time": null, "ended_time": null,
            "project": {"id": 100, "name": "Test Project"}, "task": {"id": 200, "name": "Development"},
            "user": {"id": 1, "name": "Test User"}, "client": null,
            "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
            "created_at": null, "updated_at": null
        }))).mount(&server).await;

    commands::log::execute(
        &c,
        2.0,
        None,
        Some(100),
        Some(200),
        None,
        false,
        Some("2026-06-08".into()),
        false,
    )
    .await
    .unwrap();
}

// --- Note command with inline notes ---

#[tokio::test]
async fn test_note_single_timer() {
    let server = MockServer::start().await;
    let c = client(&server.uri());

    let running = json!({
        "id": 1, "is_running": true, "timer_started_at": "2026-06-08T14:00:00Z",
        "project": {"id": 100, "name": "Test Project"}, "task": {"id": 200, "name": "Development"},
        "user": {"id": 1, "name": "Test User"}, "client": {"id": 1, "name": "Test Client"},
        "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
        "created_at": null, "updated_at": null,
        "spent_date": null, "notes": null, "hours": null, "started_time": null, "ended_time": null
    });

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/time_entries"))
        .respond_with(json_response(json!({"time_entries": [running], "total_pages": 1, "page": 1, "total_entries": 1, "per_page": 100})))
        .mount(&server).await;
    Mock::given(method("PATCH")).and(path("/time_entries/1"))
        .respond_with(json_response(json!({
            "id": 1, "is_running": true, "notes": "updated notes",
            "project": {"id": 100, "name": "Test Project"}, "task": {"id": 200, "name": "Development"},
            "user": {"id": 1, "name": "Test User"}, "client": {"id": 1, "name": "Test Client"},
            "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
            "created_at": null, "updated_at": null,
            "spent_date": null, "hours": null, "timer_started_at": null, "started_time": null, "ended_time": null
        }))).mount(&server).await;

    commands::note::execute(&c, Some("updated notes".into()), false, false)
        .await
        .unwrap();
}

// --- Alias list ---

#[tokio::test]
async fn test_alias_list_empty() {
    let _guard = ENV_MUTEX.lock().await;
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.json"),
        r#"{"access_token":"t","account_id":"1"}"#,
    )
    .unwrap();
    commands::alias::list_execute(&harv_cli::OutputFormat::Table)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_alias_delete_not_found() {
    let _guard = ENV_MUTEX.lock().await;
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.json"),
        r#"{"access_token":"t","account_id":"1"}"#,
    )
    .unwrap();
    commands::alias::delete_execute("nonexistent")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_alias_list_with_data() {
    let _guard = ENV_MUTEX.lock().await;
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.json"),
        r#"{"access_token":"t","account_id":"1","aliases":{"dev":{"project_id":10,"task_id":20}}}"#,
    )
    .unwrap();
    commands::alias::list_execute(&harv_cli::OutputFormat::Table)
        .await
        .unwrap();
}

// --- Start command delegation test ---

#[tokio::test]
async fn test_start_delegation() {
    let server = MockServer::start().await;
    let c = client(&server.uri());

    Mock::given(method("GET"))
        .and(path("/users/me/project_assignments"))
        .respond_with(json_response(project_assignments_json()))
        .mount(&server)
        .await;
    Mock::given(method("POST")).and(path("/time_entries"))
        .respond_with(json_response(json!({
            "id": 99, "spent_date": "2026-06-08", "hours": null, "notes": null,
            "is_running": true, "timer_started_at": "2026-06-08T14:00:00Z",
            "started_time": null, "ended_time": null,
            "project": {"id": 100, "name": "Test Project"}, "task": {"id": 200, "name": "Development"},
            "user": {"id": 1, "name": "Test User"}, "client": null,
            "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
            "created_at": null, "updated_at": null
        }))).mount(&server).await;

    commands::start::execute(
        &c,
        None,
        Some(100),
        Some(200),
        None,
        false,
        Some("2026-06-08".into()),
        false,
    )
    .await
    .unwrap();
}

// --- Note command with existing notes (append) ---

#[tokio::test]
async fn test_note_append_to_existing() {
    let server = MockServer::start().await;
    let c = client(&server.uri());

    let running = json!({
        "id": 1, "is_running": true, "timer_started_at": "2026-06-08T14:00:00Z",
        "project": {"id": 100, "name": "Test Project"}, "task": {"id": 200, "name": "Development"},
        "user": {"id": 1, "name": "Test User"}, "client": {"id": 1, "name": "Test Client"},
        "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
        "created_at": null, "updated_at": null,
        "spent_date": null, "notes": "existing notes\nmore notes", "hours": null,
        "started_time": null, "ended_time": null
    });

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/time_entries"))
        .respond_with(json_response(json!({"time_entries": [running], "total_pages": 1, "page": 1, "total_entries": 1, "per_page": 100})))
        .mount(&server).await;
    Mock::given(method("PATCH"))
        .and(path("/time_entries/1"))
        .respond_with(json_response(running.clone()))
        .mount(&server)
        .await;

    commands::note::execute(&c, Some("fresh note".into()), false, false)
        .await
        .unwrap();
}

// --- Note command with overwrite ---

#[tokio::test]
async fn test_note_overwrite() {
    let server = MockServer::start().await;
    let c = client(&server.uri());

    let running = json!({
        "id": 1, "is_running": true, "timer_started_at": "2026-06-08T14:00:00Z",
        "project": {"id": 100, "name": "Test Project"}, "task": {"id": 200, "name": "Development"},
        "user": {"id": 1, "name": "Test User"}, "client": {"id": 1, "name": "Test Client"},
        "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
        "created_at": null, "updated_at": null,
        "spent_date": null, "notes": "old notes", "hours": null,
        "started_time": null, "ended_time": null
    });

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/time_entries"))
        .respond_with(json_response(json!({"time_entries": [running], "total_pages": 1, "page": 1, "total_entries": 1, "per_page": 100})))
        .mount(&server).await;
    Mock::given(method("PATCH"))
        .and(path("/time_entries/1"))
        .respond_with(json_response(running.clone()))
        .mount(&server)
        .await;

    commands::note::execute(&c, Some("replaced".into()), true, false)
        .await
        .unwrap();
}

// --- Stop command with notes append ---

#[tokio::test]
async fn test_stop_with_notes_append() {
    let server = MockServer::start().await;
    let c = client(&server.uri());

    let running_entry = json!({
        "id": 1, "is_running": true, "hours": null, "timer_started_at": "2026-06-08T14:00:00Z",
        "project": {"id": 100, "name": "Test Project"}, "task": {"id": 200, "name": "Development"},
        "user": {"id": 1, "name": "Test User"}, "client": {"id": 1, "name": "Test Client"},
        "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
        "created_at": null, "updated_at": null,
        "spent_date": null, "notes": "previous notes", "started_time": null, "ended_time": null
    });

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/time_entries"))
        .respond_with(json_response(json!({"time_entries": [running_entry], "total_pages": 1, "page": 1, "total_entries": 1, "per_page": 100})))
        .mount(&server).await;
    Mock::given(method("PATCH"))
        .and(path("/time_entries/1"))
        .respond_with(json_response(running_entry.clone()))
        .mount(&server)
        .await;
    Mock::given(method("PATCH")).and(path("/time_entries/1/stop"))
        .respond_with(json_response(json!({
            "id": 1, "is_running": false, "hours": 1.5,
            "project": {"id": 100, "name": "Test Project"}, "task": {"id": 200, "name": "Development"},
            "user": {"id": 1, "name": "Test User"}, "client": {"id": 1, "name": "Test Client"},
            "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
            "created_at": null, "updated_at": null,
            "spent_date": null, "notes": "previous notes\n\nstop note", "timer_started_at": null,
            "started_time": null, "ended_time": null
        }))).mount(&server).await;

    commands::stop::execute(&c, Some("stop note".into()), false, false)
        .await
        .unwrap();
}

// --- Stop command with notes overwrite ---

#[tokio::test]
async fn test_stop_with_notes_overwrite() {
    let server = MockServer::start().await;
    let c = client(&server.uri());

    let running_entry = json!({
        "id": 1, "is_running": true, "hours": null, "timer_started_at": "2026-06-08T14:00:00Z",
        "project": {"id": 100, "name": "Test Project"}, "task": {"id": 200, "name": "Development"},
        "user": {"id": 1, "name": "Test User"}, "client": {"id": 1, "name": "Test Client"},
        "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
        "created_at": null, "updated_at": null,
        "spent_date": null, "notes": "old", "started_time": null, "ended_time": null
    });

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET")).and(path("/time_entries"))
        .respond_with(json_response(json!({"time_entries": [running_entry], "total_pages": 1, "page": 1, "total_entries": 1, "per_page": 100})))
        .mount(&server).await;
    Mock::given(method("PATCH"))
        .and(path("/time_entries/1"))
        .respond_with(json_response(running_entry.clone()))
        .mount(&server)
        .await;
    Mock::given(method("PATCH")).and(path("/time_entries/1/stop"))
        .respond_with(json_response(json!({
            "id": 1, "is_running": false, "hours": 1.5,
            "project": {"id": 100, "name": "Test Project"}, "task": {"id": 200, "name": "Development"},
            "user": {"id": 1, "name": "Test User"}, "client": {"id": 1, "name": "Test Client"},
            "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
            "created_at": null, "updated_at": null,
            "spent_date": null, "notes": "fresh", "timer_started_at": null,
            "started_time": null, "ended_time": null
        }))).mount(&server).await;

    commands::stop::execute(&c, Some("fresh".into()), true, false)
        .await
        .unwrap();
}

// --- Whoami ---

#[tokio::test]
async fn test_whoami_execute() {
    let server = MockServer::start().await;
    let c = client(&server.uri());
    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/company"))
        .respond_with(json_response(json!({"name": "Test Company"})))
        .mount(&server)
        .await;

    commands::whoami::execute(&c, &harv_cli::OutputFormat::Table)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_whoami_execute_json() {
    let server = MockServer::start().await;
    let c = client(&server.uri());
    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/company"))
        .respond_with(json_response(json!({"name": "Test Company"})))
        .mount(&server)
        .await;

    commands::whoami::execute(&c, &harv_cli::OutputFormat::Json)
        .await
        .unwrap();
}

// --- Disconnect ---

#[tokio::test]
async fn test_disconnect_no_config() {
    let _guard = ENV_MUTEX.lock().await;
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };

    commands::disconnect::run().await.unwrap();
}

#[tokio::test]
async fn test_disconnect_with_config() {
    let _guard = ENV_MUTEX.lock().await;
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };

    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    let config_path = harv_dir.join("config.json");
    std::fs::write(&config_path, r#"{"access_token":"tok","account_id":"123"}"#).unwrap();
    let cache_path = harv_dir.join("projects_cache_123.json");
    std::fs::write(&cache_path, "{}").unwrap();

    assert!(config_path.exists());
    assert!(cache_path.exists());

    commands::disconnect::run().await.unwrap();

    assert!(!config_path.exists());
    assert!(!cache_path.exists());
}
