use harv_sdk::HarvConfig;

pub async fn run() -> color_eyre::eyre::Result<()> {
    let path = HarvConfig::path();

    if !path.exists() {
        println!("Not authenticated. Nothing to disconnect.");
        return Ok(());
    }

    let account_id = HarvConfig::load().await.map(|c| c.account_id).ok();

    if let Some(ref id) = account_id {
        println!("Disconnecting from Harvest account {}...", id);
    } else {
        println!("Disconnecting...");
    }

    if let Some(ref id) = account_id
        && let Err(e) = harv_sdk::clear_cache(id).await
    {
        eprintln!(
            "Warning: could not clear project cache: {}",
            e.user_message()
        );
    }

    tokio::fs::remove_file(&path).await?;

    println!("Config removed: {}", path.display());
    if account_id.is_some() {
        println!("Project cache cleared.");
    }
    println!("You are now disconnected. Run `harv connect` to log back in.");

    Ok(())
}
