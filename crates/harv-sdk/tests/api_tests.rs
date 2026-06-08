use harv_sdk::HarvClient;
use harv_sdk::HarvConfig;
use serde_json::json;
use std::collections::HashMap;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_config() -> HarvConfig {
    HarvConfig {
        access_token: "test-token".into(),
        account_id: "1234567".into(),
        cache_ttl_hours: 24,
        last_project_id: None,
        last_task_id: None,
        aliases: HashMap::new(),
    }
}

fn json_response(body: serde_json::Value) -> ResponseTemplate {
    ResponseTemplate::new(200)
        .set_body_json(body)
        .insert_header("Content-Type", "application/json")
}

#[tokio::test]
async fn test_time_entries_list() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/time_entries"))
        .respond_with(json_response(json!({
            "time_entries": [{
                "id": 1,
                "spent_date": "2026-06-08",
                "hours": 2.5,
                "notes": null,
                "is_running": false,
                "timer_started_at": null,
                "started_time": null,
                "ended_time": null,
                "project": {"id": 100, "name": "Test Project"},
                "task": {"id": 200, "name": "Development"},
                "user": {"id": 300, "name": "Test User"},
                "is_billed": false,
                "billable": true,
                "billable_rate": null,
                "cost_rate": null,
                "created_at": null,
                "updated_at": null
            }],
            "total_pages": 1,
            "page": 1,
            "total_entries": 1,
            "per_page": 100
        })))
        .mount(&server)
        .await;

    let params = harv_sdk::resources::time_entries::TimeEntryListParams::default();
    let entries = client.time_entries().list(&params).await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].project.name, "Test Project");
}

#[tokio::test]
async fn test_time_entries_get() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/time_entries/42"))
        .respond_with(json_response(json!({
            "id": 42,
            "spent_date": "2026-06-08",
            "hours": 2.5,
            "notes": "Work done",
            "is_running": false,
            "timer_started_at": null,
            "project": {"id": 100, "name": "Test Project"},
            "task": {"id": 200, "name": "Development"},
            "user": {"id": 300, "name": "Test User"},
            "is_billed": false,
            "billable": true,
            "billable_rate": null,
            "cost_rate": null,
            "created_at": null,
            "updated_at": null,
            "started_time": null,
            "ended_time": null
        })))
        .mount(&server)
        .await;

    let entry = client.time_entries().get(42).await.unwrap();
    assert_eq!(entry.id, 42);
    assert_eq!(entry.hours, Some(2.5));
}

#[tokio::test]
async fn test_time_entries_create() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("POST"))
        .and(path("/time_entries"))
        .respond_with(json_response(json!({
            "id": 99,
            "spent_date": "2026-06-08",
            "hours": 2.0,
            "notes": "Testing",
            "is_running": false,
            "project": {"id": 100, "name": "Test Project"},
            "task": {"id": 200, "name": "Development"},
            "user": {"id": 300, "name": "Test User"},
            "is_billed": false,
            "billable": true,
            "billable_rate": null,
            "cost_rate": null,
            "created_at": null,
            "updated_at": null,
            "timer_started_at": null,
            "started_time": null,
            "ended_time": null
        })))
        .mount(&server)
        .await;

    use harv_core::CreateTimeEntry;
    let entry = CreateTimeEntry {
        project_id: 100,
        task_id: 200,
        spent_date: None,
        hours: Some(2.0),
        notes: Some("Testing".into()),
        started_time: None,
        ended_time: None,
    };
    let created = client.time_entries().create(&entry).await.unwrap();
    assert_eq!(created.id, 99);
}

#[tokio::test]
async fn test_time_entries_update() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("PATCH"))
        .and(path("/time_entries/42"))
        .respond_with(json_response(json!({
            "id": 42,
            "hours": 3.0,
            "notes": "Updated notes",
            "is_running": false,
            "project": {"id": 100, "name": "Test Project"},
            "task": {"id": 200, "name": "Development"},
            "user": {"id": 300, "name": "Test User"},
            "is_billed": false,
            "billable": true,
            "billable_rate": null,
            "cost_rate": null,
            "created_at": null,
            "updated_at": null,
            "spent_date": null,
            "timer_started_at": null,
            "started_time": null,
            "ended_time": null
        })))
        .mount(&server)
        .await;

    use harv_core::UpdateTimeEntry;
    let update = UpdateTimeEntry {
        hours: Some(3.0),
        notes: Some("Updated notes".into()),
        ..Default::default()
    };
    let entry = client.time_entries().update(42, &update).await.unwrap();
    assert_eq!(entry.hours, Some(3.0));
}

