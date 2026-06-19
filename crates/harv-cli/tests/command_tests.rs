use harv_cli::commands;
use harv_sdk::mock_data;
use harv_sdk::{Alias, HarvClient, HarvConfig};
use serde_json::json;
use tokio::sync::Mutex;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

static ENV_MUTEX: Mutex<()> = Mutex::const_new(());

fn ensure_locale() {
    harv_core::init_locale(None);
}

fn test_config() -> HarvConfig {
    mock_data::test_config()
}

fn client(uri: &str) -> HarvClient {
    HarvClient::new(test_config()).unwrap().with_base_url(uri)
}

fn client_with_last_used(uri: &str, pid: u64, tid: u64) -> HarvClient {
    HarvClient::new(mock_data::config_with_last_used(pid, tid))
        .unwrap()
        .with_base_url(uri)
}

fn json_response(body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200)
        .set_body_json(body)
        .insert_header("Content-Type", "application/json")
}

async fn mock_assignments_and_projects(server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/users/me/project_assignments"))
        .respond_with(json_response(mock_data::project_assignments_minimal_json()))
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path("/projects"))
        .respond_with(json_response(mock_data::paginated(
            "projects",
            vec![mock_data::project_minimal_json()],
        )))
        .mount(server)
        .await;
}

fn user_json() -> serde_json::Value {
    mock_data::user_json()
}

// --- Projects command ---

#[tokio::test]
async fn test_projects_execute() {
    ensure_locale();
    let server = MockServer::start().await;
    let c = client(&server.uri());
    mock_assignments_and_projects(&server).await;

    commands::projects::execute(&c, None, false, &harv_cli::OutputFormat::Table)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_projects_with_search() {
    ensure_locale();
    let server = MockServer::start().await;
    let c = client(&server.uri());
    mock_assignments_and_projects(&server).await;

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
    ensure_locale();
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
    ensure_locale();
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
    ensure_locale();
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
    ensure_locale();
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
    ensure_locale();
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
    ensure_locale();
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
    ensure_locale();
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
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "tok"
account_id = "1"
cache_ttl_hours = 48
"#,
    )
    .unwrap();
    commands::config_cmd::execute(&harv_cli::ConfigArgs { action: None })
        .await
        .unwrap();
}

