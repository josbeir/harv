use crate::commands::track;

pub async fn run(
    hours: f64,
    alias: Option<String>,
    project_id: Option<u64>,
    task_id: Option<u64>,
    notes: Option<String>,
    editor: bool,
    date: Option<String>,
) -> color_eyre::eyre::Result<()> {
    track::run(project_id, task_id, Some(hours), notes, editor, date, alias).await
}
