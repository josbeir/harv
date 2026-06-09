use harv_sdk::HarvConfig;

use crate::{ConfigAction, ConfigArgs};

pub async fn execute(args: &ConfigArgs) -> color_eyre::eyre::Result<()> {
    match &args.action {
        Some(ConfigAction::Set { setting, value }) => set(setting, value).await,
        Some(ConfigAction::Get { setting }) => get(setting).await,
        None => show().await,
    }
}

async fn show() -> color_eyre::eyre::Result<()> {
    let path = HarvConfig::path();

    println!("Config file: {}", path.display());

    if !path.exists() {
        println!("  (not found)");
        println!("Run `harv connect` to authenticate with Harvest.");
        return Ok(());
    }

    let config = HarvConfig::load()
        .await
        .map_err(|e| color_eyre::eyre::eyre!("Failed to load config: {}", e.user_message()))?;

    println!();
    println!(
        "  {:<20} {}",
        "access-token:",
        redact_token(&config.access_token)
    );
    println!("  {:<20} {}", "account-id:", config.account_id);
    println!("  {:<20} {}h", "cache-ttl:", config.cache_ttl_hours);
    println!(
        "  {:<20} {}",
        "aliases:",
        if config.aliases.is_empty() {
            "(none)".into()
        } else {
            config
                .aliases
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        }
    );

    Ok(())
}

async fn get(setting: &str) -> color_eyre::eyre::Result<()> {
    let config = HarvConfig::load()
        .await
        .map_err(|e| color_eyre::eyre::eyre!("Failed to load config: {}", e.user_message()))?;

    match setting {
        "cache-ttl" => println!("{}", config.cache_ttl_hours),
        "access-token" => println!("{}", redact_token(&config.access_token)),
        "account-id" => println!("{}", config.account_id),
        "aliases" => {
            if config.aliases.is_empty() {
                println!("(none)");
            } else {
                for (name, alias) in config.aliases.iter() {
                    println!(
                        "{} -> project: {}, task: {}",
                        name, alias.project_id, alias.task_id
                    );
                }
            }
        }
        other => {
            return Err(color_eyre::eyre::eyre!(
                "Unknown setting: {}. Valid settings: access-token, account-id, aliases, cache-ttl",
                other
            ))
        }
    }
    Ok(())
}

async fn set(setting: &str, value: &str) -> color_eyre::eyre::Result<()> {
    let mut config = HarvConfig::load()
        .await
        .map_err(|e| color_eyre::eyre::eyre!("Failed to load config: {}", e.user_message()))?;

    match setting {
        "cache-ttl" => {
            let hours: u64 = value
                .parse()
                .map_err(|_| color_eyre::eyre::eyre!("cache-ttl must be a positive number"))?;
            config.cache_ttl_hours = hours;
        }
        other => {
            return Err(color_eyre::eyre::eyre!(
                "Unknown setting: {}. Valid settings: cache-ttl",
                other
            ))
        }
    }

    config
        .save()
        .await
        .map_err(|e| color_eyre::eyre::eyre!("Failed to save config: {}", e.user_message()))?;
    println!("{} set to {}", setting, value);
    Ok(())
}

fn redact_token(token: &str) -> String {
    let chars: Vec<char> = token.chars().collect();
    if chars.len() <= 8 {
        return "<redacted>".into();
    }
    let prefix: String = chars.iter().take(4).collect();
    let suffix: String = chars
        .iter()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{}...{}", prefix, suffix)
}
