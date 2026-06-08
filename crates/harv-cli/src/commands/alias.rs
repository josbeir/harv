use crate::output;
use crate::prompts;
use crate::spinner;
use crate::OutputFormat;
use harv_sdk::{Alias, HarvClient, HarvConfig};

pub async fn create_execute(client: &HarvClient, name: &str) -> color_eyre::eyre::Result<()> {
    let pb = spinner::new_spinner("Loading project assignments...");
    let assignments = client.projects().my_assignments(false).await?;
    pb.finish_and_clear();

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

pub async fn create(name: String) -> color_eyre::eyre::Result<()> {
    let client = HarvClient::from_config_file().await?;
    create_execute(&client, &name).await
}

pub async fn list_execute(format: &OutputFormat) -> color_eyre::eyre::Result<()> {
    let config = HarvConfig::load().await?;
    if config.aliases.is_empty() {
        println!("No aliases defined.\nUse `harv alias create <name>` to create one.");
        return Ok(());
    }
    let headers = ["Alias", "Project ID", "Task ID"];
    let rows: Vec<[String; 3]> = config
        .aliases
        .iter()
        .map(|(n, a)| [n.clone(), a.project_id.to_string(), a.task_id.to_string()])
        .collect();
    println!("{}", output::render(&headers, &rows, format));
    Ok(())
}

pub async fn list(format: &OutputFormat) -> color_eyre::eyre::Result<()> {
    list_execute(format).await
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
