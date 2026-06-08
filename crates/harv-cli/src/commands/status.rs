use crate::prompts;
use harv_sdk::HarvClient;

pub async fn run(output: &crate::OutputFormat) -> color_eyre::eyre::Result<()> {
    let client = HarvClient::from_config_file().await?;
    let user = client.users().me().await?;
    let running = client.time_entries().running(user.id).await?;

    if running.is_empty() {
        println!("No timer is currently running.");
    } else {
        println!("Running timers:\n");
        for timer in &running {
            let elapsed = timer.timer_started_at.map(|ts| {
                let now = chrono::Utc::now();
                let duration = now - ts;
                duration.num_minutes()
            });

            let display = prompts::format_timer_display(timer, elapsed, output);
            println!("{}", display);
            if running.len() > 1 {
                println!();
            }
        }
    }

    // Show today's entries
    let today = harv_core::datetime::today();
    let params = harv_sdk::resources::time_entries::TimeEntryListParams {
        user_id: Some(user.id),
        from: Some(today),
        to: Some(today),
        ..Default::default()
    };
    let today_entries = client.time_entries().list(&params).await?;

    let total: f64 = today_entries.iter().filter_map(|e| e.hours).sum();

    match output {
        crate::OutputFormat::Table => {
            if !today_entries.is_empty() {
                println!("\nToday's entries ({:.2}h total):", total);
                for entry in &today_entries {
                    let hours = entry
                        .hours
                        .map(|h| format!("{:.2}h", h))
                        .unwrap_or_default();
                    let project = &entry.project.name;
                    let task = &entry.task.name;
                    let notes = entry.notes.as_deref().unwrap_or("");
                    println!(
                        "  #{}\t{}\t{} → {}\t{}",
                        entry.id, hours, project, task, notes
                    );
                }
            }
        }
        crate::OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "running": running.iter().map(|t| serde_json::json!({
                        "id": t.id,
                        "project": t.project.name,
                        "task": t.task.name,
                        "started_at": t.timer_started_at.map(|ts| ts.to_rfc3339()),
                    })).collect::<Vec<_>>(),
                    "today": {
                        "total_hours": total,
                        "entries": today_entries.iter().map(|e| serde_json::json!({
                            "id": e.id,
                            "hours": e.hours,
                            "project": e.project.name,
                            "task": e.task.name,
                            "notes": e.notes,
                        })).collect::<Vec<_>>(),
                    }
                })
            );
        }
    }

    Ok(())
}