#[tokio::test]
async fn test_time_entries_delete() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("DELETE"))
        .and(path("/time_entries/42"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let result = client.time_entries().delete(42).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_time_entries_stop() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("PATCH"))
        .and(path("/time_entries/42/stop"))
        .respond_with(json_response(json!({
            "id": 42,
            "is_running": false,
            "hours": 1.5,
            "project": {"id": 100, "name": "Test Project"},
            "task": {"id": 200, "name": "Development"},
            "user": {"id": 300, "name": "Test User"},
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

    let entry = client.time_entries().stop(42).await.unwrap();
    assert!(!entry.is_running);
}

#[tokio::test]
async fn test_time_entries_restart() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("PATCH"))
        .and(path("/time_entries/42/restart"))
        .respond_with(json_response(json!({
            "id": 42,
            "is_running": true,
            "project": {"id": 100, "name": "Test Project"},
            "task": {"id": 200, "name": "Development"},
            "user": {"id": 300, "name": "Test User"},
            "is_billed": false,
            "billable": true,
            "billable_rate": null,
            "cost_rate": null,
            "created_at": null,
            "updated_at": null,
            "spent_date": null,
            "notes": null,
            "hours": null,
            "timer_started_at": null,
            "started_time": null,
            "ended_time": null
        })))
        .mount(&server)
        .await;

    let entry = client.time_entries().restart(42).await.unwrap();
    assert!(entry.is_running);
}

#[tokio::test]
async fn test_time_entries_running() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/time_entries"))
        .respond_with(json_response(json!({
            "time_entries": [{
                "id": 1,
                "is_running": true,
                "timer_started_at": "2026-06-08T14:00:00Z",
                "project": {"id": 100, "name": "Test Project"},
                "task": {"id": 200, "name": "Development"},
                "user": {"id": 300, "name": "Test User"},
                "is_billed": false,
                "billable": true,
                "billable_rate": null,
                "cost_rate": null,
                "created_at": null,
                "updated_at": null,
                "spent_date": null,
                "notes": null,
                "hours": null,
                "started_time": null,
                "ended_time": null
            }],
            "total_pages": 1,
            "page": 1,
            "total_entries": 1,
            "per_page": 100
        })))
        .mount(&server)
        .await;

    let entries = client.time_entries().running(300).await.unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].is_running);
}

#[tokio::test]
async fn test_unauthorized_response() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/company"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let result = client.company().get().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        harv_core::HarvError::NotAuthenticated => {}
        e => panic!("Expected NotAuthenticated, got {:?}", e),
    }
}

#[tokio::test]
async fn test_api_error_response() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/time_entries/999"))
        .respond_with(
            ResponseTemplate::new(422).set_body_string(r#"{"message": "Validation failed"}"#),
        )
        .mount(&server)
        .await;

    let result = client.time_entries().get(999).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        harv_core::HarvError::Api { status, .. } => assert_eq!(status, 422),
        e => panic!("Expected Api error, got {:?}", e),
    }
}

// --- Projects API tests ---

#[tokio::test]
async fn test_projects_list() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/projects"))
        .respond_with(json_response(json!({
            "projects": [{
                "id": 1,
                "name": "Test Project",
                "client": null,
                "is_active": true,
                "code": null,
                "notes": null,
                "starts_on": null,
                "ends_on": null,
                "created_at": null,
                "updated_at": null
            }],
            "total_pages": 1,
            "page": 1,
            "total_entries": 1,
            "per_page": 100
        })))
        .mount(&server)
        .await;

    let projects = client.projects().list().await.unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].name, "Test Project");
}

#[tokio::test]
async fn test_projects_get() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/projects/1"))
        .respond_with(json_response(json!({
            "id": 1,
            "name": "Test Project",
            "client": null,
            "is_active": true,
            "code": null,
            "notes": null,
            "starts_on": null,
            "ends_on": null,
            "created_at": null,
            "updated_at": null
        })))
        .mount(&server)
        .await;

    let project = client.projects().get(1).await.unwrap();
    assert_eq!(project.name, "Test Project");
}

#[tokio::test]
async fn test_projects_my_assignments() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/users/me/project_assignments"))
        .respond_with(json_response(json!({
            "project_assignments": [{
                "id": 1,
                "project": {"id": 100, "name": "Test Project"},
                "client": {"id": 50, "name": "Test Client"},
                "task_assignments": [
                    {"id": 10, "task": {"id": 200, "name": "Development"}}
                ],
                "is_active": true
            }],
            "total_pages": 1,
            "page": 1,
            "total_entries": 1,
            "per_page": 100
        })))
        .mount(&server)
        .await;

    let assignments = client.projects().my_assignments(false).await.unwrap();
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].project.name, "Test Project");
    assert_eq!(assignments[0].task_assignments.len(), 1);
}

#[tokio::test]
async fn test_projects_task_assignments() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/projects/100/task_assignments"))
        .respond_with(json_response(json!({
            "task_assignments": [
                {"id": 10, "task": {"id": 200, "name": "Development"}},
                {"id": 11, "task": {"id": 201, "name": "Code Review"}}
            ],
            "total_pages": 1,
            "page": 1,
            "total_entries": 2,
            "per_page": 100
        })))
        .mount(&server)
        .await;

    let tasks = client.projects().task_assignments(100).await.unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].task.name, "Development");
}

// --- Tasks API tests ---

