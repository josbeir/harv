use harv_core::{t, t_args};
use harv_sdk::HarvConfig;
use harv_sdk::auth;

pub async fn run() -> color_eyre::eyre::Result<()> {
    eprintln!("{}", t("cli-connect-opening"));
    eprintln!("{}\n", t("cli-auth-manual-url"));

    let (access_token, account_id) = auth::authenticate().await.map_err(|e| {
        color_eyre::eyre::eyre!(
            "{}",
            t_args("cli-connect-failed", &[("err", e.user_message())])
        )
    })?;

    let config = HarvConfig {
        access_token,
        account_id,
        cache_ttl_hours: 24,
        last_project_id: None,
        last_task_id: None,
        locale: None,
        aliases: Default::default(),
    };

    config.save().await.map_err(|e| {
        color_eyre::eyre::eyre!(
            "{}",
            t_args("cli-connect-save-failed", &[("err", e.user_message())])
        )
    })?;

    let path = HarvConfig::path();
    println!(
        "{}",
        t_args(
            "cli-connect-success",
            &[("path", path.display().to_string())]
        )
    );

    Ok(())
}
