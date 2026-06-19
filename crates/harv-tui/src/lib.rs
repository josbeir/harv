pub mod action;
pub mod app;
mod loading;
mod popup;
pub mod theme;
mod tui;
pub mod views;

use color_eyre::eyre;
use harv_sdk::{HarvClient, ResolvedConfig};

pub async fn run() -> eyre::Result<()> {
    harv_core::init_locale(None);

    let client = HarvClient::from_config_or_mock()
        .await
        .map_err(|e| eyre::eyre!("{}", e.user_message()))?;

    if let Some(locale) = client.config().locale() {
        harv_core::init_locale(Some(locale));
    }

    let resolved = ResolvedConfig::resolve_from_environment(client.config())
        .await
        .map_err(|e| eyre::eyre!("{}", e.user_message()))?;

    let theme = theme::Theme::detect();
    tui::init()?;
    let mut app = app::App::new(client, theme, resolved);
    let result = app.run().await;
    tui::restore()?;
    result
}
