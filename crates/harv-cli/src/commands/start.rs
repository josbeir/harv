use crate::commands::track;

pub async fn run(
    alias: Option<String>,
    project_id: Option<u64>,
    task_id: Option<u64>,
    notes: Option<String>,
    editor: bool,
    date: Option<String>,
) -> color_eyre::eyre::Result<()> {
    // For start, we always start a running timer (hours = None)
    // unless --hours is passed, but start doesn't have that flag
    track::run(project_id, task_id, None, notes, editor, date, alias).await
}
