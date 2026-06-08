use crate::prompts;
use harv_core::{CreateTimeEntry, HarvError};
use harv_sdk::HarvClient;

pub async fn run(
    project_id: Option<u64>,
    task_id: Option<u64>,
    hours: Option<f64>,
    notes: Option<String>,
    editor: bool,
    date: Option<String>,
    alias: Option<String>,
) -> color_eyre::eyre::Result<()> {
    let client = HarvClient::from_config_file().await?;
    let config = client.config().clone();
    let assignments = client.projects().my_assignments().await?;
    let choices = prompts::build_project_choices(&assignments);

    if choices.is_empty() {
        return Err(color_eyre::eyre::eyre!(
            HarvError::NoProjectAssignments.user_message()
        ));
    }

    // Resolve alias if provided
    let (resolved_project_id, resolved_task_id) = if let Some(alias_name) = &alias {
        let alias_obj = config.alias(alias_name).ok_or_else(|| {
            color_eyre::eyre::eyre!(HarvError::AliasNotFound(alias_name.clone()).user_message())
        })?;
        (Some(alias_obj.project_id), Some(alias_obj.task_id))
    } else {
        (project_id, task_id)
    };

    // Pick or use project
    let (p_id, task_assignments) = if let Some(pid) = resolved_project_id {
        let choice = choices
            .iter()
            .find(|c| c.project_id == pid)
            .ok_or_else(|| {
                color_eyre::eyre::eyre!("Project ID {} not found in your assignments", pid)
            })?;
        (choice.project_id, choice.task_assignments.clone())
    } else {
        let choice = prompts::pick_project(&choices)?;
        (choice.project_id, choice.task_assignments.clone())
    };

    // Pick or use task
    let t_id = if let Some(tid) = resolved_task_id {
        task_assignments
            .iter()
            .find(|t| t.task.id == tid)
            .map(|t| t.task.id)
            .ok_or_else(|| {
                color_eyre::eyre::eyre!("Task ID {} not assigned to project {}", tid, p_id)
            })?
    } else {
        // We need to reconstruct a ProjectChoice for the pick_task function
        let choice = choices.iter().find(|c| c.project_id == p_id).unwrap();
        let task = prompts::pick_task(choice)?;
        task.task.id
    };

    // Date
    let spent_date = if let Some(ref d) = date {
        harv_core::datetime::parse_date_not_future(d)
            .map_err(|e| color_eyre::eyre::eyre!(e.user_message()))?
    } else {
        let today = harv_core::datetime::today();
        prompts::ask_date(today)?
    };

    // Hours
    let resolved_hours = if hours.is_some() {
        hours
    } else if date.is_none() && alias.is_none() && project_id.is_none() && task_id.is_none() {
        // Only prompt for hours in interactive mode (no flags, no alias)
        prompts::ask_hours()?
    } else {
        None
    };

    // Notes
    let resolved_notes = if let Some(n) = notes {
        Some(n)
    } else if editor {
        prompts::ask_notes(true)?
    } else if date.is_none() && alias.is_none() && project_id.is_none() && task_id.is_none() {
        prompts::ask_notes(false)?
    } else {
        None
    };

    // Calculate start/end time if hours provided
    let (started_time, ended_time) = resolved_hours
        .map(|h| {
            let (start, end) = harv_core::datetime::time_window(h);
            (start, end)
        })
        .unzip();

    let entry = CreateTimeEntry {
        project_id: p_id,
        task_id: t_id,
        spent_date: Some(spent_date),
        hours: resolved_hours,
        notes: resolved_notes,
        started_time,
        ended_time,
    };

    let created = client
        .time_entries()
        .create(&entry)
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e.user_message()))?;

    let confirmation = prompts::format_entry_confirmation(
        resolved_hours,
        &created.project,
        &created.task,
        spent_date,
        created.is_running,
        created.id,
    );

    if created.is_running {
        println!("Timer started! {}", confirmation);
    } else {
        println!("Created: {}", confirmation);
    }

    Ok(())
}
