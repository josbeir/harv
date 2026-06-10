use harv_core::{HarvError, ProjectAssignment};
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
struct ProjectsCache {
    fetched_at: chrono::DateTime<chrono::Utc>,
    assignments: Vec<ProjectAssignment>,
}

impl ProjectsCache {
    fn is_fresh(&self, ttl_hours: u64) -> bool {
        if ttl_hours == 0 {
            return false;
        }
        let age = chrono::Utc::now() - self.fetched_at;
        age.num_hours() < ttl_hours as i64
    }

    fn path(account_id: &str) -> std::path::PathBuf {
        crate::config::HarvConfig::path()
            .parent()
            .unwrap()
            .join(format!("projects_cache_{}.json", account_id))
    }

    async fn load(account_id: &str) -> Result<Option<Self>, HarvError> {
        let path = Self::path(account_id);
        match fs::read_to_string(&path).await {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(cache) => Ok(Some(cache)),
                Err(_) => {
                    let _ = fs::remove_file(&path).await;
                    Ok(None)
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(HarvError::Io(e)),
        }
    }

    async fn save(&self, account_id: &str) -> Result<(), HarvError> {
        let path = Self::path(account_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let json =
            serde_json::to_string(self).map_err(|e| HarvError::ConfigMalformed(e.to_string()))?;
        fs::write(&path, json).await?;
        Ok(())
    }
}

pub async fn clear_cache(account_id: &str) -> Result<(), HarvError> {
    let path = ProjectsCache::path(account_id);
    match fs::remove_file(&path).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(HarvError::Io(e)),
    }
}

pub(crate) async fn get_cached_assignments(
    account_id: &str,
    ttl_hours: u64,
    force: bool,
    fetch: impl std::future::Future<Output = Result<Vec<ProjectAssignment>, HarvError>>,
) -> Result<Vec<ProjectAssignment>, HarvError> {
    #[allow(clippy::collapsible_if)]
    if !force {
        if let Ok(Some(cache)) = ProjectsCache::load(account_id).await {
            if cache.is_fresh(ttl_hours) {
                tracing::debug!(
                    "Using cached project assignments (fetched {})",
                    cache.fetched_at
                );
                return Ok(cache.assignments);
            }
        }
    }

    let assignments = fetch.await?;

    let cache = ProjectsCache {
        fetched_at: chrono::Utc::now(),
        assignments: assignments.clone(),
    };
    let _ = cache.save(account_id).await;

    Ok(assignments)
}
