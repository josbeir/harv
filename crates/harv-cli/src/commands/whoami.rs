use crate::OutputFormat;
use harv_sdk::{HarvClient, HarvConfig};

pub async fn execute(client: &HarvClient, output: &OutputFormat) -> color_eyre::eyre::Result<()> {
    let user = client.users().me().await?;
    let company = client.company().get().await.ok();

    match output {
        OutputFormat::Table => {
            println!("  Authenticated to account {}", client.config().account_id);
            println!();
            println!("  {:<20} {} {}", "Name:", user.first_name, user.last_name);
            println!("  {:<20} {}", "Email:", user.email);
            println!(
                "  {:<20} {}",
                "Active:",
                if user.is_active { "yes" } else { "no" }
            );
            if let Some(ref tz) = user.timezone {
                println!("  {:<20} {}", "Timezone:", tz);
            }
            if let Some(cap) = user.weekly_capacity {
                let hours = cap as f64 / 3600.0;
                println!("  {:<20} {:.0}h", "Weekly capacity:", hours);
            }
            if let Some(is_admin) = user.is_admin {
                println!("  {:<20} {}", "Admin:", if is_admin { "yes" } else { "no" });
            }
            if let Some(is_pm) = user.is_project_manager {
                println!(
                    "  {:<20} {}",
                    "Project manager:",
                    if is_pm { "yes" } else { "no" }
                );
            }
            if let Some(ref company) = company {
                println!("  {:<20} {}", "Company:", company.name);
            }
        }
        OutputFormat::Json => {
            let json = serde_json::json!({
                "authenticated": true,
                "account_id": client.config().account_id,
                "first_name": user.first_name,
                "last_name": user.last_name,
                "email": user.email,
                "is_active": user.is_active,
                "timezone": user.timezone,
                "weekly_capacity": user.weekly_capacity,
                "is_admin": user.is_admin,
                "is_project_manager": user.is_project_manager,
                "company": company.as_ref().map(|c| &c.name),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
    }

    Ok(())
}

pub async fn run(output: &OutputFormat) -> color_eyre::eyre::Result<()> {
    if !HarvConfig::path().exists() {
        println!("Not authenticated.");
        println!("Run `harv connect` to log in with your Harvest account.");
        return Ok(());
    }

    let client = HarvClient::from_config_file().await?;
    execute(&client, output).await
}
