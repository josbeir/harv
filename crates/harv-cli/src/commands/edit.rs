use crate::prompts;
use crate::spinner;
use harv_core::{HarvError, UpdateTimeEntry};
use harv_sdk::HarvClient;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    client: &HarvClient,
    entry_id: Option<u64>,
    project_id: Option<u64>,
    task_id: Option<u64>,
    hours: Option<f64>,
    notes: Option<String>,
    editor: bool,
    overwrite: bool,
    date: Option<String>,
    refresh: bool,
) -> color_eyre::eyre::Result<()> {
    let user = {
        let pb = spinner::new_spinner("Loading...");
        let u = client.users().me().await?;
        pb.finish_and_clear();
        u
    };

    // Step 1: resolve the entry to edit
    let entry = if let Some(id) = entry_id {
        client
            .time_entries()
            .get(id)
            .await
            .map_err(|e| color_eyre::eyre::eyre!(e.user_message()))?
    } else {
        let pb = spinner::new_spinner("Loading...");
        let running = client.time_entries().running(user.id).await?;
        pb.finish_and_clear();

        let picked_id = if !running.is_empty() {
            pick_entry_id(&running, "Which timer do you want to edit?")?
        } else {
            let today = harv_core::datetime::today();
            let params = harv_sdk::resources::time_entries::TimeEntryListParams {
                user_id: Some(user.id),
                from: Some(today),
                to: Some(today),
                ..Default::default()
            };
            let entries = client
                .time_entries()
                .list(&params)
                .await
                .map_err(|e| color_eyre::eyre::eyre!(e.user_message()))?;

            if entries.is_empty() {
                return Err(color_eyre::eyre::eyre!(
                    "No entries to edit. Use `harv track` to create one."
                ));
            }
            pick_entry_id(&entries, "Which entry do you want to edit?")?
        };

        let pb = spinner::new_spinner("Loading entry details...");
        let entry = client
            .time_entries()
            .get(picked_id)
            .await
            .map_err(|e| color_eyre::eyre::eyre!(e.user_message()))?;
        pb.finish_and_clear();
        entry
    };

    let is_running = entry.is_running;

    // Step 2: guard against invalid flags for running entries
    if is_running && hours.is_some() {
        return Err(color_eyre::eyre::eyre!(
            "Cannot change hours on a running timer. Stop it first with `harv stop`."
        ));
    }
    if is_running && date.is_some() {
        return Err(color_eyre::eyre::eyre!(
            "Cannot change the date on a running timer. Stop it first with `harv stop`."
        ));
    }

    // Non-interactive path: entry_id + at least one update flag, skip prompts
    let has_flags = project_id.is_some()
        || task_id.is_some()
        || hours.is_some()
        || notes.is_some()
        || editor
        || date.is_some();
    if entry_id.is_some() && has_flags {
        return execute_non_interactive(
            client, &entry, project_id, task_id, hours, notes, editor, overwrite, date,
        )
        .await;
    }

    // Interactive path: load assignments and prompt
    let pb = spinner::new_spinner("Loading project assignments...");
    let assignments = client.projects().my_assignments(refresh).await?;
    pb.finish_and_clear();

    let choices = prompts::build_project_choices(&assignments, None);
    if choices.is_empty() {
        return Err(color_eyre::eyre::eyre!(
            HarvError::NoProjectAssignments.user_message()
        ));
    }

    // Step 4: resolve project
    let (p_id, task_assignments) = if let Some(pid) = project_id {
        let choice = choices
            .iter()
            .find(|c| c.project_id == pid)
            .ok_or_else(|| {
                color_eyre::eyre::eyre!("Project ID {} not found in your assignments", pid)
            })?;
        (pid, choice.task_assignments.clone())
    } else {
        let (pid, tasks) = prompts::pick_project_with_default(&choices, entry.project.id)?;
        (pid, tasks)
    };

    // Step 5: resolve task
    let t_id = if let Some(tid) = task_id {
        task_assignments
            .iter()
            .find(|t| t.task.id == tid)
            .map(|t| t.task.id)
            .ok_or_else(|| {
                color_eyre::eyre::eyre!("Task ID {} not assigned to project {}", tid, p_id)
            })?
    } else {
        prompts::pick_task_with_default(&task_assignments, entry.task.id)?
            .task
            .id
    };

    // Step 6: resolve date (stopped entries only)
    let spent_date = if is_running {
        None
    } else if let Some(ref d) = date {
        Some(
            harv_core::datetime::parse_date_not_future(d)
                .map_err(|e| color_eyre::eyre::eyre!(e.user_message()))?,
        )
    } else {
        let current_date = entry.spent_date.unwrap_or_else(harv_core::datetime::today);
        prompts::ask_date_with_default(current_date)?
    };

    // Step 7: resolve hours (stopped entries only)
    let resolved_hours = if is_running {
        None
    } else if let Some(h) = hours {
        Some(h)
    } else {
        prompts::ask_hours_with_default(entry.hours)?
    };

    // Step 8: resolve notes
    let resolved_notes = if let Some(n) = notes {
        if n.is_empty() { None } else { Some(n) }
    } else if editor {
        prompts::ask_notes(true)?
    } else {
        let existing = entry.notes.clone().unwrap_or_default();
        let prompt = format!(
            "Notes (current: \"{}\", empty to keep):",
            harv_core::text::truncate(&existing, 40)
        );
        inquire::Text::new(&prompt)
            .prompt_skippable()?
            .filter(|s| !s.trim().is_empty())
            .map(|n| {
                if overwrite || existing.is_empty() {
                    n
                } else {
                    harv_core::text::append_notes(&existing, &n)
                }
            })
    };

    // Step 9: PATCH the entry
    let pb = spinner::new_spinner("Saving changes...");
    let update = UpdateTimeEntry {
        project_id: if p_id != entry.project.id {
            Some(p_id)
        } else {
            None
        },
        task_id: if t_id != entry.task.id {
            Some(t_id)
        } else {
            None
        },
        spent_date,
        hours: resolved_hours,
        notes: resolved_notes,
        ..Default::default()
    };
    let updated = client
        .time_entries()
        .update(entry.id, &update)
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e.user_message()))?;
    pb.finish_and_clear();

    // Step 10: confirmation
    let display_hours = updated.hours.or(resolved_hours).or(entry.hours);
    let hours_str = display_hours
        .map(harv_core::text::format_hours)
        .unwrap_or_else(|| {
            if updated.is_running {
                "Running".into()
            } else {
                "—".into()
            }
        });
    let date_str = updated
        .spent_date
        .map(|d| d.format("%b %e, %Y").to_string())
        .unwrap_or_else(|| "today".into());
    println!(
        "Updated: #{} — {} — {} → {} → {}",
        updated.id, hours_str, date_str, updated.project.name, updated.task.name,
    );

    // Save last-used project/task
    let mut saved_cfg = client.config().clone();
    saved_cfg.set_last_used(p_id, t_id);
    let _ = saved_cfg.save().await;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn execute_non_interactive(
    client: &HarvClient,
    entry: &harv_core::TimeEntry,
    project_id: Option<u64>,
    task_id: Option<u64>,
    hours: Option<f64>,
    notes: Option<String>,
    editor: bool,
    overwrite: bool,
    date: Option<String>,
) -> color_eyre::eyre::Result<()> {
    let spent_date = if let Some(ref d) = date {
        Some(
            harv_core::datetime::parse_date_not_future(d)
                .map_err(|e| color_eyre::eyre::eyre!(e.user_message()))?,
        )
    } else {
        None
    };

    let resolved_notes = if let Some(n) = notes {
        if n.is_empty() {
            None
        } else if overwrite || entry.notes.as_deref().unwrap_or("").is_empty() {
            Some(n)
        } else {
            Some(harv_core::text::append_notes(
                entry.notes.as_deref().unwrap_or(""),
                &n,
            ))
        }
    } else if editor {
        prompts::ask_notes(true)?
    } else {
        None
    };

    let pb = spinner::new_spinner("Saving changes...");
    let update = UpdateTimeEntry {
        project_id,
        task_id,
        spent_date,
        hours,
        notes: resolved_notes,
        ..Default::default()
    };
    let updated = client
        .time_entries()
        .update(entry.id, &update)
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e.user_message()))?;
    pb.finish_and_clear();

    let hours_str = updated
        .hours
        .or(hours)
        .or(entry.hours)
        .map(harv_core::text::format_hours)
        .unwrap_or_else(|| {
            if updated.is_running {
                "Running".into()
            } else {
                "—".into()
            }
        });
    let date_str = updated
        .spent_date
        .map(|d| d.format("%b %e, %Y").to_string())
        .unwrap_or_else(|| "today".into());
    println!(
        "Updated: #{} — {} — {} → {} → {}",
        updated.id, hours_str, date_str, updated.project.name, updated.task.name,
    );

    if let (Some(pid), Some(tid)) = (project_id, task_id) {
        let mut saved_cfg = client.config().clone();
        saved_cfg.set_last_used(pid, tid);
        let _ = saved_cfg.save().await;
    }

    Ok(())
}

