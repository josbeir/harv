use harv_sdk::HarvConfig;

pub async fn execute() -> color_eyre::eyre::Result<()> {
    let path = HarvConfig::path();

    print!("Config file: {}", path.display());

    if !path.exists() {
        println!();
        println!("Config file does not exist.");
        println!("Run `harv connect` to authenticate with Harvest.");
        return Ok(());
    }

    let config = HarvConfig::load()
        .await
        .map_err(|e| color_eyre::eyre::eyre!("Failed to load config: {}", e.user_message()))?;

    println!();
    println!("  Account ID: {}", config.account_id);
    println!(
        "  Access token: {}...",
        &config.access_token[..12.min(config.access_token.len())]
    );
    println!("  Aliases: {}", config.aliases.len());

    if !config.aliases.is_empty() {
        println!();
        println!("Aliases:");
        for (name, alias) in config.aliases.iter() {
            println!(
                "  {} -> project: {}, task: {}",
                name, alias.project_id, alias.task_id
            );
        }
    }

    Ok(())
}

pub async fn run() -> color_eyre::eyre::Result<()> {
    execute().await
}
