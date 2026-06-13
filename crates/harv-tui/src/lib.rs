pub mod action;
pub mod app;
mod loading;
mod popup;
pub mod theme;
mod tui;
pub mod views;

use color_eyre::eyre;
use harv_sdk::{HarvClient, ProjectConfig, ResolvedConfig};

pub async fn run() -> eyre::Result<()> {
    harv_core::init_locale(None);

    let client = HarvClient::from_config_file()
        .await
        .map_err(|e| eyre::eyre!("{}", e.user_message()))?;

    if let Some(locale) = &client.config().locale {
        harv_core::init_locale(Some(locale));
    }

    // Discover project-level config and merge with global config.
    let project_config = ProjectConfig::discover()
        .await
        .map_err(|e| eyre::eyre!("{}", e.user_message()))?;
    let resolved = ResolvedConfig::resolve(client.config(), project_config.as_ref());

    let theme = theme::Theme::detect();
    tui::init()?;
    let mut app = app::App::new(client, theme, resolved);
    let result = app.run().await;
    tui::restore()?;
    result
}
