use harv_core::{t, t_args};
use harv_sdk::HarvConfig;

pub async fn run() -> color_eyre::eyre::Result<()> {
    let path = HarvConfig::path();

    if !path.exists() {
        println!("{}", t("cli-disconnect-not-auth"));
        return Ok(());
    }

    let account_id = HarvConfig::load()
        .await
        .map(|c| c.account_id().to_string())
        .ok();

    if let Some(ref id) = account_id {
        println!(
            "{}",
            t_args("cli-disconnect-disconnecting", &[("id", id.clone())])
        );
    } else {
        println!("{}", t("cli-disconnect-disconnecting-no-id"));
    }

    if let Some(ref id) = account_id
        && let Err(e) = harv_sdk::clear_cache(id).await
    {
        eprintln!(
            "{}",
            t_args("cli-disconnect-warning-cache", &[("err", e.user_message())])
        );
    }

    tokio::fs::remove_file(&path).await?;

    println!(
        "{}",
        t_args(
            "cli-disconnect-removed",
            &[("path", path.display().to_string())]
        )
    );
    if account_id.is_some() {
        println!("{}", t("cli-disconnect-cache-cleared"));
    }
    println!("{}", t("cli-disconnect-done"));

    Ok(())
}
