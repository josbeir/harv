use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use harv_core::HarvError;
use harv_sdk::mock_data;
use harv_sdk::{HarvClient, TimerPollUpdate, TimerPoller};
use tokio::sync::mpsc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SERVER_ENV: &str = "HARV_TIMER_SYNC_TEST_SERVER";
const STATE_ENV: &str = "HARV_TIMER_SYNC_TEST_STATE";
const MODE_ENV: &str = "HARV_TIMER_SYNC_TEST_MODE";

fn worker_state_directory() -> Option<PathBuf> {
    std::env::var_os(STATE_ENV).map(PathBuf::from)
}

#[tokio::test]
async fn timer_sync_worker() {
    let Some(server) = std::env::var_os(SERVER_ENV) else {
        return;
    };
    let state_directory = worker_state_directory().expect("worker state directory is set");
    let mode = std::env::var(MODE_ENV).expect("worker mode is set");
    let client = HarvClient::new(mock_data::test_config())
        .expect("test client")
        .with_base_url(&server.to_string_lossy());
    let (updates, mut rx) = mpsc::unbounded_channel();
    let _poller = TimerPoller::start_in_directory(client, 1, updates, state_directory);

    if mode == "success" || mode == "takeover" {
        let update = tokio::time::timeout(Duration::from_secs(3), rx.recv())
            .await
            .expect("shared timer update arrived")
            .expect("poller sender remains connected");
        let TimerPollUpdate::Entries(entries) = update else {
            panic!("expected shared timer entries");
        };
        assert_eq!(entries.len(), 1);

        if mode == "success" {
            // Keep the elected leader alive until followers have consumed its
            // snapshot; otherwise a follower could legitimately take over.
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    } else {
        let update = tokio::time::timeout(Duration::from_secs(3), rx.recv())
            .await
            .expect("rate-limit error arrived")
            .expect("poller sender remains connected");
        assert!(matches!(
            update,
            TimerPollUpdate::Error(HarvError::RateLimited {
                retry_after_secs: Some(2)
            })
        ));
        tokio::time::sleep(Duration::from_millis(500)).await;
        assert!(rx.try_recv().is_err());
    }
}

fn worker_command(server: &MockServer, state_directory: &std::path::Path, mode: &str) -> Command {
    let mut command = Command::new(std::env::current_exe().expect("test executable"));
    command
        .args(["--exact", "timer_sync_worker", "--nocapture"])
        .env(SERVER_ENV, server.uri())
        .env(STATE_ENV, state_directory)
        .env(MODE_ENV, mode);
    command
}

async fn wait_for_worker(mut worker: std::process::Child) -> std::process::ExitStatus {
    tokio::task::spawn_blocking(move || worker.wait().expect("worker exits"))
        .await
        .expect("worker wait task completes")
}

#[tokio::test]
async fn separate_processes_share_one_timer_poll() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/time_entries"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(mock_data::paginated(
                "time_entries",
                vec![mock_data::running_timer_json()],
            )),
        )
        .mount(&server)
        .await;

    let state_directory = tempfile::tempdir().expect("temporary state directory");
    let first = worker_command(&server, state_directory.path(), "success")
        .spawn()
        .expect("first worker starts");
    tokio::time::sleep(Duration::from_millis(100)).await;
    let second = worker_command(&server, state_directory.path(), "success")
        .spawn()
        .expect("second worker starts");

    assert!(wait_for_worker(first).await.success());
    assert!(wait_for_worker(second).await.success());

    let requests = server
        .received_requests()
        .await
        .expect("server recorded requests");
    assert_eq!(requests.len(), 1, "only the elected leader polls Harvest");
}

#[tokio::test]
async fn timer_poller_honors_shared_rate_limit_cooldown() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/time_entries"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "2"))
        .mount(&server)
        .await;

    let state_directory = tempfile::tempdir().expect("temporary state directory");
    let worker = worker_command(&server, state_directory.path(), "rate_limited")
        .spawn()
        .expect("worker starts");
    assert!(wait_for_worker(worker).await.success());

    let requests = server
        .received_requests()
        .await
        .expect("server recorded requests");
    assert_eq!(requests.len(), 1, "Retry-After prevents a rapid retry");
}

#[tokio::test]
async fn successor_polls_after_the_leader_exits() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/time_entries"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(mock_data::paginated(
                "time_entries",
                vec![mock_data::running_timer_json()],
            )),
        )
        .mount(&server)
        .await;

    let state_directory = tempfile::tempdir().expect("temporary state directory");
    let first = worker_command(&server, state_directory.path(), "takeover")
        .spawn()
        .expect("first worker starts");
    assert!(wait_for_worker(first).await.success());

    let successor = worker_command(&server, state_directory.path(), "takeover")
        .spawn()
        .expect("successor starts");
    assert!(wait_for_worker(successor).await.success());

    let requests = server
        .received_requests()
        .await
        .expect("server recorded requests");
    assert_eq!(requests.len(), 2, "successor becomes the new poll leader");
}
