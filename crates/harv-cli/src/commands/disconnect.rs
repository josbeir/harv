use harv_sdk::HarvConfig;

pub async fn run() -> color_eyre::eyre::Result<()> {
    let path = HarvConfig::path();

    if !path.exists() {
        println!("Not authenticated. Nothing to disconnect.");
        return Ok(());
    }

    let config = HarvConfig::load()
        .await
        .map_err(|e| color_eyre::eyre::eyre!("Failed to load config: {}", e.user_message()))?;

    println!(
        "Disconnecting from Harvest account {}...",
        config.account_id
    );

    harv_sdk::cache::clear_cache(&config.account_id).await?;

    tokio::fs::remove_file(&path).await?;

    println!("Config removed: {}", path.display());
    println!("Project cache cleared.");
    println!("You are now disconnected. Run `harv connect` to log back in.");

    Ok(())
}
