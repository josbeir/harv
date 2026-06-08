use harv_sdk::HarvClient;

pub async fn run(
    notes: Option<String>,
    overwrite: bool,
    editor: bool,
) -> color_eyre::eyre::Result<()> {
    let client = HarvClient::from_config_file().await?;
    let user = client.users().me().await?;
    let running = client.time_entries().running(user.id).await?;

    if running.is_empty() {
        println!("No timer is currently running.");
        return Ok(());
    }

    let timer = if running.len() == 1 {
        &running[0]
    } else {
        // Multiple timers: pick one
        let items: Vec<String> = running
            .iter()
            .map(|t| {
                format!(
                    "[{}] {} => {} => {}",
                    t.timer_started_at
                        .map(|ts| harv_core::datetime::format_local(ts, true))
                        .unwrap_or_default(),
                    t.client
                        .as_ref()
                        .map(|c| c.name.as_str())
                        .unwrap_or("No client"),
                    t.project.name,
                    t.task.name,
                )
            })
            .collect();
        let items_str: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
        let selection =
            inquire::Select::new("Which timer do you want to stop?", items_str.clone()).prompt()?;
        let idx = items_str.iter().position(|&s| s == selection).unwrap();
        &running[idx]
    };

    // Handle notes update before stopping
    if notes.is_some() || editor {
        let existing = timer.notes.clone().unwrap_or_default();
        let updated = if editor {
            let input = inquire::Text::new("Notes (empty to keep current):")
                .prompt_skippable()?
                .filter(|s| !s.trim().is_empty());
            input.map(|n| {
                if overwrite || existing.is_empty() {
                    n
                } else {
                    format!("{}\n\n{}", existing, n)
                }
            })
        } else if let Some(n) = notes {
            if n.is_empty() {
                None
            } else if overwrite || existing.is_empty() {
                Some(n)
            } else {
                Some(format!("{}\n\n{}", existing, n))
            }
        } else {
            None
        };

        if let Some(n) = updated {
            let update = harv_core::UpdateTimeEntry {
                notes: Some(n),
                ..Default::default()
            };
            client.time_entries().update(timer.id, &update).await?;
            tracing::info!("Updated notes for timer #{}", timer.id);
        }
    }

    // Stop the timer
    let stopped = client.time_entries().stop(timer.id).await?;
    println!("Timer stopped.");
    println!(
        "  #{}\t{} → {} → {}\t{}h",
        stopped.id,
        stopped
            .client
            .as_ref()
            .map(|c| c.name.as_str())
            .unwrap_or("No client"),
        stopped.project.name,
        stopped.task.name,
        stopped.hours.unwrap_or(0.0),
    );

    Ok(())
}
