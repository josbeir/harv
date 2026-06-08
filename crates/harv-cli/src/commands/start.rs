use crate::commands::track;
use harv_sdk::HarvClient;

#[allow(clippy::too_many_arguments)]
pub async fn execute(
    client: &HarvClient,
    alias: Option<String>,
    project_id: Option<u64>,
    task_id: Option<u64>,
    notes: Option<String>,
    editor: bool,
    date: Option<String>,
) -> color_eyre::eyre::Result<()> {
    track::execute(
        client, project_id, task_id, None, notes, editor, date, alias,
    )
    .await
}

pub async fn run(
    alias: Option<String>,
    project_id: Option<u64>,
    task_id: Option<u64>,
    notes: Option<String>,
    editor: bool,
    date: Option<String>,
) -> color_eyre::eyre::Result<()> {
    let client = HarvClient::from_config_file().await?;
    execute(&client, alias, project_id, task_id, notes, editor, date).await
}
