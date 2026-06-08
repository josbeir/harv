use crate::client::HarvClient;
use harv_core::Company;

/// Client for the Harvest Company API.
pub struct CompanyApi<'c> {
    client: &'c HarvClient,
}

impl<'c> CompanyApi<'c> {
    pub(crate) fn new(client: &'c HarvClient) -> Self {
        Self { client }
    }

    /// Retrieve the company for the authenticated user.
    pub async fn get(&self) -> Result<Company, harv_core::HarvError> {
        self.client.get("/company", &[]).await
    }
}
