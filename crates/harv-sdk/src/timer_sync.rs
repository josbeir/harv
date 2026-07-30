//! Cross-process coordination for the TUI's running-timer polling.
//!
//! One process holds an advisory leader lock and polls Harvest. All other
//! processes consume its atomically-written snapshot, which keeps a shared
//! Harvest account well below the API rate limit.

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, Utc};
use fs4::fs_std::FileExt;
use harv_core::{HarvError, TimeEntry};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::HarvClient;

const POLL_INTERVAL: Duration = Duration::from_secs(15);
const FOLLOWER_REFRESH_INTERVAL: Duration = Duration::from_secs(1);
const DEFAULT_RETRY_AFTER_SECS: u64 = 60;
const MAX_RETRY_AFTER_SECS: u64 = 60 * 60;
const SNAPSHOT_VERSION: u8 = 1;

/// Keeps the process-local polling worker alive.
///
/// Dropping this value aborts the worker and releases its leader lock if it
/// owns one, allowing another Harv window to become the poller.
pub struct TimerPoller {
    task: tokio::task::JoinHandle<()>,
}

/// A timer polling update delivered to the TUI.
#[derive(Debug)]
pub enum TimerPollUpdate {
    /// The current running time entries.
    Entries(Vec<TimeEntry>),
    /// A recoverable polling failure suitable for display to the user.
    Error(HarvError),
}

impl TimerPoller {
    /// Starts a worker that either polls Harvest as the elected leader or
    /// follows a snapshot written by another local Harv process.
    pub fn start(
        client: HarvClient,
        user_id: u64,
        updates: UnboundedSender<TimerPollUpdate>,
    ) -> Self {
        Self::start_in_directory(client, user_id, updates, state_directory())
    }

    /// Starts a poller in a caller-provided state directory.
    ///
    /// This is public for integration tests that need isolated, independent
    /// processes. Application code should use [`Self::start`].
    #[doc(hidden)]
    pub fn start_in_directory(
        client: HarvClient,
        user_id: u64,
        updates: UnboundedSender<TimerPollUpdate>,
        directory: PathBuf,
    ) -> Self {
        let paths = PollPaths::new(client.config().account_id(), user_id, directory);
        let task = tokio::spawn(run_poller(client, paths, updates));
        Self { task }
    }
}

