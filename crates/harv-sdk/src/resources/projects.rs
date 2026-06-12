use crate::client::HarvClient;
use harv_core::{Project, ProjectAssignment, TaskAssignment};

/// Client for the Harvest Projects API.
pub struct ProjectsApi<'c> {
    client: &'c HarvClient,
}

impl<'c> ProjectsApi<'c> {
    pub(crate) fn new(client: &'c HarvClient) -> Self {
        Self { client }
    }

    /// List all projects.
    pub async fn list(&self) -> Result<Vec<Project>, harv_core::HarvError> {
        crate::pagination::fetch_all_pages(self.client, "/projects", &[], "projects").await
    }

    /// Retrieve a single project by ID.
    pub async fn get(&self, id: u64) -> Result<Project, harv_core::HarvError> {
        let path = format!("/projects/{}", id);
        self.client.get(&path, &[]).await
    }

    /// List the authenticated user's project assignments.
    pub async fn my_assignments(
        &self,
        force: bool,
    ) -> Result<Vec<ProjectAssignment>, harv_core::HarvError> {
        let account_id = self.client.config().account_id.clone();
        let ttl = self.client.config().cache_ttl_hours;
        crate::cache::get_cached_assignments(&account_id, ttl, force, async {
            let mut assignments: Vec<ProjectAssignment> =
                crate::pagination::fetch_all_pages(
                    self.client,
                    "/users/me/project_assignments",
                    &[],
                    "project_assignments",
                )
                .await?;

            let projects: Vec<Project> = self.list().await?;
            let code_map: std::collections::HashMap<u64, Option<String>> = projects
                .into_iter()
                .map(|p| (p.id, p.code))
                .collect();

            for a in &mut assignments {
                a.project_code = code_map
                    .get(&a.project.id)
                    .and_then(|c| c.clone());
            }

            Ok(assignments)
        })
        .await
    }

    /// List task assignments for a project.
    pub async fn task_assignments(
        &self,
        project_id: u64,
    ) -> Result<Vec<TaskAssignment>, harv_core::HarvError> {
        crate::pagination::fetch_all_pages(
            self.client,
            &format!("/projects/{}/task_assignments", project_id),
            &[],
            "task_assignments",
        )
        .await
    }
}
