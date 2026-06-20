use std::time::Duration;

pub struct UpdateInfo {
    pub version: String,
    pub url: String,
}

const CACHE_FILE: &str = ".update_check";
const CACHE_TTL_HOURS: u64 = 24;
const USER_AGENT: &str = "harv-updater";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

pub async fn check_for_update(current_version: &str) -> Option<UpdateInfo> {
    let cache_path = cache_path();

    if let Some(info) = read_valid_cache(&cache_path, current_version).await {
        return Some(info);
    }

    let info = fetch_latest_release().await?;

    if is_newer(current_version, &info.version) {
        write_cache(&cache_path, &info.version).await;
        Some(info)
    } else {
        write_cache(&cache_path, &info.version).await;
        None
    }
}

async fn read_valid_cache(path: &std::path::Path, current_version: &str) -> Option<UpdateInfo> {
    let content = tokio::fs::read_to_string(path).await.ok()?;
    let mut lines = content.lines();

    let timestamp: i64 = lines.next()?.parse().ok()?;
    let version = lines.next()?.trim().to_string();

    let age_secs = timestamp_now() - timestamp;
    if age_secs < 0 || (age_secs as u64) >= CACHE_TTL_HOURS * 3600 {
        return None;
    }

    let url = lines
        .next()
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if is_newer(current_version, &version) {
        Some(UpdateInfo { version, url })
    } else {
        None
    }
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

async fn write_cache(path: &std::path::Path, version: &str) {
    let content = format!("{}\n{}\n", timestamp_now(), version);
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
    let cur: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();
    let lat: Vec<u32> = latest.split('.').filter_map(|s| s.parse().ok()).collect();
    cur < lat
}