impl Drop for TimerPoller {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[derive(Debug, Clone)]
struct PollPaths {
    directory: PathBuf,
    leader_lock: PathBuf,
    snapshot: PathBuf,
    user_id: u64,
}

impl PollPaths {
    fn new(account_id: &str, user_id: u64, directory: PathBuf) -> Self {
        let scope = format!(
            "timer-v{}-{}-{}",
            SNAPSHOT_VERSION,
            encode_path_component(account_id),
            user_id
        );
        Self {
            leader_lock: directory.join(format!("{scope}.lock")),
            snapshot: directory.join(format!("{scope}.mp")),
            directory,
            user_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimerSnapshot {
    version: u8,
    user_id: u64,
    sequence: u64,
    fetched_at: DateTime<Utc>,
    entries: Vec<TimeEntry>,
    cooldown_until: Option<DateTime<Utc>>,
}

impl TimerSnapshot {
    fn empty(user_id: u64) -> Self {
        Self {
            version: SNAPSHOT_VERSION,
            user_id,
            sequence: 0,
            fetched_at: Utc::now(),
            entries: Vec::new(),
            cooldown_until: None,
        }
    }
}

async fn run_poller(
    client: HarvClient,
    paths: PollPaths,
    updates: UnboundedSender<TimerPollUpdate>,
) {
    if let Err(error) = tokio::fs::create_dir_all(&paths.directory).await {
        tracing::warn!("Unable to create shared timer state directory: {error}");
        let _ = updates.send(TimerPollUpdate::Error(error.into()));
        return;
    }

    let mut delivered_sequence = 0;
    loop {
        match try_acquire_leader(&paths.leader_lock).await {
            Ok(Some(leader_lock)) => {
                run_leader(client, paths, updates, leader_lock).await;
                return;
            }
            Ok(None) => {
                if let Some(snapshot) = load_snapshot(&paths.snapshot).await
                    && snapshot.sequence != delivered_sequence
                {
                    delivered_sequence = snapshot.sequence;
                    let _ = updates.send(TimerPollUpdate::Entries(snapshot.entries));
                }
            }
            Err(error) => {
                tracing::warn!("Unable to acquire shared timer poll lock: {error}");
                let _ = updates.send(TimerPollUpdate::Error(error));
            }
        }

        tokio::time::sleep(FOLLOWER_REFRESH_INTERVAL).await;
    }
}

async fn run_leader(
    client: HarvClient,
    paths: PollPaths,
    updates: UnboundedSender<TimerPollUpdate>,
    _leader_lock: File,
) {
    let mut snapshot = load_snapshot(&paths.snapshot)
        .await
        .filter(|snapshot| {
            snapshot.version == SNAPSHOT_VERSION && snapshot.user_id == paths.user_id
        })
        .unwrap_or_else(|| TimerSnapshot::empty(paths.user_id));

    let user_id = paths.user_id;

    loop {
        if let Some(delay) = snapshot.cooldown_until.and_then(duration_until) {
            tokio::time::sleep(delay).await;
            snapshot.cooldown_until = None;
        }

        match client.time_entries().running(user_id).await {
            Ok(entries) => {
                snapshot.sequence = snapshot.sequence.wrapping_add(1).max(1);
                snapshot.fetched_at = Utc::now();
                snapshot.entries = entries.clone();
                snapshot.cooldown_until = None;
                if let Err(error) = save_snapshot(&paths.snapshot, &snapshot).await {
                    tracing::warn!("Unable to save shared timer snapshot: {error}");
                }
                let _ = updates.send(TimerPollUpdate::Entries(entries));
                tokio::time::sleep(POLL_INTERVAL).await;
            }
            Err(error @ HarvError::RateLimited { retry_after_secs }) => {
                let retry_after_secs = retry_after_secs
                    .unwrap_or(DEFAULT_RETRY_AFTER_SECS)
                    .clamp(1, MAX_RETRY_AFTER_SECS);
                snapshot.cooldown_until =
                    Some(Utc::now() + chrono::Duration::seconds(retry_after_secs as i64));
                if let Err(error) = save_snapshot(&paths.snapshot, &snapshot).await {
                    tracing::warn!("Unable to save shared timer cooldown: {error}");
                }
                tracing::warn!(
                    "Harvest timer polling rate limited; retrying in {retry_after_secs} seconds"
                );
                let _ = updates.send(TimerPollUpdate::Error(error));
            }
            Err(error) => {
                tracing::warn!("Harvest timer polling failed: {error}");
                let _ = updates.send(TimerPollUpdate::Error(error));
                tokio::time::sleep(POLL_INTERVAL).await;
            }
        }
    }
}

fn state_directory() -> PathBuf {
    dirs::cache_dir()
        .map(|path| path.join("harv"))
        .unwrap_or_else(|| {
            crate::config::HarvConfig::path()
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("harv"))
        })
}

fn encode_path_component(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn duration_until(time: DateTime<Utc>) -> Option<Duration> {
    (time - Utc::now()).to_std().ok()
}

async fn try_acquire_leader(path: &Path) -> Result<Option<File>, HarvError> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(path)?;
        match file.try_lock_exclusive() {
            Ok(true) => Ok(Some(file)),
            Ok(false) => Ok(None),
            Err(error) => Err(HarvError::Io(error)),
        }
    })
    .await
    .map_err(|error| HarvError::Other(format!("Timer lock task failed: {error}")))?
}

async fn load_snapshot(path: &Path) -> Option<TimerSnapshot> {
    let bytes = tokio::fs::read(path).await.ok()?;
    rmp_serde::from_slice(&bytes).ok()
}

async fn save_snapshot(path: &Path, snapshot: &TimerSnapshot) -> Result<(), HarvError> {
    let bytes = rmp_serde::to_vec_named(snapshot)
        .map_err(|error| HarvError::Other(format!("Unable to encode timer snapshot: {error}")))?;
    crate::storage::atomic_write(path, bytes).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_component_is_safe_and_deterministic() {
        assert_eq!(encode_path_component("abc/123"), "6162632f313233");
    }

    #[test]
    fn expired_cooldown_has_no_delay() {
        assert!(duration_until(Utc::now() - chrono::Duration::seconds(1)).is_none());
    }

    #[test]
    fn poll_paths_do_not_use_raw_account_ids() {
        let paths = PollPaths::new("account/name", 42, PathBuf::from("state"));
        assert!(!paths.leader_lock.to_string_lossy().contains("account/name"));
        assert!(
            paths
                .leader_lock
                .to_string_lossy()
                .contains("6163636f756e742f6e616d65")
        );
    }
}
