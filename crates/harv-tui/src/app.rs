use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::NaiveDate;
use crossterm::execute;
use crossterm::terminal::SetTitle;
use futures_util::StreamExt;
use harv_core::CreateTimeEntry;
use harv_sdk::{HarvClient, ResolvedConfig};
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::Alignment;
use ratatui::layout::{Constraint, Layout, Rect, Spacing};
use ratatui::style::{Modifier, Style};
use ratatui::symbols::merge::MergeStrategy;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use tokio::sync::mpsc::{self, UnboundedSender};

use crate::action::Action;
use crate::theme::{Theme, ThemeMode};
use crate::tui;
use crate::views::View;
use crate::views::date_picker::DatePicker;
use crate::views::form::TimeEntryForm;
use crate::views::help::Help;

pub struct App {
    client: Arc<HarvClient>,
    user_id: u64,
    user_name: Option<String>,
    current_view: View,
    form: Option<TimeEntryForm>,
    theme: Theme,
    help: Help,
    tick: u64,
    pending_confirm: Option<(String, Vec<Action>)>,
    date_picker: Option<DatePicker>,
    project_codes: HashMap<u64, String>,
    resolved_config: ResolvedConfig,
}

impl App {
    pub fn new(client: HarvClient, theme: Theme, resolved_config: ResolvedConfig) -> Self {
        Self {
            client: Arc::new(client),
            user_id: 0,
            user_name: None,
            current_view: View::default(),
            form: None,
            theme,
            help: Help::default(),
            tick: 0,
            pending_confirm: None,
            date_picker: None,
            project_codes: HashMap::new(),
            resolved_config,
        }
    }

    // Test helpers
    #[doc(hidden)]
    pub fn new_for_testing(client: HarvClient) -> Self {
        let resolved = ResolvedConfig::resolve(client.config(), None);
        Self::new(client, Theme::default(), resolved)
    }

    #[doc(hidden)]
    #[doc(hidden)]
    pub fn user_id(&self) -> u64 {
        self.user_id
    }

    #[doc(hidden)]
    pub fn set_user_id(&mut self, id: u64) {
        self.user_id = id;
    }

    #[doc(hidden)]
    pub fn has_form(&self) -> bool {
        self.form.is_some()
    }

    #[doc(hidden)]
    pub fn dashboard(&self) -> &crate::views::dashboard::Dashboard {
        match &self.current_view {
            View::Dashboard(d) => d,
        }
    }

