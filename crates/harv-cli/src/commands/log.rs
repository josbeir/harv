use crate::commands::track;
use harv_sdk::HarvClient;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    client: &HarvClient,
    hours: f64,
    alias: Option<String>,
    project_id: Option<u64>,
    task_id: Option<u64>,
    notes: Option<String>,
    editor: bool,
    date: Option<String>,
    refresh: bool,
) -> color_eyre::eyre::Result<()> {
    track::execute(
        client,
        project_id,
        task_id,
        Some(hours),
        notes,
        editor,
        date,
        refresh,
        alias,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn run(
    hours: f64,
    alias: Option<String>,
    project_id: Option<u64>,
    task_id: Option<u64>,
    notes: Option<String>,
    editor: bool,
    date: Option<String>,
    refresh: bool,
) -> color_eyre::eyre::Result<()> {
    let client = HarvClient::from_config_file().await?;
    execute(
        &client, hours, alias, project_id, task_id, notes, editor, date, refresh,
    )
    .await
}
