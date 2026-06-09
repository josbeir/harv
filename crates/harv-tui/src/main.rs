#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    harv_tui::run().await
}
