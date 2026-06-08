use crate::output;
use crate::OutputFormat;
use harv_sdk::HarvClient;

pub async fn run(search: Option<String>, format: &OutputFormat) -> color_eyre::eyre::Result<()> {
    let client = HarvClient::from_config_file().await?;
    let assignments = client.projects().my_assignments().await?;

    let mut filtered: Vec<_> = if let Some(query) = &search {
        let q = query.to_lowercase();
        assignments
            .into_iter()
            .filter(|a| {
                a.project.name.to_lowercase().contains(&q)
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
                a.project.name.clone(),
                a.task_assignments.len().to_string(),
                a.project.id.to_string(),
            ]
        })
        .collect();

    println!("{}", output::render(&headers, &rows, format));
    Ok(())
}