#[tokio::test]
async fn test_config_get_cache_ttl() {
    let _guard = ENV_MUTEX.lock().await;
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "tok"
account_id = "1"
cache_ttl_hours = 48
"#,
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
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "tok"
account_id = "1"
"#,
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
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "tok"
account_id = "1"
"#,
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
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "tok"
account_id = "1"
"#,
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
    let _guard = ENV_MUTEX.lock().await;
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let server = MockServer::start().await;
    let c = client(&server.uri());

    mock_assignments_and_projects(&server).await;
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
        false,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_track_with_last_used_auto_task() {
    let _guard = ENV_MUTEX.lock().await;
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();

    let server = MockServer::start().await;
    let c = client_with_last_used(&server.uri(), 100, 200);

    mock_assignments_and_projects(&server).await;
    Mock::given(method("POST")).and(path("/time_entries"))
        .respond_with(json_response(json!({
            "id": 99, "spent_date": "2026-06-08", "hours": 2.0, "notes": "test",
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
        Some("test".into()),
        false,
        Some("2026-06-08".into()),
        false,
        None,
        false,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_track_no_project_assignments() {
    let _guard = ENV_MUTEX.lock().await;
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "t"
account_id = "1"
"#,
    )
    .unwrap();

    let server = MockServer::start().await;
    let c = client(&server.uri());
    Mock::given(method("GET"))
        .and(path("/users/me/project_assignments"))
        .respond_with(json_response(json!({
            "project_assignments": [],
            "total_pages": 1, "page": 1, "total_entries": 0, "per_page": 100
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/projects"))
        .respond_with(json_response(mock_data::paginated("projects", vec![])))
        .mount(&server)
        .await;

    let result =
        commands::track::execute(&c, None, None, None, None, false, None, false, None, false).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_track_alias_not_found() {
    let _guard = ENV_MUTEX.lock().await;
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "t"
account_id = "1"
"#,
    )
    .unwrap();

    let server = MockServer::start().await;
    let c = client(&server.uri());
    Mock::given(method("GET"))
        .and(path("/users/me/project_assignments"))
        .respond_with(json_response(mock_data::project_assignments_minimal_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/projects"))
        .respond_with(json_response(mock_data::paginated(
            "projects",
            vec![mock_data::project_minimal_json()],
        )))
        .mount(&server)
        .await;

    let result = commands::track::execute(
        &c,
        None,
        None,
        None,
        None,
        false,
        None,
        false,
        Some("nonexistent".into()),
        false,
    )
    .await;
    assert!(result.is_err());
}

// --- Note command with inline notes ---

#[tokio::test]
async fn test_note_single_timer() {
    ensure_locale();
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
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "t"
account_id = "1"
"#,
    )
    .unwrap();
    let c = client("http://unused"); // never hit — aliases empty, early returns
    commands::alias::list_execute(&c, &harv_cli::OutputFormat::Table)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_alias_delete_not_found() {
    let _guard = ENV_MUTEX.lock().await;
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "t"
account_id = "1"
"#,
    )
    .unwrap();
    commands::alias::delete_execute("nonexistent")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_alias_list_with_data() {
    ensure_locale();
    let server = MockServer::start().await;
    let mut config = test_config();
    config.aliases.insert(
        "dev".into(),
        Alias {
            project_id: 100,
            task_id: 200,
        },
    );
    let c = HarvClient::new(config)
        .unwrap()
        .with_base_url(&server.uri());
    mock_assignments_and_projects(&server).await;

    commands::alias::list_execute(&c, &harv_cli::OutputFormat::Table)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_alias_list_stale() {
    ensure_locale();
    let server = MockServer::start().await;
    let mut config = test_config();
    config.aliases.insert(
        "old".into(),
        Alias {
            project_id: 999,
            task_id: 888,
        },
    );
    let c = HarvClient::new(config)
        .unwrap()
        .with_base_url(&server.uri());
    mock_assignments_and_projects(&server).await;

    commands::alias::list_execute(&c, &harv_cli::OutputFormat::Table)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_alias_list_api_error() {
    ensure_locale();
    let server = MockServer::start().await;
    let mut config = test_config();
    config.aliases.insert(
        "dev".into(),
        Alias {
            project_id: 100,
            task_id: 200,
        },
    );
    let c = HarvClient::new(config)
        .unwrap()
        .with_base_url(&server.uri());
    Mock::given(method("GET"))
        .and(path("/users/me/project_assignments"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/projects"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let result = commands::alias::list_execute(&c, &harv_cli::OutputFormat::Table).await;
    assert!(result.is_err());
}

// --- Start command delegation test ---

#[tokio::test]
async fn test_start_delegation() {
    ensure_locale();
    let server = MockServer::start().await;
    let c = client(&server.uri());

    mock_assignments_and_projects(&server).await;
    Mock::given(method("POST")).and(path("/time_entries"))
        .respond_with(json_response(json!({
            "id": 99, "spent_date": "2026-06-08", "hours": null, "notes": "test",
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
        Some("test".into()),
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
    ensure_locale();
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
    ensure_locale();
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
    ensure_locale();
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
    ensure_locale();
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
    ensure_locale();
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
    ensure_locale();
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
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };

    commands::disconnect::run().await.unwrap();
}

#[tokio::test]
async fn test_disconnect_with_config() {
    let _guard = ENV_MUTEX.lock().await;
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };

    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    let config_path = harv_dir.join("config.toml");
    std::fs::write(
        &config_path,
        r#"access_token = "tok"
account_id = "123"
"#,
    )
    .unwrap();
    let cache_path = harv_dir.join("projects_cache_123.json");
    std::fs::write(&cache_path, "{}").unwrap();

    assert!(config_path.exists());
    assert!(cache_path.exists());

    commands::disconnect::run().await.unwrap();

    assert!(!config_path.exists());
    assert!(!cache_path.exists());
}

// --- harv edit tests ---

fn stopped_entry_json(id: u64, hours: f64) -> serde_json::Value {
    json!({
        "id": id, "is_running": false, "hours": hours,
        "spent_date": "2026-06-08",
        "project": {"id": 100, "name": "Test Project"},
        "task": {"id": 200, "name": "Development"},
        "user": {"id": 1, "name": "Test User"},
        "client": null,
        "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
        "created_at": null, "updated_at": null,
        "timer_started_at": null, "started_time": null, "ended_time": null,
        "notes": null
    })
}

fn running_entry_json(id: u64) -> serde_json::Value {
    json!({
        "id": id, "is_running": true, "hours": null,
        "spent_date": null,
        "timer_started_at": "2026-06-08T14:00:00Z",
        "project": {"id": 100, "name": "Test Project"},
        "task": {"id": 200, "name": "Development"},
        "user": {"id": 1, "name": "Test User"},
        "client": null,
        "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
        "created_at": null, "updated_at": null,
        "started_time": null, "ended_time": null,
        "notes": null
    })
}

#[tokio::test]
async fn test_edit_non_interactive_with_entry_id() {
    ensure_locale();
    let server = MockServer::start().await;
    let c = client(&server.uri());

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/time_entries/1"))
        .respond_with(json_response(stopped_entry_json(1, 1.0)))
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/time_entries/1"))
        .respond_with(json_response(stopped_entry_json(1, 2.5)))
        .mount(&server)
        .await;

    commands::edit::execute(
        &c,
        Some(1),
        Some(100),
        Some(201),
        Some(2.5),
        Some("revised".into()),
        false,
        false,
        None,
        false,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_edit_running_entry_rejects_hours() {
    ensure_locale();
    let server = MockServer::start().await;
    let c = client(&server.uri());

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/time_entries/1"))
        .respond_with(json_response(running_entry_json(1)))
        .mount(&server)
        .await;

    let result = commands::edit::execute(
        &c,
        Some(1),
        None,
        None,
        Some(2.0),
        None,
        false,
        false,
        None,
        false,
    )
    .await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Cannot change hours")
    );
}

#[tokio::test]
async fn test_edit_running_entry_rejects_date() {
    ensure_locale();
    let server = MockServer::start().await;
    let c = client(&server.uri());

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/time_entries/1"))
        .respond_with(json_response(running_entry_json(1)))
        .mount(&server)
        .await;

    let result = commands::edit::execute(
        &c,
        Some(1),
        None,
        None,
        None,
        None,
        false,
        false,
        Some("2026-06-09".into()),
        false,
    )
    .await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Cannot change the date")
    );
}

#[tokio::test]
async fn test_edit_running_entry_allows_notes() {
    ensure_locale();
    let server = MockServer::start().await;
    let c = client(&server.uri());

    let updated = json!({
        "id": 1, "is_running": true, "hours": null,
        "timer_started_at": "2026-06-08T14:00:00Z",
        "spent_date": null,
        "project": {"id": 100, "name": "Test Project"},
        "task": {"id": 200, "name": "Development"},
        "user": {"id": 1, "name": "Test User"},
        "client": null, "notes": "added note",
        "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
        "created_at": null, "updated_at": null,
        "started_time": null, "ended_time": null
    });

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/time_entries/1"))
        .respond_with(json_response(running_entry_json(1)))
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/time_entries/1"))
        .respond_with(json_response(updated))
        .mount(&server)
        .await;

    commands::edit::execute(
        &c,
        Some(1),
        None,
        None,
        None,
        Some("added note".into()),
        false,
        false,
        None,
        false,
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn test_edit_confirmation_falls_back_to_submitted_hours() {
    ensure_locale();
    let server = MockServer::start().await;
    let c = client(&server.uri());

    // PATCH response omits hours (API may do this for unchanged fields)
    let patch_response = json!({
        "id": 1, "is_running": false, "hours": null,
        "spent_date": "2026-06-08",
        "project": {"id": 100, "name": "Test Project"},
        "task": {"id": 200, "name": "Development"},
        "user": {"id": 1, "name": "Test User"},
        "client": null, "notes": null,
        "is_billed": false, "billable": true, "billable_rate": null, "cost_rate": null,
        "created_at": null, "updated_at": null,
        "timer_started_at": null, "started_time": null, "ended_time": null
    });

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/time_entries/1"))
        .respond_with(json_response(stopped_entry_json(1, 1.5)))
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/time_entries/1"))
        .respond_with(json_response(patch_response))
        .mount(&server)
        .await;

    // Should not panic — falls back to submitted entry.hours
    commands::edit::execute(
        &c,
        Some(1),
        Some(100),
        None,
        None,
        None,
        false,
        false,
        None,
        false,
    )
    .await
    .unwrap();
}

// --- Config locale get/set ---

#[tokio::test]
async fn test_config_get_locale() {
    let _guard = ENV_MUTEX.lock().await;
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "tok"
account_id = "1"
locale = "nl"
"#,
    )
    .unwrap();
    commands::config_cmd::execute(&harv_cli::ConfigArgs {
        action: Some(harv_cli::ConfigAction::Get {
            setting: "locale".into(),
        }),
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn test_config_get_account_id() {
    let _guard = ENV_MUTEX.lock().await;
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "tok"
account_id = "1"
"#,
    )
    .unwrap();
    commands::config_cmd::execute(&harv_cli::ConfigArgs {
        action: Some(harv_cli::ConfigAction::Get {
            setting: "account-id".into(),
        }),
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn test_config_get_aliases_empty() {
    let _guard = ENV_MUTEX.lock().await;
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "tok"
account_id = "1"
"#,
    )
    .unwrap();
    commands::config_cmd::execute(&harv_cli::ConfigArgs {
        action: Some(harv_cli::ConfigAction::Get {
            setting: "aliases".into(),
        }),
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn test_config_set_locale_valid() {
    let _guard = ENV_MUTEX.lock().await;
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "tok"
account_id = "1"
"#,
    )
    .unwrap();
    commands::config_cmd::execute(&harv_cli::ConfigArgs {
        action: Some(harv_cli::ConfigAction::Set {
            setting: "locale".into(),
            value: "fr".into(),
        }),
    })
    .await
    .unwrap();

    let config = HarvConfig::load().await.unwrap();
    assert_eq!(config.locale.as_deref(), Some("fr"));
}

#[tokio::test]
async fn test_config_set_locale_auto() {
    let _guard = ENV_MUTEX.lock().await;
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "tok"
account_id = "1"
locale = "nl"
"#,
    )
    .unwrap();
    commands::config_cmd::execute(&harv_cli::ConfigArgs {
        action: Some(harv_cli::ConfigAction::Set {
            setting: "locale".into(),
            value: "auto".into(),
        }),
    })
    .await
    .unwrap();

    let config = HarvConfig::load().await.unwrap();
    assert!(config.locale.is_none());
}

#[tokio::test]
async fn test_config_set_locale_invalid() {
    let _guard = ENV_MUTEX.lock().await;
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "tok"
account_id = "1"
"#,
    )
    .unwrap();
    let result = commands::config_cmd::execute(&harv_cli::ConfigArgs {
        action: Some(harv_cli::ConfigAction::Set {
            setting: "locale".into(),
            value: "jp".into(),
        }),
    })
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_config_set_cache_ttl_invalid() {
    let _guard = ENV_MUTEX.lock().await;
    ensure_locale();
    let tmp = tempfile::tempdir().unwrap();
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    unsafe { std::env::set_var("HOME", tmp.path()) };
    let harv_dir = tmp.path().join(".config").join("harv");
    std::fs::create_dir_all(&harv_dir).unwrap();
    std::fs::write(
        harv_dir.join("config.toml"),
        r#"access_token = "tok"
account_id = "1"
"#,
    )
    .unwrap();
    let result = commands::config_cmd::execute(&harv_cli::ConfigArgs {
        action: Some(harv_cli::ConfigAction::Set {
            setting: "cache-ttl".into(),
            value: "not-a-number".into(),
        }),
    })
    .await;
    assert!(result.is_err());
}

// --- Edit non-interactive with date ---

#[tokio::test]
async fn test_edit_non_interactive_with_date() {
    ensure_locale();
    let server = MockServer::start().await;
    let c = client(&server.uri());

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(user_json()))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/time_entries/1"))
        .respond_with(json_response(stopped_entry_json(1, 1.0)))
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/time_entries/1"))
        .respond_with(json_response(stopped_entry_json(1, 1.0)))
        .mount(&server)
        .await;

    commands::edit::execute(
        &c,
        Some(1),
        None,
        None,
        None,
        None,
        false,
        false,
        Some("2026-06-08".into()),
        false,
    )
    .await
    .unwrap();
}

// --- Track with date ---

#[tokio::test]
async fn test_track_with_date() {
    ensure_locale();
    let server = MockServer::start().await;
    let c = client(&server.uri());

    mock_assignments_and_projects(&server).await;
    Mock::given(method("POST")).and(path("/time_entries"))
        .respond_with(json_response(json!({
            "id": 99, "spent_date": "2026-06-01", "hours": 2.0, "notes": "notes",
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
        Some("2026-06-01".into()),
        false,
        None,
        false,
    )
    .await
    .unwrap();
}
