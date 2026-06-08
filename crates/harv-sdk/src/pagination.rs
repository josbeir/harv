use crate::client::HarvClient;
use harv_core::HarvError;
use serde::de::DeserializeOwned;

/// Fetches all pages of a paginated Harvest API endpoint.
///
/// The `items_key` is the JSON key that holds the array of items
/// (e.g. "time_entries", "projects", "project_assignments").
#[allow(dead_code)]
pub(crate) async fn fetch_all_pages<T>(
    client: &HarvClient,
    path: &str,
    base_query: &[(&str, &str)],
    items_key: &str,
) -> Result<Vec<T>, HarvError>
where
    T: DeserializeOwned,
{
    let mut all_items: Vec<T> = Vec::new();
    let mut current_page = 1u64;

    loop {
        let mut query: Vec<(&str, &str)> = base_query.to_vec();
        let page_str = current_page.to_string();
        query.push(("page", &page_str));
        if !base_query.iter().any(|(k, _)| *k == "per_page") {
            query.push(("per_page", "100"));
        }

        let response: serde_json::Value = client.get(path, &query).await?;

        let items: Vec<T> = response
            .get(items_key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        all_items.extend(items);

        let total_pages = response
            .get("total_pages")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);

        if current_page >= total_pages {
            break;
        }
        current_page += 1;
    }

    Ok(all_items)
}
