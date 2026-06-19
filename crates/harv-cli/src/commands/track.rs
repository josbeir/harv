use crate::prompts;
use crate::spinner;
use harv_core::{CreateTimeEntry, HarvError};
use harv_sdk::{HarvClient, ResolvedConfig, TemplateContext};

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
    let resolved = ResolvedConfig::resolve_from_environment(&config).await?;

    let (assignments, _) = spinner::with_spinner(
        "Loading project assignments...",
        client.projects().my_assignments(refresh),
    )
    .await?;

    // Use resolved defaults for project pre-selection (project config
    // takes priority over global last_used).
    let choices = prompts::build_project_choices(&assignments, resolved.default_project_id);

    if choices.is_empty() {
        return Err(color_eyre::eyre::eyre!(
            HarvError::NoProjectAssignments.user_message()
        ));
    }

    // Resolve alias from merged (global + project) alias map.
    let (resolved_project_id, resolved_task_id) = if let Some(alias_name) = &alias {
        let alias_obj = resolved.resolve_alias(alias_name).ok_or_else(|| {
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
        let cursor =
            crate::resolution::starting_cursor_for_default(&choices, resolved.default_project_id);
        let choice = prompts::pick_project(&choices, cursor)?;
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

    // Notes resolution with optional template expansion.
    // Priority: CLI flag > editor > template expansion > interactive prompt
    let resolved_notes = if let Some(n) = notes {
        Some(n)
    } else if editor {
        prompts::ask_notes(true)?
    } else {
        // Try template expansion: gather context and expand the default
        // template (named "default") if one exists.
        let template_default = resolved.default_template().map(|tpl| {
            let mut vars = TemplateContext::gather().unwrap_or_default();
            // Add project and task names from the assignments we already have.
            if let Some(choice) = choices.iter().find(|c| c.project_id == p_id) {
                // Extract project name from the display string (format: "Client => Project Name")
                let proj_name = choice
                    .display
                    .split(" => ")
                    .last()
                    .unwrap_or(&choice.display);
                vars.insert("project_name", proj_name.to_string());
            }
            if let Some(ta) = task_assignments.iter().find(|t| t.task.id == t_id) {
                vars.insert("task_name", ta.task.name.clone());
            }
            tpl.expand(&vars)
        });

        if let Some(default_notes) = template_default {
            prompts::ask_notes_with_default(&default_notes)?
        } else {
            prompts::ask_notes(false)?
        }
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

    let created = spinner::with_spinner(
        "Creating time entry...",
        client.time_entries().create(&entry),
    )
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

    // Persist last-used project/task to global config.
    {
        let mut saved_cfg = client.config().clone();
        let _ = saved_cfg.save_last_used(p_id, t_id).await;
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
    let client = HarvClient::from_config_or_mock().await?;
    execute(
        &client, project_id, task_id, hours, notes, editor, date, refresh, alias, false,
    )
    .await
}
