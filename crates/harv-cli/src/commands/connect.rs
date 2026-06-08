use harv_sdk::auth;
use harv_sdk::HarvConfig;

pub async fn run() -> color_eyre::eyre::Result<()> {
    eprintln!("Opening browser for Harvest authentication...");
    eprintln!("If the browser does not open, visit the URL shown below.\n");

    let (access_token, account_id) = auth::authenticate()
        .await
        .map_err(|e| color_eyre::eyre::eyre!("Authentication failed: {}", e.user_message()))?;

    let config = HarvConfig {
        access_token,
        account_id,
        cache_ttl_hours: 24,
        aliases: Default::default(),
    };

    config
        .save()
        .await
        .map_err(|e| color_eyre::eyre::eyre!("Failed to save config: {}", e.user_message()))?;

    let path = HarvConfig::path();
    println!(
        "Successfully authenticated with Harvest.\nConfig saved to {}",
        path.display()
    );

    Ok(())
}
