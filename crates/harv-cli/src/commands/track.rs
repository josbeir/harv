use crate::prompts;
use crate::spinner;
use harv_core::{CreateTimeEntry, HarvError};
use harv_sdk::HarvClient;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    client: &HarvClient,
    project_id: Option<u64>,
    task_id: Option<u64>,
    hours: Option<f64>,
    notes: Option<String>,
    editor: bool,
    date: Option<String>,
    refresh: bool,
    alias: Option<String>,
) -> color_eyre::eyre::Result<()> {
    let config = client.config().clone();

    let pb = spinner::new_spinner("Loading project assignments...");
    let assignments = client.projects().my_assignments(refresh).await?;
    pb.finish_and_clear();

    let choices = prompts::build_project_choices(&assignments);

    if choices.is_empty() {
        return Err(color_eyre::eyre::eyre!(
            HarvError::NoProjectAssignments.user_message()
        ));
    }

    let (resolved_project_id, resolved_task_id) = if let Some(alias_name) = &alias {
        let alias_obj = config.alias(alias_name).ok_or_else(|| {
            color_eyre::eyre::eyre!(HarvError::AliasNotFound(alias_name.clone()).user_message())
        })?;
        (Some(alias_obj.project_id), Some(alias_obj.task_id))
    } else {
        (project_id, task_id)
    };

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

    let t_id = if let Some(tid) = resolved_task_id {
        task_assignments
            .iter()
            .find(|t| t.task.id == tid)
            .map(|t| t.task.id)
            .ok_or_else(|| {
                color_eyre::eyre::eyre!("Task ID {} not assigned to project {}", tid, p_id)
            })?
    } else {
        let choice = choices.iter().find(|c| c.project_id == p_id).unwrap();
        let task = prompts::pick_task(choice)?;
        task.task.id
    };

    let spent_date = if let Some(ref d) = date {
        harv_core::datetime::parse_date_not_future(d)
            .map_err(|e| color_eyre::eyre::eyre!(e.user_message()))?
    } else {
        let today = harv_core::datetime::today();
        prompts::ask_date(today)?
    };

    let resolved_hours = if hours.is_some() {
        hours
    } else if date.is_none() && alias.is_none() && project_id.is_none() && task_id.is_none() {
        prompts::ask_hours()?
    } else {
        None
    };

    let resolved_notes = if let Some(n) = notes {
        Some(n)
    } else if editor {
        prompts::ask_notes(true)?
    } else if date.is_none() && alias.is_none() && project_id.is_none() && task_id.is_none() {
        prompts::ask_notes(false)?
    } else {
        None
    };

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

    let pb = spinner::new_spinner("Creating time entry...");
    let created = client
        .time_entries()
        .create(&entry)
        .await
        .map_err(|e| color_eyre::eyre::eyre!(e.user_message()))?;
    pb.finish_and_clear();

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

#[allow(clippy::too_many_arguments)]
pub async fn run(
    project_id: Option<u64>,
    task_id: Option<u64>,
    hours: Option<f64>,
    notes: Option<String>,
    editor: bool,
    date: Option<String>,
    refresh: bool,
    alias: Option<String>,
) -> color_eyre::eyre::Result<()> {
    let client = HarvClient::from_config_file().await?;
    execute(
        &client, project_id, task_id, hours, notes, editor, date, refresh, alias,
    )
    .await
}
