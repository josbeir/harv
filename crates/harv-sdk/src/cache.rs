use std::collections::HashMap;

use harv_core::{HarvError, ProjectAssignment, Reference, TaskAssignment};
use serde::{Deserialize, Serialize};
use tokio::fs;

/// Shared task definition stored once in the cache and referenced by index.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TaskDefinition {
    id: u64,
    name: String,
}

/// Per-assignment task data referencing the shared task list.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompactTaskEntry {
    task_idx: u32,
    billable: bool,
    hourly_rate: Option<f64>,
    is_active: bool,
    budget: Option<f64>,
}

/// Lighter assignment that references tasks by index into the shared list.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompactAssignment {
    id: u64,
    project_id: u64,
    project_name: String,
    #[serde(default)]
    project_code: Option<String>,
    client_id: Option<u64>,
    client_name: Option<String>,
    task_entries: Vec<CompactTaskEntry>,
    #[serde(default)]
    is_active: bool,
}

/// Compact, deduplicated cache format using MessagePack binary serialization.
#[derive(Debug, Serialize, Deserialize)]
struct ProjectsCache {
    fetched_at: chrono::DateTime<chrono::Utc>,
    tasks: Vec<TaskDefinition>,
    assignments: Vec<CompactAssignment>,
    total_projects: usize,
}

// ── Conversion: Vec<ProjectAssignment> ↔ ProjectsCache ──────────────

impl ProjectsCache {
    fn from_assignments(
        assignments: &[ProjectAssignment],
        total_projects: usize,
        fetched_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        // Build a deduplicated task list. Since multiple projects share the
        // same task definitions, this removes the bulk of the redundant data.
        let mut task_map: HashMap<u64, u32> = HashMap::new();
        let mut tasks: Vec<TaskDefinition> = Vec::new();

        for a in assignments {
            for ta in &a.task_assignments {
                task_map.entry(ta.task.id).or_insert_with(|| {
                    let idx = tasks.len() as u32;
                    tasks.push(TaskDefinition {
                        id: ta.task.id,
                        name: ta.task.name.clone(),
                    });
                    idx
                });
            }
        }

        let compact: Vec<CompactAssignment> = assignments
            .iter()
            .map(|a| CompactAssignment {
                id: a.id,
                project_id: a.project.id,
                project_name: a.project.name.clone(),
                project_code: a.project_code.clone(),
                client_id: a.client.as_ref().map(|c| c.id),
                client_name: a.client.as_ref().map(|c| c.name.clone()),
                task_entries: a
                    .task_assignments
                    .iter()
                    .map(|ta| CompactTaskEntry {
                        task_idx: task_map[&ta.task.id],
                        billable: ta.billable,
                        hourly_rate: ta.hourly_rate,
                        is_active: ta.is_active,
                        budget: ta.budget,
                    })
                    .collect(),
                is_active: a.is_active,
            })
            .collect();

        Self {
            fetched_at,
            tasks,
            assignments: compact,
            total_projects,
        }
    }

    fn into_assignments(self) -> Vec<ProjectAssignment> {
        self.assignments
            .into_iter()
            .map(|a| {
                let project = Reference {
                    id: a.project_id,
                    name: a.project_name,
                };
                let client = a
                    .client_id
                    .zip(a.client_name)
                    .map(|(id, name)| Reference { id, name });

                let task_assignments: Vec<TaskAssignment> = a
                    .task_entries
                    .into_iter()
                    .map(|te| {
                        let task_def = &self.tasks[te.task_idx as usize];
                        TaskAssignment {
                            // task_assignment ID is never accessed by the
                            // application — only task.id is used.
                            id: 0,
                            task: Reference {
                                id: task_def.id,
                                name: task_def.name.clone(),
                            },
                            billable: te.billable,
                            hourly_rate: te.hourly_rate,
                            is_active: te.is_active,
                            budget: te.budget,
                        }
                    })
                    .collect();

                ProjectAssignment {
                    id: a.id,
                    project,
                    project_code: a.project_code,
                    client,
                    task_assignments,
                    is_active: a.is_active,
                }
            })
            .collect()
    }
}

