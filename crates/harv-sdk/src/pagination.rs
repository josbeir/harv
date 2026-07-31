use std::collections::BTreeMap;

use crate::client::HarvClient;
use harv_core::HarvError;
use serde::de::DeserializeOwned;
use tokio::task::JoinSet;

/// Maximum concurrent page requests for paginated endpoints.
const MAX_CONCURRENT_PAGES: usize = 3;

#[derive(Clone)]
struct PageRequest {
    path: String,
    base_query: Vec<(String, String)>,
    items_key: String,
}

impl PageRequest {
    fn new(path: &str, base_query: &[(&str, &str)], items_key: &str) -> Self {
        Self {
            path: path.to_owned(),
            base_query: base_query
                .iter()
                .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
                .collect(),
            items_key: items_key.to_owned(),
        }
    }

    async fn fetch<T: DeserializeOwned>(
        &self,
        client: &HarvClient,
        page: u64,
    ) -> Result<(Vec<T>, u64), HarvError> {
        let mut query = self.base_query.clone();
        query.push(("page".into(), page.to_string()));
        if !query.iter().any(|(key, _)| key == "per_page") {
            query.push(("per_page".into(), "100".into()));
        }
        let query_refs: Vec<(&str, &str)> = query
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect();
        let response: serde_json::Value = client.get(&self.path, &query_refs).await?;
        parse_page(response, &self.items_key)
    }
}

fn parse_page<T: DeserializeOwned>(
    mut response: serde_json::Value,
    items_key: &str,
) -> Result<(Vec<T>, u64), HarvError> {
    let items = response
        .get_mut(items_key)
        .ok_or_else(|| HarvError::InvalidApiResponse(format!("missing `{items_key}`")))?
        .take();
    let items = serde_json::from_value(items).map_err(|error| {
        HarvError::InvalidApiResponse(format!("unable to parse `{items_key}`: {error}"))
    })?;
    let total_pages = response
        .get("total_pages")
        .and_then(serde_json::Value::as_u64)
        .filter(|pages| *pages > 0)
        .ok_or_else(|| HarvError::InvalidApiResponse("invalid `total_pages`".into()))?;

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
    let request = PageRequest::new(path, base_query, items_key);
    let (page1_items, total_pages) = request.fetch(client, 1).await?;
    let mut all_items = page1_items;

    if total_pages > 1 {
        let mut set = JoinSet::new();
        let mut page_iter = (2..=total_pages).peekable();
        let mut page_results: BTreeMap<u64, Vec<T>> = BTreeMap::new();

        for _ in 0..MAX_CONCURRENT_PAGES {
            if let Some(&page) = page_iter.peek() {
                page_iter.next();
                let client = client.clone();
                let request = request.clone();
                set.spawn(async move { (page, request.fetch(&client, page).await) });
            }
        }

        while let Some(task_result) = set.join_next().await {
            let (page, page_result) =
                task_result.map_err(|error| HarvError::Http(error.to_string()))?;
            let (page_items, _) = page_result?;
            page_results.insert(page, page_items);

            if let Some(&next_page) = page_iter.peek() {
                page_iter.next();
                let client = client.clone();
                let request = request.clone();
                set.spawn(async move { (next_page, request.fetch(&client, next_page).await) });
            }
        }

        for items in page_results.into_values() {
            all_items.extend(items);
        }
    }

    Ok(all_items)
}

#[cfg(test)]
mod tests {
    use super::parse_page;
    use harv_core::HarvError;

    #[test]
    fn parses_valid_page() {
        let response = serde_json::json!({"items": [{"id": 1}], "total_pages": 2});
        let (items, total_pages) = parse_page::<serde_json::Value>(response, "items").unwrap();

        assert_eq!(items, vec![serde_json::json!({"id": 1})]);
        assert_eq!(total_pages, 2);
    }

    #[test]
    fn rejects_missing_or_malformed_items() {
        assert!(matches!(
            parse_page::<serde_json::Value>(serde_json::json!({"total_pages": 1}), "items"),
            Err(HarvError::InvalidApiResponse(_))
        ));
        assert!(matches!(
            parse_page::<u64>(
                serde_json::json!({"items": "bad", "total_pages": 1}),
                "items"
            ),
            Err(HarvError::InvalidApiResponse(_))
        ));
    }

    #[test]
    fn rejects_missing_or_invalid_total_pages() {
        assert!(matches!(
            parse_page::<serde_json::Value>(serde_json::json!({"items": []}), "items"),
            Err(HarvError::InvalidApiResponse(_))
        ));
        assert!(matches!(
            parse_page::<serde_json::Value>(
                serde_json::json!({"items": [], "total_pages": 0}),
                "items"
            ),
            Err(HarvError::InvalidApiResponse(_))
        ));
    }
}
