use crate::OutputFormat;
use crate::output;
use harv_sdk::HarvClient;

pub async fn execute(
    client: &HarvClient,
    project_id: u64,
    format: &OutputFormat,
) -> color_eyre::eyre::Result<()> {
    let assignments = client.projects().task_assignments(project_id).await?;

    let headers = ["Task", "Task ID", "Billable"];
    let rows: Vec<[String; 3]> = assignments
        .iter()
        .map(|t| {
            [
                t.task.name.clone(),
                t.task.id.to_string(),
                if t.billable { "Yes" } else { "No" }.into(),
            ]
        })
        .collect();

    output::print(&headers, &rows, format);
    Ok(())
}

pub async fn run(project_id: u64, format: &OutputFormat) -> color_eyre::eyre::Result<()> {
    let client = HarvClient::from_config_file().await?;
    execute(&client, project_id, format).await
}
