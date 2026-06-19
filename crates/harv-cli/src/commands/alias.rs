use std::collections::HashMap;

use crate::OutputFormat;
use crate::output;
use crate::prompts;
use crate::spinner;
use harv_sdk::{Alias, HarvClient, HarvConfig, ProjectConfig};

pub async fn create_execute(client: &HarvClient, name: &str) -> color_eyre::eyre::Result<()> {
    let (assignments, _) = spinner::with_spinner(
        "Loading project assignments...",
        client.projects().my_assignments(false),
    )
    .await?;

    let choices = prompts::build_project_choices(&assignments, None);
    if choices.is_empty() {
        println!("No project assignments found.");
        return Ok(());
    }

    let project_choice = prompts::pick_project(&choices, 0)?;
    let task = prompts::pick_task(project_choice)?;

    let mut config = HarvConfig::load().await?;
    config
        .set_alias(
            name,
            Alias {
                project_id: project_choice.project_id,
                task_id: task.task.id,
            },
        )
        .await?;

    println!(
        "Alias '{}' created: {} => {}",
        name, project_choice.display, task.task.name
    );
    Ok(())
}

pub async fn create(name: Option<String>) -> color_eyre::eyre::Result<()> {
    let name = match name {
        Some(n) => n,
        None => prompts::prompt_alias_name()?,
    };
    let client = HarvClient::from_config_file().await?;
    create_execute(&client, &name).await
}

pub async fn list_execute(
    client: &HarvClient,
    format: &OutputFormat,
) -> color_eyre::eyre::Result<()> {
    let config = client.config().clone();
    let project_config = ProjectConfig::discover().await?;
    let resolved = harv_sdk::ResolvedConfig::resolve(&config, project_config.as_ref());

    if resolved.aliases.is_empty() {
        println!("No aliases defined.\nUse `harv alias create` to create one.");
        return Ok(());
    }

    let (assignments, _) = client.projects().my_assignments(false).await?;

    let mut project_names: HashMap<u64, String> = HashMap::new();
    let mut task_names: HashMap<u64, String> = HashMap::new();
    for pa in &assignments {
        project_names.insert(pa.project.id, pa.project.name.clone());
        for ta in &pa.task_assignments {
            task_names.insert(ta.task.id, ta.task.name.clone());
        }
    }

    // Build a set of project-config alias names for source indication.
    let project_alias_names: std::collections::HashSet<&String> = project_config
        .as_ref()
        .map(|pc| pc.aliases.keys())
        .into_iter()
        .flatten()
        .collect();

    let headers = ["Alias", "Project", "Task", "Source"];
    let rows: Vec<[String; 4]> = resolved
        .aliases
        .iter()
        .map(|(n, a)| {
            let source = if project_alias_names.contains(n) {
                "project"
            } else {
                "global"
            };
            [
                n.clone(),
                project_names
                    .get(&a.project_id)
                    .cloned()
                    .unwrap_or_else(|| "—".into()),
                task_names
                    .get(&a.task_id)
                    .cloned()
                    .unwrap_or_else(|| "—".into()),
                source.to_string(),
            ]
        })
        .collect();
    output::print(&headers, &rows, format);
    Ok(())
}

pub async fn list(format: &OutputFormat) -> color_eyre::eyre::Result<()> {
    let client = HarvClient::from_config_file().await?;
    list_execute(&client, format).await
}

pub async fn delete_execute(name: &str) -> color_eyre::eyre::Result<()> {
    let mut config = HarvConfig::load().await?;
    if config.alias(name).is_none() {
        println!("Alias '{}' not found.", name);
        return Ok(());
    }
    config.remove_alias(name).await?;
    println!("Alias '{}' deleted.", name);
    Ok(())
}

pub async fn delete(name: String) -> color_eyre::eyre::Result<()> {
    delete_execute(&name).await
}