fn pick_entry_id(entries: &[harv_core::TimeEntry], prompt: &str) -> color_eyre::eyre::Result<u64> {
    if entries.len() == 1 {
        return Ok(entries[0].id);
    }
    let items: Vec<String> = entries.iter().map(format_entry_line).collect();
    let items_str: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
    let selection = inquire::Select::new(prompt, items_str.clone()).prompt()?;
    let idx = items_str.iter().position(|&s| s == selection).unwrap();
    Ok(entries[idx].id)
}

fn format_entry_line(entry: &harv_core::TimeEntry) -> String {
    let hours = entry
        .hours
        .map(harv_core::text::format_hours)
        .unwrap_or_else(|| {
            if entry.is_running {
                "Running".into()
            } else {
                "—".into()
            }
        });
    format!(
        "#{}  {}  {} → {}",
        entry.id, hours, entry.project.name, entry.task.name,
    )
}

#[allow(clippy::too_many_arguments)]
pub async fn run(
    entry_id: Option<u64>,
    project_id: Option<u64>,
    task_id: Option<u64>,
    hours: Option<f64>,
    notes: Option<String>,
    editor: bool,
    overwrite: bool,
    date: Option<String>,
    refresh: bool,
) -> color_eyre::eyre::Result<()> {
    let client = HarvClient::from_config_file().await?;
    execute(
        &client, entry_id, project_id, task_id, hours, notes, editor, overwrite, date, refresh,
    )
    .await
}
