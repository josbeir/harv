use crate::client::HarvClient;
use harv_core::Client;

/// Client for the Harvest Clients API.
pub struct ClientsApi<'c> {
    client: &'c HarvClient,
}

impl<'c> ClientsApi<'c> {
    pub(crate) fn new(client: &'c HarvClient) -> Self {
        Self { client }
    }

    /// List all clients.
    pub async fn list(&self) -> Result<Vec<Client>, harv_core::HarvError> {
        crate::pagination::fetch_all_pages(self.client, "/clients", &[], "clients").await
    }

    /// Retrieve a single client by ID.
    pub async fn get(&self, id: u64) -> Result<Client, harv_core::HarvError> {
        let path = format!("/clients/{}", id);
        self.client.get(&path, &[]).await
    }
}
