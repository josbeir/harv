mod action;
mod app;
mod loading;
mod popup;
mod theme;
mod tui;
mod views;

use color_eyre::eyre;
use harv_sdk::HarvClient;

pub async fn run() -> eyre::Result<()> {
    let client = HarvClient::from_config_file()
        .await
        .map_err(|e| eyre::eyre!("{}", e.user_message()))?;

    tui::init()?;
    let result = app::App::new(client).await?.run().await;
    tui::restore()?;
    result
}
