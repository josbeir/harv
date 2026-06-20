use std::time::Duration;

pub struct UpdateInfo {
    pub version: String,
    pub url: String,
}

struct CacheEntry {
    version: String,
    url: String,
}

const CACHE_FILE: &str = ".update_check";
const CACHE_TTL_HOURS: u64 = 24;
const USER_AGENT: &str = "harv-updater";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

pub async fn check_for_update(current_version: &str) -> Option<UpdateInfo> {
    let cache_path = cache_path();

    if let Some(entry) = read_cache(&cache_path).await {
        if is_newer(current_version, &entry.version) {
            return Some(UpdateInfo {
                version: entry.version,
                url: entry.url,
            });
        }
        return None;
    }

    let info = fetch_latest_release().await?;

    write_cache(&cache_path, &info.version, &info.url).await;

    if is_newer(current_version, &info.version) {
        Some(info)
    } else {
        None
    }
}

async fn read_cache(path: &std::path::Path) -> Option<CacheEntry> {
    let content = tokio::fs::read_to_string(path).await.ok()?;
    let mut lines = content.lines();

    let timestamp: i64 = lines.next()?.parse().ok()?;
    let version = lines.next()?.trim().to_string();
    let url = lines
        .next()
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let age_secs = timestamp_now() - timestamp;
    if age_secs < 0 || (age_secs as u64) >= CACHE_TTL_HOURS * 3600 {
        return None;
    }

    Some(CacheEntry { version, url })
}

async fn fetch_latest_release() -> Option<UpdateInfo> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .user_agent(USER_AGENT)
        .build()
        .ok()?;

    let resp = client
        .get("https://api.github.com/repos/josbeir/harv/releases/latest")
        .send()
        .await
        .ok()?;

    let json: serde_json::Value = resp.json().await.ok()?;

    let tag = json.get("tag_name")?.as_str()?;
    let version = tag.strip_prefix('v').unwrap_or(tag).to_string();
    let url = json
        .get("html_url")
        .and_then(|v| v.as_str())
        .unwrap_or("https://github.com/josbeir/harv/releases/latest")
        .to_string();

    Some(UpdateInfo { version, url })
}

async fn write_cache(path: &std::path::Path, version: &str, url: &str) {
    let content = format!("{}\n{}\n{}\n", timestamp_now(), version, url);
    if let Some(parent) = path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    let _ = tokio::fs::write(path, content).await;
}

fn cache_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("harv")
        .join(CACHE_FILE)
}

fn timestamp_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn is_newer(current: &str, latest: &str) -> bool {
    let mut cur: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();
    let mut lat: Vec<u32> = latest.split('.').filter_map(|s| s.parse().ok()).collect();

    let max_len = cur.len().max(lat.len());
    cur.resize(max_len, 0);
    lat.resize(max_len, 0);

    cur < lat
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_same_version() {
        assert!(!is_newer("0.3.0", "0.3.0"));
    }

    #[test]
    fn test_is_newer_greater_patch() {
        assert!(is_newer("0.3.0", "0.3.1"));
    }

    #[test]
    fn test_is_newer_greater_minor() {
        assert!(is_newer("0.3.0", "0.4.0"));
    }

    #[test]
    fn test_is_newer_greater_major() {
        assert!(is_newer("0.3.0", "1.0.0"));
    }

    #[test]
    fn test_is_newer_not_newer() {
        assert!(!is_newer("0.4.0", "0.3.0"));
    }

    #[test]
    fn test_is_newer_unequal_lengths_four_vs_four_zero() {
        assert!(!is_newer("0.4", "0.4.0"));
    }

    #[test]
    fn test_is_newer_unequal_lengths_four_zero_vs_four() {
        assert!(!is_newer("0.4.0", "0.4"));
    }

    #[test]
    fn test_is_newer_unequal_lengths_newer_longer() {
        assert!(is_newer("0.4", "0.4.1"));
    }

    #[test]
    fn test_is_newer_unequal_lengths_older_shorter() {
        assert!(!is_newer("0.4.1", "0.4"));
    }

    #[test]
    fn test_is_newer_two_digit_versions() {
        assert!(is_newer("1.9", "1.10"));
    }

    #[tokio::test]
    async fn test_cache_read_expired() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CACHE_FILE);
        let old_ts = timestamp_now() - (CACHE_TTL_HOURS * 3600 + 1) as i64;
        tokio::fs::write(&path, format!("{}\n0.4.0\nhttps://example.com\n", old_ts))
            .await
            .unwrap();

        let result = read_cache(&path).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_read_valid() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CACHE_FILE);
        let ts = timestamp_now();
        tokio::fs::write(&path, format!("{}\n0.4.0\nhttps://example.com\n", ts))
            .await
            .unwrap();

        let result = read_cache(&path).await;
        assert!(result.is_some());
        let entry = result.unwrap();
        assert_eq!(entry.version, "0.4.0");
        assert_eq!(entry.url, "https://example.com");
    }

    #[tokio::test]
    async fn test_cache_read_old_format_no_url() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CACHE_FILE);
        let ts = timestamp_now();
        tokio::fs::write(&path, format!("{}\n0.4.0\n", ts))
            .await
            .unwrap();

        let result = read_cache(&path).await;
        assert!(result.is_some());
        let entry = result.unwrap();
        assert_eq!(entry.version, "0.4.0");
        assert_eq!(entry.url, "");
    }

    #[tokio::test]
    async fn test_cache_read_missing_file() {
        let result = read_cache(std::path::Path::new("/nonexistent/path/.update_check")).await;
        assert!(result.is_none());
    }

    #[test]
    fn test_timestamp_now_is_recent() {
        let ts = timestamp_now();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        assert!(ts > 0);
        assert!((now - ts).abs() < 2);
    }

    #[test]
    fn test_is_newer_empty_inputs() {
        assert!(!is_newer("", ""));
    }
}