#[tokio::test]
async fn test_tasks_list() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/tasks"))
        .respond_with(json_response(json!({
            "tasks": [{
                "id": 1,
                "name": "Development",
                "billable_by_default": true,
                "default_hourly_rate": null,
                "is_default": false,
                "is_active": true,
                "created_at": null,
                "updated_at": null
            }],
            "total_pages": 1,
            "page": 1,
            "total_entries": 1,
            "per_page": 100
        })))
        .mount(&server)
        .await;

    let tasks = client.tasks().list().await.unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].name, "Development");
}

#[tokio::test]
async fn test_tasks_get() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/tasks/1"))
        .respond_with(json_response(json!({
            "id": 1,
            "name": "Development",
            "billable_by_default": true,
            "default_hourly_rate": null,
            "is_default": false,
            "is_active": true,
            "created_at": null,
            "updated_at": null
        })))
        .mount(&server)
        .await;

    let task = client.tasks().get(1).await.unwrap();
    assert_eq!(task.name, "Development");
}

// --- Users API tests ---

#[tokio::test]
async fn test_users_me() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/users/me"))
        .respond_with(json_response(json!({
            "id": 1,
            "first_name": "Test",
            "last_name": "User",
            "email": "test@example.com",
            "is_active": true,
            "created_at": null,
            "updated_at": null
        })))
        .mount(&server)
        .await;

    let user = client.users().me().await.unwrap();
    assert_eq!(user.first_name, "Test");
    assert_eq!(user.email, "test@example.com");
}

#[tokio::test]
async fn test_users_list() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/users"))
        .respond_with(json_response(json!({
            "users": [{
                "id": 1,
                "first_name": "Test",
                "last_name": "User",
                "email": "test@example.com",
                "is_active": true,
                "created_at": null,
                "updated_at": null
            }],
            "total_pages": 1,
            "page": 1,
            "total_entries": 1,
            "per_page": 100
        })))
        .mount(&server)
        .await;

    let users = client.users().list().await.unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].first_name, "Test");
}

#[tokio::test]
async fn test_users_get() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/users/1"))
        .respond_with(json_response(json!({
            "id": 1,
            "first_name": "Test",
            "last_name": "User",
            "email": "test@example.com",
            "is_active": true,
            "created_at": null,
            "updated_at": null
        })))
        .mount(&server)
        .await;

    let user = client.users().get(1).await.unwrap();
    assert_eq!(user.first_name, "Test");
}

// --- Clients API tests ---

#[tokio::test]
async fn test_clients_list() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/clients"))
        .respond_with(json_response(json!({
            "clients": [{
                "id": 1,
                "name": "Test Client",
                "is_active": true,
                "address": null,
                "currency": null,
                "created_at": null,
                "updated_at": null
            }],
            "total_pages": 1,
            "page": 1,
            "total_entries": 1,
            "per_page": 100
        })))
        .mount(&server)
        .await;

    let clients = client.clients().list().await.unwrap();
    assert_eq!(clients.len(), 1);
    assert_eq!(clients[0].name, "Test Client");
}

#[tokio::test]
async fn test_clients_get() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/clients/1"))
        .respond_with(json_response(json!({
            "id": 1,
            "name": "Test Client",
            "is_active": true,
            "address": null,
            "currency": null,
            "created_at": null,
            "updated_at": null
        })))
        .mount(&server)
        .await;

    let client_obj = client.clients().get(1).await.unwrap();
    assert_eq!(client_obj.name, "Test Client");
}

// --- Company API tests ---

#[tokio::test]
async fn test_company_get() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    Mock::given(method("GET"))
        .and(path("/company"))
        .respond_with(json_response(json!({
            "name": "Test Company"
        })))
        .mount(&server)
        .await;

    let company = client.company().get().await.unwrap();
    assert_eq!(company.name, "Test Company");
}

// --- Pagination tests ---

#[tokio::test]
async fn test_pagination_multiple_pages() {
    let server = MockServer::start().await;
    let client = HarvClient::new(test_config())
        .unwrap()
        .with_base_url(&server.uri());

    // Page 1
    Mock::given(method("GET"))
        .and(path("/projects"))
        .and(query_param("page", "1"))
        .respond_with(json_response(json!({
            "projects": [{"id": 1, "name": "Project 1", "client": null, "is_active": true, "code": null, "notes": null, "starts_on": null, "ends_on": null, "created_at": null, "updated_at": null}],
            "total_pages": 2,
            "page": 1,
            "total_entries": 2,
            "per_page": 1
        })))
        .mount(&server)
        .await;

    // Page 2
    Mock::given(method("GET"))
        .and(path("/projects"))
        .and(query_param("page", "2"))
        .respond_with(json_response(json!({
            "projects": [{"id": 2, "name": "Project 2", "client": null, "is_active": true, "code": null, "notes": null, "starts_on": null, "ends_on": null, "created_at": null, "updated_at": null}],
            "total_pages": 2,
            "page": 2,
            "total_entries": 2,
            "per_page": 1
        })))
        .mount(&server)
        .await;

    let projects = client.projects().list().await.unwrap();
    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].name, "Project 1");
    assert_eq!(projects[1].name, "Project 2");
}
