use harv_core::{t, t_args};
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

    println!(
        "{}",
        t_args("cli-config-file", &[("path", path.display().to_string())])
    );

    if !path.exists() {
        println!("  {}", t("cli-config-not-found"));
        println!("{}", t("err-config-not-found"));
        return Ok(());
    }

    let config = HarvConfig::load().await.map_err(|e| {
        color_eyre::eyre::eyre!(
            "{}",
            t_args("cli-config-load-failed", &[("err", e.user_message())])
        )
    })?;

    println!();
    println!(
        "  {:<20} {}",
        t("cli-config-access-token"),
        redact_token(config.access_token())
    );
    println!(
        "  {:<20} {}",
        t("cli-config-account-id"),
        config.account_id()
    );
    println!(
        "  {:<20} {}",
        t("cli-config-locale"),
        config.locale().unwrap_or(&t("cli-config-auto-detect"))
    );
    println!(
        "  {:<20} {}h",
        t("cli-config-cache-ttl"),
        config.cache_ttl_hours()
    );
    println!(
        "  {:<20} {}",
        t("cli-config-aliases"),
        if config.aliases().is_empty() {
            t("cli-config-none-bare")
        } else {
            config
                .aliases()
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        }
    );
    println!(
        "  {:<20} {}",
        t("cli-config-check-updates"),
        if config.check_updates() {
            t("text-yes")
        } else {
            t("text-no")
        }
    );

    Ok(())
}

async fn get(setting: &str) -> color_eyre::eyre::Result<()> {
    let config = HarvConfig::load().await.map_err(|e| {
        color_eyre::eyre::eyre!(
            "{}",
            t_args("cli-config-load-failed", &[("err", e.user_message())])
        )
    })?;

    match setting {
        "cache-ttl" => println!("{}", config.cache_ttl_hours()),
        "access-token" => println!("{}", redact_token(config.access_token())),
        "account-id" => println!("{}", config.account_id()),
        "locale" => println!("{}", config.locale().unwrap_or("")),
        "check-updates" => println!("{}", config.check_updates()),
        "aliases" => {
            if config.aliases().is_empty() {
                println!("{}", t("cli-config-none-bare"));
            } else {
                for (name, alias) in config.aliases().iter() {
                    println!(
                        "{}",
                        t_args(
                            "cli-config-alias-format",
                            &[
                                ("name", name.clone()),
                                ("pid", alias.project_id.to_string()),
                                ("tid", alias.task_id.to_string()),
                            ],
                        )
                    );
                }
            }
        }
        other => {
            return Err(color_eyre::eyre::eyre!(
                "{}",
                t_args("cli-config-unknown-setting", &[("setting", other.into())])
            ));
        }
    }
    Ok(())
}

async fn set(setting: &str, value: &str) -> color_eyre::eyre::Result<()> {
    let mut config = HarvConfig::load().await.map_err(|e| {
        color_eyre::eyre::eyre!(
            "{}",
            t_args("cli-config-load-failed", &[("err", e.user_message())])
        )
    })?;

    match setting {
        "cache-ttl" => {
            let hours: u64 = value
                .parse()
                .map_err(|_| color_eyre::eyre::eyre!("{}", t("cli-config-cache-ttl-invalid")))?;
            config.set_cache_ttl_hours(hours);
        }
        "locale" => {
            if value.is_empty() || value == "auto" {
                config.set_locale(None);
            } else {
                let valid = harv_core::locale::SUPPORTED_LANGS;
                if valid.contains(&value) {
                    config.set_locale(Some(value.into()));
                } else {
                    return Err(color_eyre::eyre::eyre!(
                        "{}",
                        t_args(
                            "cli-config-locale-invalid",
                            &[("value", value.into()), ("supported", valid.join(", ")),]
                        )
                    ));
                }
            }
        }
        "check-updates" => {
            let val: bool = match value.to_lowercase().as_str() {
                "true" | "yes" | "1" | "on" => true,
                "false" | "no" | "0" | "off" => false,
                _ => {
                    return Err(color_eyre::eyre::eyre!(
                        "{}",
                        t("cli-config-check-updates-invalid")
                    ));
                }
            };
            config.set_check_updates(val);
        }
        other => {
            return Err(color_eyre::eyre::eyre!(
                "{}",
                t_args(
                    "cli-config-unknown-setting-set",
                    &[("setting", other.into())]
                )
            ));
        }
    }

    config.save().await.map_err(|e| {
        color_eyre::eyre::eyre!(
            "{}",
            t_args("cli-config-save-failed", &[("err", e.user_message())])
        )
    })?;
    println!(
        "{}",
        t_args(
            "cli-config-set-success",
            &[("setting", setting.into()), ("value", value.into())]
        )
    );
    Ok(())
}

fn redact_token(token: &str) -> String {
    let chars: Vec<char> = token.chars().collect();
    if chars.len() <= 8 {
        return t("cli-config-redacted");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_token_long() {
        harv_core::init_locale(Some("en"));
        let result = redact_token("abcdefghijklmnop");
        assert!(result.starts_with("abcd"));
        assert!(result.ends_with("mnop"));
        assert!(result.contains("..."));
    }

    #[test]
    fn test_redact_token_short() {
        harv_core::init_locale(Some("en"));
        let result = redact_token("short");
        assert!(!result.contains("..."));
    }

    #[test]
    fn test_redact_token_exactly_8() {
        harv_core::init_locale(Some("en"));
        let result = redact_token("12345678");
        assert!(!result.contains("..."));
    }
}
