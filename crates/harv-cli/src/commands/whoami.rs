use crate::OutputFormat;
use harv_core::{t, t_args};
use harv_sdk::{HarvClient, HarvConfig};

pub async fn execute(client: &HarvClient, output: &OutputFormat) -> color_eyre::eyre::Result<()> {
    let user = client.users().me().await?;
    let company = match client.company().get().await {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!(
                "{}",
                t_args("cli-whoami-warning-company", &[("err", e.user_message())])
            );
            None
        }
    };

    match output {
        OutputFormat::Table => {
            println!(
                "  {}",
                t_args(
                    "cli-whoami-account-label",
                    &[("account_id", client.config().account_id().to_string())]
                )
            );
            println!();
            println!(
                "  {:<20} {} {}",
                t("cli-whoami-name"),
                user.first_name,
                user.last_name
            );
            println!("  {:<20} {}", t("cli-whoami-email"), user.email);
            println!(
                "  {:<20} {}",
                t("cli-whoami-active"),
                if user.is_active {
                    t("text-yes")
                } else {
                    t("text-no")
                }
            );
            if let Some(ref tz) = user.timezone {
                println!("  {:<20} {}", t("cli-whoami-timezone"), tz);
            }
            if let Some(cap) = user.weekly_capacity {
                let hours = cap as f64 / 3600.0;
                println!("  {:<20} {:.0}h", t("cli-whoami-capacity"), hours);
            }
            if let Some(ref roles) = user.access_roles {
                println!("  {:<20} {}", t("cli-whoami-roles"), roles.join(", "));
            }
            if let Some(ref company) = company {
                println!("  {:<20} {}", t("cli-whoami-company"), company.name);
            }
        }
        OutputFormat::Json => {
            let json = serde_json::json!({
                "authenticated": true,
                "account_id": client.config().account_id(),
                "first_name": user.first_name,
                "last_name": user.last_name,
                "email": user.email,
                "is_active": user.is_active,
                "timezone": user.timezone,
                "weekly_capacity": user.weekly_capacity,
                "access_roles": user.access_roles,
                "company": company.as_ref().map(|c| &c.name),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
    }

    Ok(())
}

pub async fn run(output: &OutputFormat) -> color_eyre::eyre::Result<()> {
    if !HarvConfig::path().exists() {
        println!("{}", t("cli-whoami-not-auth"));
        println!("{}", t("cli-whoami-not-auth-hint"));
        return Ok(());
    }

    let client = HarvClient::from_config_file().await?;
    execute(&client, output).await
}
