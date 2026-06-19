use crate::prompts;
use crate::spinner;
use harv_sdk::HarvClient;

pub async fn execute(
    client: &HarvClient,
    notes: Option<String>,
    overwrite: bool,
    editor: bool,
) -> color_eyre::eyre::Result<()> {
    let user = spinner::with_spinner("Loading...", client.users().me()).await?;
    let running = client.time_entries().running(user.id).await?;

    if running.is_empty() {
        println!("No timer is currently running.");
        return Ok(());
    }

    let timer = prompts::pick_running_timer(&running, "Which timer?")?;

    let existing = timer.notes.as_deref().unwrap_or("");
    let updated = prompts::resolve_entry_notes(existing, notes.as_deref(), overwrite, editor)?;

    if let Some(updated_notes) = updated {
        let update = harv_core::UpdateTimeEntry {
            notes: Some(updated_notes),
            ..Default::default()
        };
        client.time_entries().update(timer.id, &update).await?;
        println!("Notes updated for timer #{}", timer.id);
    } else {
        println!("Nothing to update for timer #{}", timer.id);
    }

    Ok(())
}

pub async fn run(
    notes: Option<String>,
    overwrite: bool,
    editor: bool,
) -> color_eyre::eyre::Result<()> {
    let client = HarvClient::from_config_file().await?;
    execute(&client, notes, overwrite, editor).await
}
