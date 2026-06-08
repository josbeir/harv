use crate::client::HarvClient;
use harv_core::Task;

/// Client for the Harvest Tasks API.
pub struct TasksApi<'c> {
    client: &'c HarvClient,
}

impl<'c> TasksApi<'c> {
    pub(crate) fn new(client: &'c HarvClient) -> Self {
        Self { client }
    }

    /// List all tasks.
    pub async fn list(&self) -> Result<Vec<Task>, harv_core::HarvError> {
        crate::pagination::fetch_all_pages(self.client, "/tasks", &[], "tasks").await
    }

    /// Retrieve a single task by ID.
    pub async fn get(&self, id: u64) -> Result<Task, harv_core::HarvError> {
        let path = format!("/tasks/{}", id);
        self.client.get(&path, &[]).await
    }
}
