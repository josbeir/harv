pub mod action;
pub mod app;
mod loading;
mod popup;
pub mod theme;
mod tui;
pub mod views;

use color_eyre::eyre;
use harv_sdk::HarvClient;

pub async fn run() -> eyre::Result<()> {
    let client = HarvClient::from_config_file()
        .await
        .map_err(|e| eyre::eyre!("{}", e.user_message()))?;

    let theme = theme::Theme::detect();
    tui::init()?;
    let mut app = app::App::new(client, theme);
    let result = app.run().await;
    tui::restore()?;
    result
}
