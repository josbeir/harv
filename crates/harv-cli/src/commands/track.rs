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
    is_start: bool,
) -> color_eyre::eyre::Result<()> {
    let config = client.config().clone();

    let pb = spinner::new_spinner("Loading project assignments...");
    let (assignments, _) = client.projects().my_assignments(refresh).await?;
    pb.finish_and_clear();

    let choices = prompts::build_project_choices(&assignments, config.last_project_id);

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

    let (p_id, task_assignments, is_last) = if let Some(pid) = resolved_project_id {
        let choice = choices
            .iter()
            .find(|c| c.project_id == pid)
            .ok_or_else(|| {
                color_eyre::eyre::eyre!("Project ID {} not found in your assignments", pid)
            })?;
        let is_last = config.last_project_id == Some(pid);
        (choice.project_id, choice.task_assignments.clone(), is_last)
    } else {
        let choice = prompts::pick_project(&choices, 0)?;
        let is_last = config.last_project_id == Some(choice.project_id);
        (choice.project_id, choice.task_assignments.clone(), is_last)
    };

    let t_id = if let Some(tid) = resolved_task_id {
        task_assignments
            .iter()
            .find(|t| t.task.id == tid)
            .map(|t| t.task.id)
            .ok_or_else(|| {
                color_eyre::eyre::eyre!("Task ID {} not assigned to project {}", tid, p_id)
            })?
    } else if is_last {
        let use_last = config
            .last_task_id
            .is_some_and(|ltid| task_assignments.iter().any(|t| t.task.id == ltid));
        if use_last {
            config.last_task_id.unwrap()
        } else {
            let choice = choices.iter().find(|c| c.project_id == p_id).unwrap();
            prompts::pick_task(choice)?.task.id
        }
    } else {
        let choice = choices.iter().find(|c| c.project_id == p_id).unwrap();
        prompts::pick_task(choice)?.task.id
    };

    let spent_date = if let Some(ref d) = date {
        harv_core::datetime::parse_date_not_future(d)
            .map_err(|e| color_eyre::eyre::eyre!(e.user_message()))?
    } else if !is_start {
        let today = harv_core::datetime::today();
        prompts::ask_date(today)?
    } else {
        harv_core::datetime::today()
    };

    let resolved_hours = if is_start {
        None
    } else if let Some(h) = hours {
        Some(h)
    } else {
        prompts::ask_hours()?
    };

    let resolved_notes = if let Some(n) = notes {
        Some(n)
    } else if editor {
        prompts::ask_notes(true)?
    } else {
        prompts::ask_notes(false)?
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

    let mut saved_cfg = client.config().clone();
    saved_cfg.set_last_used(p_id, t_id);
    let _ = saved_cfg.save().await;

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
        &client, project_id, task_id, hours, notes, editor, date, refresh, alias, false,
    )
    .await
}
