use crate::OutputFormat;
use crate::output;
use crate::spinner;
use harv_core::text::format_project_display;
use harv_sdk::HarvClient;

pub async fn execute(
    client: &HarvClient,
    search: Option<String>,
    refresh: bool,
    format: &OutputFormat,
) -> color_eyre::eyre::Result<()> {
    let (assignments, _) = spinner::with_spinner(
        "Loading project assignments...",
        client.projects().my_assignments(refresh),
    )
    .await?;

    let mut filtered: Vec<_> = if let Some(query) = &search {
        let q = query.to_lowercase();
        assignments
            .into_iter()
            .filter(|a| {
                a.project.name.to_lowercase().contains(&q)
                    || a.project_code
                        .as_deref()
                        .map(|c| c.to_lowercase().contains(&q))
                        .unwrap_or(false)
                    || a.client
                        .as_ref()
                        .map(|c| c.name.to_lowercase().contains(&q))
                        .unwrap_or(false)
            })
            .collect()
    } else {
        assignments
    };

    filtered.sort_by(|a, b| {
        let a_client = a.client.as_ref().map(|c| c.name.as_str()).unwrap_or("");
        let b_client = b.client.as_ref().map(|c| c.name.as_str()).unwrap_or("");
        a_client
            .cmp(b_client)
            .then_with(|| a.project.name.cmp(&b.project.name))
    });

    let headers = ["Client", "Project", "Tasks", "Project ID"];
    let rows: Vec<[String; 4]> = filtered
        .iter()
        .map(|a| {
            [
                a.client
                    .as_ref()
                    .map(|c| c.name.clone())
                    .unwrap_or_default(),
                format_project_display(&a.project.name, a.project_code.as_deref()),
                a.task_assignments.len().to_string(),
                a.project.id.to_string(),
            ]
        })
        .collect();

    output::print(&headers, &rows, format);
    Ok(())
}

pub async fn run(
    search: Option<String>,
    refresh: bool,
    format: &OutputFormat,
) -> color_eyre::eyre::Result<()> {
    let client = HarvClient::from_config_file().await?;
    execute(&client, search, refresh, format).await
}
