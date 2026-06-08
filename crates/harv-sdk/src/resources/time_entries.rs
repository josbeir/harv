use crate::client::HarvClient;
use harv_core::{CreateTimeEntry, TimeEntry, UpdateTimeEntry};

/// Parameters for listing time entries.
#[derive(Debug, Clone, Default)]
pub struct TimeEntryListParams {
    pub user_id: Option<u64>,
    pub project_id: Option<u64>,
    pub is_running: Option<bool>,
    pub from: Option<chrono::NaiveDate>,
    pub to: Option<chrono::NaiveDate>,
    pub page: Option<u64>,
}

impl TimeEntryListParams {
    pub(crate) fn to_query(&self) -> Vec<(&str, String)> {
        let mut params = Vec::new();
        if let Some(v) = self.user_id {
            params.push(("user_id", v.to_string()));
        }
        if let Some(v) = self.project_id {
            params.push(("project_id", v.to_string()));
        }
        if let Some(v) = self.is_running {
            params.push(("is_running", v.to_string()));
        }
        if let Some(v) = self.from {
            params.push(("from", v.format("%Y-%m-%d").to_string()));
        }
        if let Some(v) = self.to {
            params.push(("to", v.format("%Y-%m-%d").to_string()));
        }
        if let Some(v) = self.page {
            params.push(("page", v.to_string()));
        }
        params
    }
}

/// Client for the Harvest Time Entries API.
///
/// Created via [`HarvClient::time_entries`](crate::HarvClient::time_entries).
pub struct TimeEntriesApi<'c> {
    client: &'c HarvClient,
}

impl<'c> TimeEntriesApi<'c> {
    pub(crate) fn new(client: &'c HarvClient) -> Self {
        Self { client }
    }

    /// List time entries.
    ///
    /// Returns all matching time entries (handles pagination automatically).
    pub async fn list(
        &self,
        params: &TimeEntryListParams,
    ) -> Result<Vec<TimeEntry>, harv_core::HarvError> {
        let query: Vec<(&str, String)> = params.to_query();
        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();

        // Use the paginated endpoint
        let response: serde_json::Value = self.client.get("/time_entries", &query_refs).await?;

        let entries: Vec<TimeEntry> = response
            .get("time_entries")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        Ok(entries)
    }

    /// Retrieve a single time entry by ID.
    pub async fn get(&self, id: u64) -> Result<TimeEntry, harv_core::HarvError> {
        let path = format!("/time_entries/{}", id);
        self.client.get(&path, &[]).await
    }

    /// Create a new time entry.
    ///
    /// Omit `hours`, `started_time`, and `ended_time` to start a running timer.
    /// Set `hours` to a positive value to create a completed time entry.
    pub async fn create(&self, entry: &CreateTimeEntry) -> Result<TimeEntry, harv_core::HarvError> {
        self.client.post("/time_entries", entry).await
    }

    /// Update an existing time entry.
    pub async fn update(
        &self,
        id: u64,
        entry: &UpdateTimeEntry,
    ) -> Result<TimeEntry, harv_core::HarvError> {
        let path = format!("/time_entries/{}", id);
        self.client.patch(&path, entry).await
    }

    /// Delete a time entry.
    pub async fn delete(&self, id: u64) -> Result<(), harv_core::HarvError> {
        let path = format!("/time_entries/{}", id);
        self.client.delete(&path).await
    }

    /// Stop a running timer.
    pub async fn stop(&self, id: u64) -> Result<TimeEntry, harv_core::HarvError> {
        let path = format!("/time_entries/{}/stop", id);
        self.client.patch(&path, &serde_json::json!({})).await
    }

    /// Restart a stopped timer.
    pub async fn restart(&self, id: u64) -> Result<TimeEntry, harv_core::HarvError> {
        let path = format!("/time_entries/{}/restart", id);
        self.client.patch(&path, &serde_json::json!({})).await
    }

    /// Get all currently running time entries for a user.
    pub async fn running(&self, user_id: u64) -> Result<Vec<TimeEntry>, harv_core::HarvError> {
        let params = TimeEntryListParams {
            user_id: Some(user_id),
            is_running: Some(true),
            ..Default::default()
        };
        self.list(&params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_params_default() {
        let params = TimeEntryListParams::default();
        let query = params.to_query();
        assert!(query.is_empty());
    }

    #[test]
    fn test_list_params_running() {
        let params = TimeEntryListParams {
            is_running: Some(true),
            ..Default::default()
        };
        let query = params.to_query();
        assert_eq!(query.len(), 1);
        assert_eq!(query[0], ("is_running", "true".to_string()));
    }

    #[test]
    fn test_list_params_full() {
        use chrono::NaiveDate;
        let params = TimeEntryListParams {
            user_id: Some(42),
            project_id: Some(10),
            is_running: Some(false),
            from: Some(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
            to: Some(NaiveDate::from_ymd_opt(2026, 6, 8).unwrap()),
            page: Some(1),
        };
        let query = params.to_query();
        assert_eq!(query.len(), 6);
    }
}
