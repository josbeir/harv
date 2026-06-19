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

    let timer = prompts::pick_running_timer(&running, "Which timer do you want to stop?")?;

    if notes.is_some() || editor {
        let existing = timer.notes.as_deref().unwrap_or("");
        let updated = prompts::resolve_entry_notes(existing, notes.as_deref(), overwrite, editor)?;
        if let Some(n) = updated {
            let update = harv_core::UpdateTimeEntry {
                notes: Some(n),
                ..Default::default()
            };
            client.time_entries().update(timer.id, &update).await?;
        }
    }

    let stopped = client.time_entries().stop(timer.id).await?;
    println!("Timer stopped.");
    println!(
        "  #{}\t{} → {} → {}\t{}h",
        stopped.id,
        harv_core::text::client_name_or_default(&stopped.client),
        stopped.project.name,
        stopped.task.name,
        stopped.hours.unwrap_or(0.0),
    );

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
