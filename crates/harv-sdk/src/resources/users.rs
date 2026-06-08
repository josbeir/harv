use crate::client::HarvClient;
use harv_core::User;

/// Client for the Harvest Users API.
pub struct UsersApi<'c> {
    client: &'c HarvClient,
}

impl<'c> UsersApi<'c> {
    pub(crate) fn new(client: &'c HarvClient) -> Self {
        Self { client }
    }

    /// Retrieve the currently authenticated user.
    pub async fn me(&self) -> Result<User, harv_core::HarvError> {
        self.client.get("/users/me", &[]).await
    }

    /// List all users.
    pub async fn list(&self) -> Result<Vec<User>, harv_core::HarvError> {
        crate::pagination::fetch_all_pages(self.client, "/users", &[], "users").await
    }

    /// Retrieve a single user by ID.
    pub async fn get(&self, id: u64) -> Result<User, harv_core::HarvError> {
        let path = format!("/users/{}", id);
        self.client.get(&path, &[]).await
    }
}
