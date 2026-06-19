use std::collections::BTreeMap;

use crate::client::HarvClient;
use harv_core::HarvError;
use serde::de::DeserializeOwned;
use tokio::task::JoinSet;

/// Maximum concurrent page requests for paginated endpoints.
const MAX_CONCURRENT_PAGES: usize = 3;

/// Fetch a single page and return `(items, total_pages)`.
async fn fetch_one_page<T: DeserializeOwned>(
    client: &HarvClient,
    path: &str,
    base_query: &[(&str, &str)],
    items_key: &str,
    page: u64,
) -> Result<(Vec<T>, u64), HarvError> {
    let mut query: Vec<(&str, &str)> = base_query.to_vec();
    let page_str = page.to_string();
    query.push(("page", &page_str));
    if !base_query.iter().any(|(k, _)| *k == "per_page") {
        query.push(("per_page", "100"));
    }

    let response: serde_json::Value = client.get(path, &query).await?;

    let items: Vec<T> = response
        .get(items_key)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let total_pages = response
        .get("total_pages")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);

    Ok((items, total_pages))
}

/// Owned version used inside tokio::spawn.
async fn fetch_one_page_owned<T: DeserializeOwned + Send + 'static>(
    client: HarvClient,
    path: String,
    base_query: Vec<(String, String)>,
    items_key: String,
    page: u64,
) -> Result<(Vec<T>, u64), HarvError> {
    let mut query: Vec<(String, String)> = base_query;
    query.push(("page".into(), page.to_string()));
    if !query.iter().any(|(k, _)| k == "per_page") {
        query.push(("per_page".into(), "100".into()));
    }
    let query_refs: Vec<(&str, &str)> = query
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let response: serde_json::Value = client.get(&path, &query_refs).await?;

    let items: Vec<T> = response
        .get(&items_key)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let total_pages = response
        .get("total_pages")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);

    Ok((items, total_pages))
}

/// Fetches all pages of a paginated Harvest API endpoint.
///
/// The `items_key` is the JSON key that holds the array of items
/// (e.g. "time_entries", "projects", "project_assignments").
///
/// Pages beyond the first are fetched concurrently with up to
/// `MAX_CONCURRENT_PAGES` in-flight requests at any time.
/// Results are always returned in page order.
pub(crate) async fn fetch_all_pages<T>(
    client: &HarvClient,
    path: &str,
    base_query: &[(&str, &str)],
    items_key: &str,
) -> Result<Vec<T>, HarvError>
where
    T: DeserializeOwned + Send + 'static,
{
    // Fetch page 1 to discover total_pages and get the first batch.
    let (page1_items, total_pages) =
        fetch_one_page::<T>(client, path, base_query, items_key, 1).await?;

    let mut all_items = page1_items;

    if total_pages > 1 {
        let client = client.clone();
        let path = path.to_string();
        let base_query: Vec<(String, String)> = base_query
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let items_key = items_key.to_string();

        let mut set = JoinSet::new();
        let mut page_iter = (2..=total_pages).peekable();
        let mut page_results: BTreeMap<u64, Vec<T>> = BTreeMap::new();

        // Fill the initial batch.
        for _ in 0..MAX_CONCURRENT_PAGES {
            if let Some(&page) = page_iter.peek() {
                page_iter.next();
                let c = client.clone();
                let p = path.clone();
                let q = base_query.clone();
                let ik = items_key.clone();
                set.spawn(async move {
                    let result = fetch_one_page_owned::<T>(c, p, q, ik, page).await;
                    (page, result)
                });
            }
        }

        // As each task completes, spawn the next page.
        while let Some(task_result) = set.join_next().await {
            let (page, page_result) = task_result.map_err(|e| HarvError::Http(e.to_string()))?;
            let (page_items, _) = page_result?;
            page_results.insert(page, page_items);

            if let Some(&next_page) = page_iter.peek() {
                page_iter.next();
                let c = client.clone();
                let p = path.clone();
                let q = base_query.clone();
                let ik = items_key.clone();
                set.spawn(async move {
                    let result = fetch_one_page_owned::<T>(c, p, q, ik, next_page).await;
                    (next_page, result)
                });
            }
        }

        // Extend in page order
        for items in page_results.into_values() {
            all_items.extend(items);
        }
    }

    Ok(all_items)
}