    pub async fn run(&mut self) -> color_eyre::eyre::Result<()> {
        let mut terminal = tui::terminal()?;
        self.update_window_title();
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();

        // Fetch user info asynchronously — dashboard shows loading animation
        {
            let client = Arc::clone(&self.client);
            let tx = action_tx.clone();
            tokio::spawn(async move {
                match client.users().me().await {
                    Ok(user) => {
                        let _ = tx.send(Action::UserLoaded(user));
                    }
                    Err(e) => {
                        let _ = tx.send(Action::Error(e.user_message()));
                    }
                }
            });
        }

        let tick_tx = action_tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(80)).await;
                let _ = tick_tx.send(Action::Tick);
            }
        });

        // Spawn OS theme change watcher
        let theme_tx = action_tx.clone();
        tokio::spawn(async move {
            watch_theme_changes(theme_tx).await;
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
                if let Some((_, actions)) = self.pending_confirm.take() {
                    if matches!(key.code, KeyCode::Char('y') | KeyCode::Char('Y')) {
                        return actions;
                    }
                    return vec![];
                }

                if let Some(ref mut picker) = self.date_picker {
                    return picker.handle_key(&key);
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
                    KeyCode::Char('c')
                        if key.modifiers == ratatui::crossterm::event::KeyModifiers::CONTROL =>
                    {
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

    pub fn dispatch(&mut self, action: Action, tx: &UnboundedSender<Action>) {
        match action {
            Action::Tick => {
                self.tick = self.tick.wrapping_add(1);
            }
            Action::UserLoaded(user) => {
                self.user_id = user.id;
                self.user_name = Some(format!("{} {}", user.first_name, user.last_name));

                // Start the timer poller now that we have a user_id
                let poll_client = Arc::clone(&self.client);
                let poll_tx = tx.clone();
                let poll_user_id = user.id;
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

                self.fetch_dashboard_data(tx, false, harv_core::datetime::today());
            }
            Action::ToggleHelp => {
                self.help.toggle();
            }
            Action::ThemeChanged(mode) => {
                self.theme = match mode {
                    ThemeMode::Dark => Theme::dark(),
                    ThemeMode::Light => Theme::light(),
                };
            }
            Action::SwitchView(_) => {
                self.form = None;
                let date = {
                    let View::Dashboard(d) = &self.current_view;
                    d.selected_date()
                };
                self.fetch_dashboard_data(tx, false, date);
            }
            Action::OpenForm {
                last_project_id,
                last_task_id,
                project_name,
                mode,
                entry_id,
                entry_date,
                entry_hours,
                entry_notes,
                is_running,
            } => {
                // Use project-config defaults when no specific override is provided.
                let pid = last_project_id.or(self.resolved_config.default_project_id);
                let tid = last_task_id.or(self.resolved_config.default_task_id);
                let form = TimeEntryForm::new(
                    pid,
                    tid,
                    project_name,
                    mode,
                    entry_id,
                    entry_date,
                    entry_hours,
                    entry_notes,
                    is_running,
                );
                self.form = Some(form);

                let client = Arc::clone(&self.client);
                let tx = tx.clone();
                tokio::spawn(async move {
                    match client.projects().my_assignments(false).await {
                        Ok((assignments, _)) => {
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
            Action::CreateEntry {
                project_id,
                task_id,
                spent_date,
                hours,
                notes,
            } => {
                let View::Dashboard(d) = &mut self.current_view;
                d.set_loading(harv_core::t("tui-app-loading-create"));

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
                    let _ = tx.send(Action::RefreshEntries);
                });
            }
            Action::EditEntry {
                entry_id,
                project_id,
                task_id,
                spent_date,
                hours,
                notes,
            } => {
                let View::Dashboard(d) = &mut self.current_view;
                d.set_loading(harv_core::t("tui-app-loading-save"));

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

                    let update = harv_core::UpdateTimeEntry {
                        project_id: Some(project_id),
                        task_id: Some(task_id),
                        spent_date: Some(spent_date),
                        hours,
                        notes,
                        ..Default::default()
                    };
                    if let Err(e) = client.time_entries().update(entry_id, &update).await {
                        let _ = tx.send(Action::Error(e.user_message()));
                    }
                    let _ = tx.send(Action::RefreshEntries);
                });
            }
            Action::TimerUpdate(mut entries) => {
                for e in &mut entries {
                    e.project_code = self.project_codes.get(&e.project.id).cloned();
                }
                let View::Dashboard(d) = &mut self.current_view;
                d.update_running(entries);
            }
            Action::TodayEntriesUpdate(mut entries, _total, project_count) => {
                for e in &entries {
                    if let Some(ref code) = e.project_code {
                        self.project_codes.insert(e.project.id, code.clone());
                    }
                }
                for e in &mut entries {
                    if e.project_code.is_none() {
                        e.project_code = self.project_codes.get(&e.project.id).cloned();
                    }
                }
                let View::Dashboard(d) = &mut self.current_view;
                // Preserve existing project_count on light refreshes (day nav)
                // that don't fetch assignments (project_count = 0 from fetch_entries).
                let pc = project_count.max(d.project_count());
                d.update_entries(entries, pc);
            }
            Action::Refresh => {
                let View::Dashboard(d) = &mut self.current_view;
                d.set_loading(harv_core::t("tui-app-loading-sync"));
                let date = d.selected_date();
                self.fetch_dashboard_data(tx, true, date);
            }
            Action::RefreshEntries => {
                let View::Dashboard(d) = &self.current_view;
                let date = d.selected_date();
                self.fetch_entries(tx, date);
            }
            Action::NavigateDayPrev => {
                let View::Dashboard(d) = &mut self.current_view;
                d.prev_day();
                let date = d.selected_date();
                d.set_loading(harv_core::t("tui-app-loading-generic"));
                self.update_window_title();
                self.fetch_entries(tx, date);
            }
            Action::NavigateDayNext => {
                let View::Dashboard(d) = &mut self.current_view;
                d.next_day();
                let date = d.selected_date();
                d.set_loading(harv_core::t("tui-app-loading-generic"));
                self.update_window_title();
                self.fetch_entries(tx, date);
            }
            Action::NavigateDayToday => {
                let View::Dashboard(d) = &mut self.current_view;
                d.go_today();
                let date = d.selected_date();
                d.set_loading(harv_core::t("tui-app-loading-generic"));
                self.update_window_title();
                self.fetch_entries(tx, date);
            }
            Action::OpenDatePicker => {
                let initial = if let Some(ref form) = self.form {
                    harv_core::datetime::parse_date(form.date())
                        .unwrap_or_else(|_| harv_core::datetime::today())
                } else {
                    let View::Dashboard(d) = &self.current_view;
                    d.selected_date()
                };
                self.date_picker = Some(DatePicker::new(initial));
            }
            Action::CloseDatePicker => {
                self.date_picker = None;
            }
            Action::SelectDate(date) => {
                if let Some(ref mut form) = self.form {
                    form.set_date(harv_core::datetime::format_date(date));
                } else {
                    let View::Dashboard(d) = &mut self.current_view;
                    d.set_date(date);
                    d.set_loading(harv_core::t("tui-app-loading-generic"));
                    self.update_window_title();
                    self.fetch_entries(tx, date);
                }
                self.date_picker = None;
            }
            Action::StopTimer { entry_id } => {
                let View::Dashboard(d) = &mut self.current_view;
                d.set_loading(harv_core::t("tui-app-loading-stop"));
                let client = Arc::clone(&self.client);
                let tx = tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = client.time_entries().stop(entry_id).await {
                        let _ = tx.send(Action::Error(e.user_message()));
                    }
                    let _ = tx.send(Action::RefreshEntries);
                });
            }
            Action::DeleteEntry { entry_id } => {
                let View::Dashboard(d) = &mut self.current_view;
                d.set_loading(harv_core::t("tui-app-loading-delete"));
                let client = Arc::clone(&self.client);
                let tx = tx.clone();
                tokio::spawn(async move {
                    match client.time_entries().delete(entry_id).await {
                        Ok(()) => {
                            let _ = tx.send(Action::RefreshEntries);
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
                self.pending_confirm = Some((
                    harv_core::t_args("tui-app-confirm-delete", &[("desc", entry_desc)]),
                    vec![Action::DeleteEntry { entry_id }],
                ));
            }
            Action::ConfirmStopAndStart {
                entry_id,
                entry_desc,
            } => {
                self.pending_confirm = Some((
                    harv_core::t_args("tui-app-confirm-stop-start", &[("desc", entry_desc)]),
                    vec![Action::StopAndStartNew { entry_id }],
                ));
            }
            Action::StopAndStartNew { entry_id } => {
                let View::Dashboard(d) = &mut self.current_view;
                d.set_loading(harv_core::t("tui-app-loading-stop"));
                let client = Arc::clone(&self.client);
                let tx = tx.clone();
                tokio::spawn(async move {
                    let _ = client.time_entries().stop(entry_id).await;
                    let _ = tx.send(Action::OpenForm {
                        last_project_id: None,
                        last_task_id: None,
                        project_name: None,
                        mode: crate::action::FormMode::Start,
                        entry_id: None,
                        entry_date: None,
                        entry_hours: None,
                        entry_notes: None,
                        is_running: false,
                    });
                    let _ = tx.send(Action::RefreshEntries);
                });
            }
            Action::SetLoadingMessage(msg) => {
                let View::Dashboard(d) = &mut self.current_view;
                d.set_loading_msg(msg);
            }
            Action::Error(msg) => {
                tracing::error!("{}", msg);
            }
            _ => {}
        }
    }

    fn fetch_dashboard_data(
        &self,
        tx: &UnboundedSender<Action>,
        force_assignments: bool,
        date: NaiveDate,
    ) {
        let user_id = self.user_id;
        if user_id == 0 {
            return;
        }

        let client = Arc::clone(&self.client);
        let tx = tx.clone();

        tokio::spawn(async move {
            use harv_sdk::resources::time_entries::TimeEntryListParams;

            let params = TimeEntryListParams {
                user_id: Some(user_id),
                from: Some(date),
                to: Some(date),
                ..Default::default()
            };

            let _ = tx.send(Action::SetLoadingMessage(harv_core::t(
                "tui-app-loading-entries",
            )));
            let entries_result = client.time_entries().list(&params).await;

            match entries_result {
                Ok(mut entries) => {
                    let _ = tx.send(Action::SetLoadingMessage(harv_core::t(
                        "tui-app-loading-assignments",
                    )));
                    let assignments_result =
                        client.projects().my_assignments(force_assignments).await;

                    let mut project_count = 0usize;
                    if let Ok((assignments, total_projects)) = &assignments_result {
                        project_count = *total_projects;
                        let code_map: std::collections::HashMap<u64, &str> = assignments
                            .iter()
                            .filter_map(|a| {
                                a.project_code.as_ref().map(|c| (a.project.id, c.as_str()))
                            })
                            .collect();
                        for e in &mut entries {
                            e.project_code = code_map.get(&e.project.id).map(|&c| c.to_string());
                        }
                    }
                    let total: f64 = entries.iter().filter_map(|e| e.hours).sum();
                    let _ = tx.send(Action::TodayEntriesUpdate(entries, total, project_count));
                }
                Err(e) => {
                    let _ = tx.send(Action::Error(e.user_message()));
                    let _ = tx.send(Action::TodayEntriesUpdate(vec![], 0.0, 0));
                }
            };
        });
    }

    fn fetch_entries(&self, tx: &UnboundedSender<Action>, date: NaiveDate) {
        let user_id = self.user_id;
        if user_id == 0 {
            return;
        }

        let client = Arc::clone(&self.client);
        let tx = tx.clone();

        tokio::spawn(async move {
            use harv_sdk::resources::time_entries::TimeEntryListParams;

            let params = TimeEntryListParams {
                user_id: Some(user_id),
                from: Some(date),
                to: Some(date),
                ..Default::default()
            };

            match client.time_entries().list(&params).await {
                Ok(entries) => {
                    let total: f64 = entries.iter().filter_map(|e| e.hours).sum();
                    let _ = tx.send(Action::TodayEntriesUpdate(entries, total, 0));
                }
                Err(e) => {
                    let _ = tx.send(Action::Error(e.user_message()));
                }
            }
        });
    }

    fn render(&mut self, f: &mut Frame) {
        let area = f.area();

        let top_split = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);
        let (top_bar, rest) = (top_split[0], top_split[1]);

        let bottom_split = Layout::vertical([Constraint::Min(0), Constraint::Length(2)])
            .spacing(Spacing::Overlap(1))
            .split(rest);
        let (main, bottom_bar) = (bottom_split[0], bottom_split[1]);

        self.render_top_bar(top_bar, f);
        self.current_view.render(main, f, &self.theme, self.tick);
        self.render_bottom_bar(bottom_bar, f);

        self.help.render(area, f, &self.theme);

        if let Some(ref mut form) = self.form {
            form.render(area, f, &self.theme, self.tick);
        }

        if let Some(ref mut picker) = self.date_picker {
            picker.render(area, f, &self.theme);
        }

        if let Some((ref msg, _)) = self.pending_confirm {
            render_confirm_dialog(area, f, msg, &self.theme);
        }
    }

    fn render_top_bar(&self, area: Rect, f: &mut Frame) {
        let version = env!("CARGO_PKG_VERSION");

        // Left: harv name + version
        let left = Line::from(vec![
            Span::styled(
                format!(" {} ", harv_core::t("tui-app-title")),
                Style::new()
                    .fg(self.theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("v{} ", version), Style::new().fg(self.theme.muted)),
        ]);

        // Center: date nav with arrows
        let date = self.current_view.selected_date();
        let is_today = date == harv_core::datetime::today();
        let date_formatted =
            harv_core::datetime::format_date_header(date, &harv_core::current_langid());

        let mut date_spans = vec![
            Span::styled(" < ", Style::new().fg(self.theme.muted)),
            Span::styled(
                date_formatted,
                Style::new().fg(self.theme.fg).add_modifier(Modifier::BOLD),
            ),
        ];
        if is_today {
            date_spans.push(Span::styled(
                format!(" {} ", harv_core::t("tui-dash-today")),
                Style::new().fg(self.theme.muted),
            ));
        }
        date_spans.push(Span::styled(" > ", Style::new().fg(self.theme.muted)));
        let center = Line::from(date_spans);

        // Right: timer status
        let status = if self.current_view.timer_running() {
            Span::styled(
                format!(" {} ", harv_core::t("tui-app-running")),
                Style::new().fg(self.theme.success),
            )
        } else {
            Span::styled(
                format!(" {} ", harv_core::t("tui-app-idle")),
                Style::new().fg(self.theme.muted),
            )
        };
        let right = Line::from(status);

        // Fill entire bar area with background
        f.render_widget(
            Paragraph::new("").style(Style::new().bg(self.theme.bg)),
            area,
        );

        // Content row centered vertically, 3-column layout horizontally
        let content_row = area.centered_vertically(Constraint::Length(1));
        let cols = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .split(content_row);

        f.render_widget(
            Paragraph::new(left).style(Style::new().bg(self.theme.bg)),
            cols[0],
        );
        f.render_widget(
            Paragraph::new(center)
                .alignment(Alignment::Center)
                .style(Style::new().bg(self.theme.bg)),
            cols[1],
        );
        f.render_widget(
            Paragraph::new(right)
                .alignment(Alignment::Right)
                .style(Style::new().bg(self.theme.bg)),
            cols[2],
        );
    }

    fn render_bottom_bar(&self, area: Rect, f: &mut Frame) {
        let mut actions = vec![
            ("h/l", harv_core::t("tui-short-day")),
            ("g", harv_core::t("tui-short-pick")),
            ("n/t", harv_core::t("tui-short-new")),
            ("s", harv_core::t("tui-short-start")),
            ("e", harv_core::t("tui-short-edit")),
        ];

        if self.current_view.timer_running() {
            actions.push(("x", harv_core::t("tui-short-stop")));
        }

        actions.push(("d", harv_core::t("tui-short-del")));
        actions.push(("r", harv_core::t("tui-short-refr")));
        actions.push(("q", harv_core::t("tui-short-quit")));
        actions.push(("?", harv_core::t("tui-short-help")));

        let spans: Vec<Span> = actions
            .iter()
            .flat_map(|(key, label)| {
                vec![
                    Span::styled(
                        format!(" [{}] ", key),
                        Style::new()
                            .fg(self.theme.primary)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("{}  ", label), Style::new().fg(self.theme.muted)),
                ]
            })
            .collect();

        let block = Block::new()
            .borders(Borders::TOP)
            .border_style(Style::new().fg(self.theme.border))
            .merge_borders(MergeStrategy::Exact);

        let inner = block.inner(area);
        f.render_widget(block, area);
        f.render_widget(
            Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
            inner,
        );
    }

    fn update_window_title(&self) {
        let version = env!("CARGO_PKG_VERSION");
        let date = self.current_view.selected_date();
        let title = if date == harv_core::datetime::today() {
            format!("Harv {} — Today", version)
        } else {
            let d = harv_core::datetime::format_date_short(date, &harv_core::current_langid());
            format!("Harv {} — {}", version, d)
        };
        let _ = execute!(std::io::stdout(), SetTitle(&title));
    }
}

fn render_confirm_dialog(area: Rect, f: &mut Frame, msg: &str, theme: &Theme) {
    let max_width = 60u16;
    let popup_width = max_width.min(area.width.saturating_sub(4));
    let popup_height = 10u16.min(area.height.saturating_sub(2));

    let centered = crate::popup::centered_rect_fixed(popup_width, popup_height, area);
    f.render_widget(Clear, centered);

    let block = Block::new()
        .title(format!(" {} ", harv_core::t("tui-app-confirm-title")))
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

    let mut lines: Vec<Line> = msg
        .split('\n')
        .map(|part| {
            Line::from(Span::styled(
                harv_core::text::truncate(part, max_desc_width),
                Style::new().fg(theme.fg),
            ))
        })
        .collect();

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(" {} ", harv_core::t("tui-app-confirm-prompt")),
        Style::new().fg(theme.muted),
    )));

    f.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        inner_with_margin,
    );
}

async fn watch_theme_changes(tx: tokio::sync::mpsc::UnboundedSender<Action>) {
    #[cfg(target_os = "linux")]
    {
        use ashpd::desktop::settings::{ColorScheme, Settings};
        use futures_util::StreamExt;
        #[allow(clippy::collapsible_if)]
        if let Ok(settings) = Settings::new().await {
            if let Ok(mut stream) = settings.receive_color_scheme_changed().await {
                while let Some(scheme) = stream.next().await {
                    let mode = match scheme {
                        ColorScheme::PreferDark => ThemeMode::Dark,
                        ColorScheme::PreferLight | ColorScheme::NoPreference => ThemeMode::Light,
                    };
                    let _ = tx.send(Action::ThemeChanged(mode));
                }
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        let mut current = dark_light::detect().unwrap_or(dark_light::Mode::Dark);
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            if let Ok(detected) = dark_light::detect() {
                if detected != current {
                    current = detected;
                    let mode = match current {
                        dark_light::Mode::Dark => ThemeMode::Dark,
                        dark_light::Mode::Light | dark_light::Mode::Unspecified => ThemeMode::Light,
                    };
                    let _ = tx.send(Action::ThemeChanged(mode));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::FormMode;
    use chrono::NaiveDate;
    use harv_core::TimeEntry;
    use harv_sdk::mock_data;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use tokio::sync::mpsc;

    fn make_client() -> HarvClient {
        HarvClient::new(mock_data::test_config()).unwrap()
    }

    fn make_app() -> App {
        App::new_for_testing(make_client())
    }

    fn make_channel() -> (
        mpsc::UnboundedSender<Action>,
        mpsc::UnboundedReceiver<Action>,
    ) {
        mpsc::unbounded_channel()
    }

    #[test]
    fn test_new_initial_state() {
        let client = make_client();
        let resolved = ResolvedConfig::resolve(client.config(), None);
        let app = App::new(client, Theme::default(), resolved);
        assert_eq!(app.user_id, 0);
        assert!(app.user_name.is_none());
        assert!(app.form.is_none());
        assert!(app.date_picker.is_none());
        assert!(app.pending_confirm.is_none());
        assert_eq!(app.tick, 0);
    }

    #[test]
    fn test_new_for_testing_initial_state() {
        let app = make_app();
        assert_eq!(app.user_id, 0);
        assert!(app.user_name.is_none());
        assert!(!app.has_form());
    }

    #[test]
    fn test_user_id_getter_setter() {
        let mut app = make_app();
        assert_eq!(app.user_id(), 0);
        app.set_user_id(42);
        assert_eq!(app.user_id(), 42);
    }

    #[test]
    fn test_dispatch_tick_increments() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        assert_eq!(app.tick, 0);
        app.dispatch(Action::Tick, &tx);
        assert_eq!(app.tick, 1);
        app.dispatch(Action::Tick, &tx);
        assert_eq!(app.tick, 2);
    }

    #[test]
    fn test_dispatch_toggle_help() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        assert!(!app.help.is_visible());
        app.dispatch(Action::ToggleHelp, &tx);
        assert!(app.help.is_visible());
        app.dispatch(Action::ToggleHelp, &tx);
        assert!(!app.help.is_visible());
    }

    #[test]
    fn test_dispatch_theme_changed() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        let dark_bg = app.theme.bg;
        app.dispatch(Action::ThemeChanged(ThemeMode::Light), &tx);
        assert_ne!(app.theme.bg, dark_bg);
        app.dispatch(Action::ThemeChanged(ThemeMode::Dark), &tx);
        assert_eq!(app.theme.bg, dark_bg);
    }

    #[test]
    fn test_dispatch_open_date_picker() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        assert!(app.date_picker.is_none());
        app.dispatch(Action::OpenDatePicker, &tx);
        assert!(app.date_picker.is_some());
    }

    #[test]
    fn test_dispatch_close_date_picker() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        app.dispatch(Action::OpenDatePicker, &tx);
        assert!(app.date_picker.is_some());
        app.dispatch(Action::CloseDatePicker, &tx);
        assert!(app.date_picker.is_none());
    }

    #[test]
    fn test_dispatch_confirm_delete_sets_pending() {
        harv_core::init_locale(Some("en"));
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        assert!(app.pending_confirm.is_none());
        app.dispatch(
            Action::ConfirmDelete {
                entry_id: 1,
                entry_desc: "Test entry".into(),
            },
            &tx,
        );
        assert!(app.pending_confirm.is_some());
        let (_msg, actions) = app.pending_confirm.as_ref().unwrap();
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn test_dispatch_confirm_stop_and_start() {
        harv_core::init_locale(Some("en"));
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        app.dispatch(
            Action::ConfirmStopAndStart {
                entry_id: 7,
                entry_desc: "Running task".into(),
            },
            &tx,
        );
        assert!(app.pending_confirm.is_some());
        let (_msg, actions) = app.pending_confirm.as_ref().unwrap();
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn test_dispatch_set_loading_message() {
        let mut app = make_app();
        let (_tx, _rx) = make_channel();
        app.dispatch(Action::SetLoadingMessage("syncing".into()), &_tx);
        assert_eq!(app.dashboard().loading_msg_str(), "syncing");
    }

    #[test]
    fn test_dispatch_form_assignments_update_no_form() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        let assignments = vec![];
        app.dispatch(Action::FormAssignmentsUpdate(assignments), &tx);
        assert!(!app.has_form());
    }

    #[test]
    fn test_dispatch_timer_update() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        let entry = TimeEntry {
            id: 1,
            spent_date: None,
            hours: None,
            notes: None,
            is_running: true,
            timer_started_at: Some(chrono::Utc::now()),
            started_time: None,
            ended_time: None,
            project: harv_core::Reference {
                id: 10,
                name: "Test".into(),
            },
            task: harv_core::Reference {
                id: 20,
                name: "Dev".into(),
            },
            user: harv_core::Reference {
                id: 1,
                name: "User".into(),
            },
            client: None,
            is_billed: false,
            billable: true,
            project_code: None,
            billable_rate: None,
            cost_rate: None,
            created_at: None,
            updated_at: None,
        };
        app.dispatch(Action::TimerUpdate(vec![entry]), &tx);
        assert!(app.dashboard().has_running());
    }

    #[tokio::test]
    async fn test_dispatch_open_form() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        app.dispatch(
            Action::OpenForm {
                last_project_id: Some(100),
                last_task_id: Some(200),
                project_name: Some("Aqueduct".into()),
                mode: FormMode::Create,
                entry_id: None,
                entry_date: None,
                entry_hours: None,
                entry_notes: None,
                is_running: false,
            },
            &tx,
        );
        assert!(app.has_form());
    }

    #[tokio::test]
    async fn test_dispatch_error() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        app.dispatch(Action::Error("test error".into()), &tx);
        assert!(!app.has_form());
        assert!(!app.dashboard().has_running());
    }

    #[tokio::test]
    async fn test_dispatch_navigate_day_prev() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        let before = app.dashboard().selected_date();
        app.dispatch(Action::NavigateDayPrev, &tx);
        let after = app.dashboard().selected_date();
        assert!(after < before);
    }

    #[tokio::test]
    async fn test_dispatch_navigate_day_next() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        let before = app.dashboard().selected_date();
        app.dispatch(Action::NavigateDayNext, &tx);
        let after = app.dashboard().selected_date();
        assert!(after > before);
    }

    #[tokio::test]
    async fn test_dispatch_navigate_day_today() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        let today = harv_core::datetime::today();
        app.dispatch(Action::NavigateDayPrev, &tx);
        assert_ne!(app.dashboard().selected_date(), today);
        app.dispatch(Action::NavigateDayToday, &tx);
        assert_eq!(app.dashboard().selected_date(), today);
    }

    #[tokio::test]
    async fn test_dispatch_refresh() {
        harv_core::init_locale(Some("en"));
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        let before = app.dashboard().loading_msg_str().to_string();
        app.dispatch(Action::Refresh, &tx);
        assert_ne!(app.dashboard().loading_msg_str(), before);
    }

    #[tokio::test]
    async fn test_dispatch_refresh_entries() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        app.dispatch(Action::RefreshEntries, &tx);
        assert_eq!(app.dashboard().entry_count(), 0);
    }

    #[test]
    fn test_dispatch_select_date_without_form() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        let target = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        app.dispatch(Action::SelectDate(target), &tx);
        assert_eq!(app.dashboard().selected_date(), target);
    }

    #[tokio::test]
    async fn test_dispatch_select_date_and_close_picker() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        app.dispatch(Action::OpenDatePicker, &tx);
        assert!(app.date_picker.is_some());
        let target = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        app.dispatch(Action::SelectDate(target), &tx);
        assert!(app.date_picker.is_none());
    }

    #[test]
    fn test_handle_event_quit_key() {
        let mut app = make_app();
        let event =
            ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent::new(
                ratatui::crossterm::event::KeyCode::Char('q'),
                ratatui::crossterm::event::KeyModifiers::NONE,
            ));
        let actions = app.handle_event(event);
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], Action::Quit));
    }

    #[test]
    fn test_handle_event_help_key() {
        let mut app = make_app();
        let event =
            ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent::new(
                ratatui::crossterm::event::KeyCode::Char('?'),
                ratatui::crossterm::event::KeyModifiers::NONE,
            ));
        let actions = app.handle_event(event);
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], Action::ToggleHelp));
    }

    #[test]
    fn test_handle_event_ctrl_c() {
        let mut app = make_app();
        let event =
            ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent::new(
                ratatui::crossterm::event::KeyCode::Char('c'),
                ratatui::crossterm::event::KeyModifiers::CONTROL,
            ));
        let actions = app.handle_event(event);
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], Action::Quit));
    }

    #[test]
    fn test_handle_event_non_key_ignored() {
        let mut app = make_app();
        let event = ratatui::crossterm::event::Event::Resize(80, 24);
        let actions = app.handle_event(event);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_handle_event_pending_confirm_accept() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        app.dispatch(
            Action::ConfirmDelete {
                entry_id: 1,
                entry_desc: "test".into(),
            },
            &tx,
        );
        let event =
            ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent::new(
                ratatui::crossterm::event::KeyCode::Char('y'),
                ratatui::crossterm::event::KeyModifiers::NONE,
            ));
        let actions = app.handle_event(event);
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], Action::DeleteEntry { entry_id: 1 }));
        assert!(app.pending_confirm.is_none());
    }

    #[test]
    fn test_handle_event_pending_confirm_reject() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        app.dispatch(
            Action::ConfirmDelete {
                entry_id: 1,
                entry_desc: "test".into(),
            },
            &tx,
        );
        let event =
            ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent::new(
                ratatui::crossterm::event::KeyCode::Char('n'),
                ratatui::crossterm::event::KeyModifiers::NONE,
            ));
        let actions = app.handle_event(event);
        assert!(actions.is_empty());
        assert!(app.pending_confirm.is_none());
    }

    #[test]
    fn test_handle_event_with_date_picker_delegates() {
        let mut app = make_app();
        let (tx, _rx) = make_channel();
        app.dispatch(Action::OpenDatePicker, &tx);
        assert!(app.date_picker.is_some());
        let event =
            ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent::new(
                ratatui::crossterm::event::KeyCode::Enter,
                ratatui::crossterm::event::KeyModifiers::NONE,
            ));
        let actions = app.handle_event(event);
        assert!(!actions.is_empty());
        assert!(matches!(actions[0], Action::SelectDate(_)));
    }

    #[test]
    fn test_handle_event_with_help_open() {
        let mut app = make_app();
        app.help.toggle();
        assert!(app.help.is_visible());
        let event =
            ratatui::crossterm::event::Event::Key(ratatui::crossterm::event::KeyEvent::new(
                ratatui::crossterm::event::KeyCode::Char('?'),
                ratatui::crossterm::event::KeyModifiers::NONE,
            ));
        let _actions = app.handle_event(event);
        assert!(!app.help.is_visible());
    }

    #[test]
    fn test_render_confirm_dialog() {
        harv_core::init_locale(Some("en"));
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                render_confirm_dialog(f.area(), f, "Delete entry \"Test item\"?", &Theme::dark());
            })
            .unwrap();
    }
}
