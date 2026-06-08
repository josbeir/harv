use crate::output;
use crate::prompts;
use crate::OutputFormat;
use harv_sdk::{Alias, HarvClient, HarvConfig};

pub async fn create(name: String) -> color_eyre::eyre::Result<()> {
    let client = HarvClient::from_config_file().await?;
    let assignments = client.projects().my_assignments().await?;
    let choices = prompts::build_project_choices(&assignments);

    if choices.is_empty() {
        println!("No project assignments found.");
        return Ok(());
    }

    let project_choice = prompts::pick_project(&choices)?;
    let task = prompts::pick_task(project_choice)?;

    let mut config = HarvConfig::load().await?;
    config
        .set_alias(
            &name,
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

pub async fn list(format: &OutputFormat) -> color_eyre::eyre::Result<()> {
    let config = HarvConfig::load().await?;

    if config.aliases.is_empty() {
        println!("No aliases defined.");
        println!("Use `harv alias create <name>` to create one.");
        return Ok(());
    }

    let headers = ["Alias", "Project ID", "Task ID"];
    let rows: Vec<[String; 3]> = config
        .aliases
        .iter()
        .map(|(name, alias)| {
            [
                name.clone(),
                alias.project_id.to_string(),
                alias.task_id.to_string(),
            ]
        })
        .collect();

    println!("{}", output::render(&headers, &rows, format));
    Ok(())
}

pub async fn delete(name: String) -> color_eyre::eyre::Result<()> {
    let mut config = HarvConfig::load().await?;

    if config.alias(&name).is_none() {
        println!("Alias '{}' not found.", name);
        return Ok(());
    }

    config.remove_alias(&name).await?;
    println!("Alias '{}' deleted.", name);
    Ok(())
}