// ── Freshness ───────────────────────────────────────────────────────

impl ProjectsCache {
    fn is_fresh(&self, ttl_hours: u64) -> bool {
        if ttl_hours == 0 {
            return false;
        }
        let age = chrono::Utc::now() - self.fetched_at;
        age.num_hours() < ttl_hours as i64
    }
}

// ── File I/O ────────────────────────────────────────────────────────

impl ProjectsCache {
    /// Path for the new binary-format cache.
    fn path(account_id: &str) -> std::path::PathBuf {
        crate::config::HarvConfig::path()
            .parent()
            .unwrap()
            .join(format!("projects_cache_{}.mp", account_id))
    }

    /// Path for the legacy JSON-format cache (used during migration only).
    fn legacy_json_path(account_id: &str) -> std::path::PathBuf {
        crate::config::HarvConfig::path()
            .parent()
            .unwrap()
            .join(format!("projects_cache_{}.json", account_id))
    }

    /// Remove the legacy JSON cache file if it exists.
    async fn remove_legacy(account_id: &str) {
        let legacy = Self::legacy_json_path(account_id);
        let _ = fs::remove_file(&legacy).await;
    }

    async fn load(account_id: &str) -> Result<Option<Self>, HarvError> {
        let path = Self::path(account_id);
        match fs::read(&path).await {
            Ok(bytes) => match rmp_serde::from_slice::<Self>(&bytes) {
                Ok(cache) => {
                    if cache.is_valid() {
                        Self::remove_legacy(account_id).await;
                        Ok(Some(cache))
                    } else {
                        tracing::warn!("Corrupted cache, removing");
                        let _ = fs::remove_file(&path).await;
                        Ok(None)
                    }
                }
                Err(_) => {
                    let _ = fs::remove_file(&path).await;
                    Ok(None)
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(HarvError::Io(e)),
        }
    }

    fn is_valid(&self) -> bool {
        for a in &self.assignments {
            for te in &a.task_entries {
                if te.task_idx as usize >= self.tasks.len() {
                    return false;
                }
            }
        }
        true
    }

    async fn save(&self, account_id: &str) -> Result<(), HarvError> {
        let path = Self::path(account_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let bytes =
            rmp_serde::to_vec_named(self).map_err(|e| HarvError::ConfigMalformed(e.to_string()))?;
        fs::write(&path, bytes).await?;
        // Clean up legacy JSON after successful save
        Self::remove_legacy(account_id).await;
        Ok(())
    }
}

/// Clear both current and legacy cache files.
pub async fn clear_cache(account_id: &str) -> Result<(), HarvError> {
    let paths = [
        ProjectsCache::path(account_id),
        ProjectsCache::legacy_json_path(account_id),
    ];
    for path in &paths {
        match fs::remove_file(path).await {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(HarvError::Io(e)),
        }
    }
    Ok(())
}

/// Fetch or load cached project assignments.
///
/// On a cache miss (or `force = true`), the `fetch` future is invoked and
/// the result is stored in a compact, deduplicated binary cache. On a cache
/// hit the raw `ProjectAssignment` values are reconstructed on load.
/// Returns `(assignments, total_projects_count)`.
pub(crate) async fn get_cached_assignments(
    account_id: &str,
    ttl_hours: u64,
    force: bool,
    fetch: impl std::future::Future<Output = Result<(Vec<ProjectAssignment>, usize), HarvError>>,
) -> Result<(Vec<ProjectAssignment>, usize), HarvError> {
    #[allow(clippy::collapsible_if)]
    if !force {
        if let Ok(Some(cache)) = ProjectsCache::load(account_id).await {
            if cache.is_fresh(ttl_hours) {
                tracing::debug!(
                    "Using cached project assignments (fetched {})",
                    cache.fetched_at
                );
                let total_projects = cache.total_projects;
                let assignments = cache.into_assignments();
                return Ok((assignments, total_projects));
            }
        }
    }

    let (assignments, total_projects) = fetch.await?;

    let cache = ProjectsCache::from_assignments(&assignments, total_projects, chrono::Utc::now());
    let _ = cache.save(account_id).await;

    Ok((assignments, total_projects))
}

#[cfg(test)]
mod tests {
    use super::*;
    use harv_core::Reference;

    fn sample_ta(id: u64, task_id: u64, task_name: &str) -> TaskAssignment {
        TaskAssignment {
            id,
            task: Reference {
                id: task_id,
                name: task_name.into(),
            },
            billable: true,
            hourly_rate: Some(150.0),
            is_active: true,
            budget: None,
        }
    }

    fn sample_assignment(
        id: u64,
        project_id: u64,
        project_name: &str,
        code: Option<&str>,
        tasks: Vec<TaskAssignment>,
    ) -> ProjectAssignment {
        ProjectAssignment {
            id,
            project: Reference {
                id: project_id,
                name: project_name.into(),
            },
            project_code: code.map(String::from),
            client: Some(Reference {
                id: 1,
                name: "Client".into(),
            }),
            task_assignments: tasks,
            is_active: true,
        }
    }

    #[test]
    fn test_roundtrip_preserves_all_fields() {
        let tasks = vec![
            sample_ta(10, 200, "Development"),
            sample_ta(11, 201, "Design"),
        ];
        let original = vec![sample_assignment(
            1,
            100,
            "Test Project",
            Some("TEST"),
            tasks,
        )];

        let now = chrono::Utc::now();
        let cache = ProjectsCache::from_assignments(&original, 0, now);
        let recovered = cache.into_assignments();

        assert_eq!(original.len(), recovered.len());
        assert_eq!(original[0].id, recovered[0].id);
        assert_eq!(original[0].project, recovered[0].project);
        assert_eq!(original[0].project_code, recovered[0].project_code);
        assert_eq!(original[0].client, recovered[0].client);
        assert_eq!(original[0].is_active, recovered[0].is_active);
        assert_eq!(
            original[0].task_assignments.len(),
            recovered[0].task_assignments.len()
        );
        assert_eq!(
            original[0].task_assignments[0].task,
            recovered[0].task_assignments[0].task
        );
        assert_eq!(
            original[0].task_assignments[0].billable,
            recovered[0].task_assignments[0].billable
        );
        assert_eq!(
            original[0].task_assignments[0].hourly_rate,
            recovered[0].task_assignments[0].hourly_rate
        );
        assert_eq!(
            original[0].task_assignments[0].is_active,
            recovered[0].task_assignments[0].is_active
        );
        assert_eq!(
            original[0].task_assignments[0].budget,
            recovered[0].task_assignments[0].budget
        );
    }

    #[test]
    fn test_roundtrip_no_client() {
        let mut a = sample_assignment(1, 100, "No Client", None, vec![]);
        a.client = None;
        let original = vec![a];
        let now = chrono::Utc::now();
        let cache = ProjectsCache::from_assignments(&original, 0, now);
        let recovered = cache.into_assignments();
        assert!(recovered[0].client.is_none());
    }

    #[test]
    fn test_roundtrip_no_code() {
        let a = sample_assignment(1, 100, "No Code", None, vec![]);
        let original = vec![a];
        let now = chrono::Utc::now();
        let cache = ProjectsCache::from_assignments(&original, 0, now);
        let recovered = cache.into_assignments();
        assert_eq!(recovered[0].project_code, None);
    }

    #[test]
    fn test_deduplication_same_task_shared_across_projects() {
        // Two assignments using the same task — only one TaskDefinition stored.
        let dev = sample_ta(10, 200, "Development");
        let a1 = sample_assignment(1, 100, "Project A", None, vec![dev.clone()]);
        let a2 = sample_assignment(2, 101, "Project B", None, vec![dev]);

        let original = vec![a1, a2];
        let now = chrono::Utc::now();
        let cache = ProjectsCache::from_assignments(&original, 0, now);

        // Only 1 unique task across 2 projects
        assert_eq!(cache.tasks.len(), 1);
        assert_eq!(cache.tasks[0].id, 200);
        assert_eq!(cache.tasks[0].name, "Development");
    }

    #[test]
    fn test_deduplication_multiple_tasks() {
        // 3 assignments with 5 task instances but only 3 unique tasks
        let dev = sample_ta(10, 200, "Development");
        let design = sample_ta(11, 201, "Design");
        let pm = sample_ta(12, 202, "Project Management");

        let a1 = sample_assignment(1, 100, "Project A", None, vec![dev.clone(), design.clone()]);
        let a2 = sample_assignment(2, 101, "Project B", None, vec![dev.clone(), pm.clone()]);
        let a3 = sample_assignment(3, 102, "Project C", None, vec![design.clone(), pm.clone()]);

        let original = vec![a1, a2, a3];
        let now = chrono::Utc::now();
        let cache = ProjectsCache::from_assignments(&original, 0, now);

        assert_eq!(cache.tasks.len(), 3);
    }

    #[test]
    fn test_empty_assignments() {
        let cache = ProjectsCache::from_assignments(&[], 0, chrono::Utc::now());
        assert!(cache.tasks.is_empty());
        assert!(cache.assignments.is_empty());

        let recovered = cache.into_assignments();
        assert!(recovered.is_empty());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let tasks = vec![
            sample_ta(10, 200, "Development"),
            sample_ta(11, 201, "Design"),
        ];
        let original = vec![sample_assignment(
            1,
            100,
            "Test Project",
            Some("TEST"),
            tasks,
        )];

        let now = chrono::Utc::now();
        let cache = ProjectsCache::from_assignments(&original, 0, now);

        // Serialize to bytes and back
        let bytes = rmp_serde::to_vec_named(&cache).unwrap();
        let deserialized: ProjectsCache = rmp_serde::from_slice(&bytes).unwrap();

        // Same number of tasks and assignments
        assert_eq!(deserialized.tasks.len(), cache.tasks.len());
        assert_eq!(deserialized.assignments.len(), cache.assignments.len());

        // Data matches
        let recovered = deserialized.into_assignments();
        assert_eq!(recovered[0].project.name, "Test Project");
        assert_eq!(recovered[0].project_code.as_deref(), Some("TEST"));
        assert_eq!(recovered[0].task_assignments.len(), 2);
        assert_eq!(recovered[0].task_assignments[0].task.name, "Development");
    }

    #[test]
    fn test_is_fresh_within_ttl() {
        let cache = ProjectsCache::from_assignments(&[], 0, chrono::Utc::now());
        assert!(cache.is_fresh(24));
    }

    #[test]
    fn test_is_fresh_stale() {
        let stale = chrono::Utc::now() - chrono::Duration::hours(25);
        let cache = ProjectsCache::from_assignments(&[], 0, stale);
        assert!(!cache.is_fresh(24));
    }

    #[test]
    fn test_is_fresh_ttl_zero() {
        let cache = ProjectsCache::from_assignments(&[], 0, chrono::Utc::now());
        assert!(!cache.is_fresh(0));
    }

    #[test]
    fn test_task_definition_partial_eq() {
        let a = TaskDefinition {
            id: 1,
            name: "Dev".into(),
        };
        let b = TaskDefinition {
            id: 1,
            name: "Dev".into(),
        };
        assert_eq!(a, b);

        let c = TaskDefinition {
            id: 2,
            name: "Dev".into(),
        };
        assert_ne!(a, c);
    }

    #[test]
    fn test_is_valid_clean_cache() {
        let dev = sample_ta(10, 200, "Development");
        let a = sample_assignment(1, 100, "Project A", None, vec![dev]);
        let cache = ProjectsCache::from_assignments(&[a], 10, chrono::Utc::now());
        assert!(cache.is_valid());
    }

    #[test]
    fn test_is_valid_out_of_range_task_idx() {
        // Build a cache, then corrupt a task_idx
        let dev = sample_ta(10, 200, "Development");
        let a = sample_assignment(1, 100, "Project A", None, vec![dev]);
        let mut cache = ProjectsCache::from_assignments(&[a], 10, chrono::Utc::now());
        // Corrupt: set task_idx beyond the task list
        cache.assignments[0].task_entries[0].task_idx = 999;
        assert!(!cache.is_valid());
    }

    #[test]
    fn test_is_valid_empty_cache() {
        let cache = ProjectsCache::from_assignments(&[], 0, chrono::Utc::now());
        assert!(cache.is_valid());
    }
}
