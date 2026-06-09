use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use harv_core::CreateTimeEntry;
use harv_sdk::HarvClient;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::Alignment;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use tokio::sync::mpsc::{self, UnboundedSender};

use crate::action::Action;
use crate::theme::Theme;
use crate::tui;
use crate::views::form::TimeEntryForm;
use crate::views::help::Help;
use crate::views::View;

pub struct App {
    client: Arc<HarvClient>,
    user_id: u64,
    current_view: View,
    form: Option<TimeEntryForm>,
    theme: Theme,
    help: Help,
    tick: u64,
    pending_confirm: Option<(String, Action)>,
}

impl App {
    pub async fn new(client: HarvClient) -> color_eyre::eyre::Result<Self> {
        let user = client
            .users()
            .me()
            .await
            .map_err(|e| color_eyre::eyre::eyre!("{}", e.user_message()))?;

        Ok(Self {
            client: Arc::new(client),
            user_id: user.id,
            current_view: View::default(),
            form: None,
            theme: Theme::default(),
            help: Help::default(),
            tick: 0,
            pending_confirm: None,
        })
    }

    pub async fn run(&mut self) -> color_eyre::eyre::Result<()> {
        let mut terminal = tui::terminal()?;
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();

        self.fetch_dashboard_data(&action_tx);

        let poll_client = Arc::clone(&self.client);
        let poll_tx = action_tx.clone();
        let poll_user_id = self.user_id;
        tokio::spawn(async move {
            loop {
                match poll_client.time_entries().running(poll_user_id).await {
                    Ok(entries) => {
                        let _ = poll_tx.send(Action::TimerUpdate(entries));
                    }
                    Err(e) => {
                        let _ = poll_tx.send(Action::Error(e.user_message()));
                    }
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        });

        let tick_tx = action_tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(80)).await;
                let _ = tick_tx.send(Action::Tick);
            }
        });

        let mut reader = tui::event_stream();

        loop {
            tokio::select! {
                Some(Ok(event)) = reader.next() => {
                    let actions = self.handle_event(event);
                    for action in actions {
                        if matches!(action, Action::Quit) {
                            return Ok(());
                        }
                        self.dispatch(action, &action_tx);
                    }
                }
                Some(action) = action_rx.recv() => {
                    self.dispatch(action, &action_tx);
                }
            }
            terminal.draw(|f| self.render(f))?;
        }
    }

    fn handle_event(&mut self, event: Event) -> Vec<Action> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if let Some((_, action)) = self.pending_confirm.take() {
                    if matches!(key.code, KeyCode::Char('y') | KeyCode::Char('Y')) {
                        return vec![action];
                    }
                    return vec![];
                }

                if let Some(ref mut form) = self.form {
                    return form.handle_key(&key);
                }

                if self.help.is_visible() {
                    return self.help.handle_key(&key);
                }

                match key.code {
                    KeyCode::Char('q') => {
                        vec![Action::Quit]
                    }
                    KeyCode::Char('?') => {
                        vec![Action::ToggleHelp]
                    }
                    _ => self.current_view.handle_key(&key),
                }
            }
            _ => vec![],
        }
    }

    fn dispatch(&mut self, action: Action, tx: &UnboundedSender<Action>) {
        match action {
            Action::Tick => {
                self.tick = self.tick.wrapping_add(1);
            }
            Action::ToggleHelp => {
                self.help.toggle();
            }
            Action::SwitchView(_) => {
                self.form = None;
                self.current_view = View::Dashboard(Default::default());
                self.fetch_dashboard_data(tx);
            }
            Action::OpenForm {
                last_project_id,
                last_task_id,
                project_name,
                log_mode,
            } => {
                let form =
                    TimeEntryForm::new(last_project_id, last_task_id, project_name, log_mode);
                self.form = Some(form);

                let client = Arc::clone(&self.client);
                let tx = tx.clone();
                tokio::spawn(async move {
                    match client.projects().my_assignments(false).await {
                        Ok(assignments) => {
                            let _ = tx.send(Action::FormAssignmentsUpdate(assignments));
                        }
                        Err(e) => {
                            let _ = tx.send(Action::Error(e.user_message()));
                        }
                    }
                });
            }
            Action::FormAssignmentsUpdate(assignments) => {
                if let Some(ref mut f) = self.form {
                    f.update_assignments(assignments);
                }
            }
            Action::FormTasksUpdate(tasks) => {
                if let Some(ref mut f) = self.form {
                    f.update_tasks(tasks);
                }
            }
            Action::FormSelectProject(project_id) => {
                let client = Arc::clone(&self.client);
                let tx = tx.clone();
                tokio::spawn(async move {
                    match client.projects().task_assignments(project_id).await {
                        Ok(tasks) => {
                            let _ = tx.send(Action::FormTasksUpdate(tasks));
                        }
                        Err(e) => {
                            let _ = tx.send(Action::Error(e.user_message()));
                        }
                    }
                });
            }
            Action::CreateEntry {
                project_id,
                task_id,
                spent_date,
                hours,
                notes,
            } => {
                // Save last used project/task to config
                {
                    let mut config = self.client.config().clone();
                    config.set_last_used(project_id, task_id);
                    let config = config;
                    tokio::spawn(async move {
                        let _ = config.save().await;
                    });
                }

                let client = Arc::clone(&self.client);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let spent_date = harv_core::datetime::parse_date(&spent_date)
                        .unwrap_or_else(|_| harv_core::datetime::today());

                    let entry = CreateTimeEntry {
                        project_id,
                        task_id,
                        spent_date: Some(spent_date),
                        hours,
                        notes,
                        started_time: None,
                        ended_time: None,
                    };
                    if let Err(e) = client.time_entries().create(&entry).await {
                        let _ = tx.send(Action::Error(e.user_message()));
                    }
                    let _ = tx.send(Action::Refresh);
                });
            }
            Action::TimerUpdate(entries) => {
                if let View::Dashboard(ref mut d) = self.current_view {
                    d.update_running(entries);
                }
            }
            Action::TodayEntriesUpdate(entries, _total) => {
                if let View::Dashboard(ref mut d) = self.current_view {
                    d.update_entries(entries);
                }
            }
            Action::Refresh => {
                if let View::Dashboard(ref mut d) = self.current_view {
                    d.set_loading();
                }
                self.fetch_dashboard_data(tx);
            }
            Action::StartTimer {
                project_id,
                task_id,
            } => {
                let client = Arc::clone(&self.client);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let entry = CreateTimeEntry {
                        project_id,
                        task_id,
                        spent_date: Some(harv_core::datetime::today()),
                        hours: None,
                        notes: None,
                        started_time: None,
                        ended_time: None,
                    };
                    if let Err(e) = client.time_entries().create(&entry).await {
                        let _ = tx.send(Action::Error(e.user_message()));
                    }
                    let _ = tx.send(Action::Refresh);
                });
            }
            Action::StopTimer { entry_id } => {
                let client = Arc::clone(&self.client);
                let tx = tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = client.time_entries().stop(entry_id).await {
                        let _ = tx.send(Action::Error(e.user_message()));
                    }
                    let _ = tx.send(Action::Refresh);
                });
            }
            Action::DeleteEntry { entry_id } => {
                let client = Arc::clone(&self.client);
                let tx = tx.clone();
                tokio::spawn(async move {
                    match client.time_entries().delete(entry_id).await {
                        Ok(()) => {
                            let _ = tx.send(Action::Refresh);
                        }
                        Err(e) => {
                            let _ = tx.send(Action::Error(e.user_message()));
                        }
                    }
                });
            }
            Action::ConfirmDelete {
                entry_id,
                entry_desc,
            } => {
                self.pending_confirm = Some((entry_desc, Action::DeleteEntry { entry_id }));
            }
            Action::Error(msg) => {
                tracing::error!("{}", msg);
            }
            _ => {}
        }
    }

    fn fetch_dashboard_data(&self, tx: &UnboundedSender<Action>) {
        let client = Arc::clone(&self.client);
        let user_id = self.user_id;
        let tx = tx.clone();

        tokio::spawn(async move {
            use harv_sdk::resources::time_entries::TimeEntryListParams;

            let today = harv_core::datetime::today();
            let params = TimeEntryListParams {
                user_id: Some(user_id),
                from: Some(today),
                to: Some(today),
                ..Default::default()
            };

            match client.time_entries().list(&params).await {
                Ok(entries) => {
                    let total: f64 = entries.iter().filter_map(|e| e.hours).sum();
                    let _ = tx.send(Action::TodayEntriesUpdate(entries, total));
                }
                Err(e) => {
                    let _ = tx.send(Action::Error(e.user_message()));
                }
            }
        });
    }

    fn render(&mut self, f: &mut Frame) {
        let area = f.area();

        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

        self.render_top_bar(layout[0], f);
        self.current_view
            .render(layout[1], f, &self.theme, self.tick);
        self.render_bottom_bar(layout[2], f);

        self.help.render(area, f, &self.theme);

        if let Some(ref mut form) = self.form {
            form.render(area, f, &self.theme, self.tick);
        }

        if let Some((ref msg, _)) = self.pending_confirm {
            render_confirm_dialog(area, f, msg, &self.theme);
        }
    }

    fn render_top_bar(&self, area: Rect, f: &mut Frame) {
        let spans = vec![
            Span::styled(
                " HARV ",
                Style::new()
                    .fg(self.theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " Dashboard ",
                Style::new()
                    .fg(self.theme.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
        ];

        let paragraph = Paragraph::new(Line::from(spans)).style(Style::new().bg(self.theme.bg));
        f.render_widget(paragraph, area);
    }

    fn render_bottom_bar(&self, area: Rect, f: &mut Frame) {
        let help = vec![
            Span::styled(" n: entry ", Style::new().fg(self.theme.muted)),
            Span::styled(" s: start ", Style::new().fg(self.theme.muted)),
            Span::styled(" d: delete ", Style::new().fg(self.theme.muted)),
            Span::styled(" r: refresh ", Style::new().fg(self.theme.muted)),
            Span::styled(" q: quit ", Style::new().fg(self.theme.muted)),
            Span::styled(" ?: help ", Style::new().fg(self.theme.muted)),
        ];

        let paragraph = Paragraph::new(Line::from(help));
        f.render_widget(paragraph, area);
    }
}

fn render_confirm_dialog(area: Rect, f: &mut Frame, msg: &str, theme: &Theme) {
    let max_width = 60u16;
    let popup_width = max_width.min(area.width.saturating_sub(4));
    let popup_height = 9;

    let centered = crate::popup::centered_rect_fixed(popup_width, popup_height, area);
    f.render_widget(Clear, centered);

    let block = Block::new()
        .title(" Confirm ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::new().fg(theme.warning))
        .style(Style::new().bg(theme.surface));

    let inner = block.inner(centered);
    f.render_widget(block, centered);

    let inner_with_margin = Rect {
        x: inner.x + 2,
        y: inner.y + 1,
        width: inner.width.saturating_sub(4),
        height: inner.height.saturating_sub(2),
    };

    let max_desc_width = inner_with_margin.width as usize;
    let truncated = harv_core::text::truncate(msg, max_desc_width);

    let lines = vec![
        Line::from(Span::styled(
            "Delete this entry?",
            Style::new().fg(theme.fg),
        )),
        Line::from(""),
        Line::from(Span::styled(truncated, Style::new().fg(theme.muted))),
        Line::from(""),
        Line::from(Span::styled(
            " y = confirm   any other key = cancel ",
            Style::new().fg(theme.muted),
        )),
    ];

    f.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        inner_with_margin,
    );
}
