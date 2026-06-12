use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::NaiveDate;
use futures_util::StreamExt;
use harv_core::CreateTimeEntry;
use harv_sdk::HarvClient;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::layout::Alignment;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
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
}

impl App {
    pub fn new(client: HarvClient, theme: Theme) -> Self {
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
        }
    }

    // Test helpers
    #[doc(hidden)]
    pub fn new_for_testing(client: HarvClient) -> Self {
        Self::new(client, Theme::default())
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
                let pid = last_project_id.or(self.client.config().last_project_id);
                let tid = last_task_id.or(self.client.config().last_task_id);
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
            Action::TodayEntriesUpdate(entries, _total) => {
                self.project_codes.clear();
                for e in &entries {
                    if let Some(ref code) = e.project_code {
                        self.project_codes.insert(e.project.id, code.clone());
                    }
                }
                let View::Dashboard(d) = &mut self.current_view;
                d.update_entries(entries);
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
                self.fetch_entries(tx, date);
            }
            Action::NavigateDayNext => {
                let View::Dashboard(d) = &mut self.current_view;
                d.next_day();
                let date = d.selected_date();
                d.set_loading(harv_core::t("tui-app-loading-generic"));
                self.fetch_entries(tx, date);
            }
            Action::NavigateDayToday => {
                let View::Dashboard(d) = &mut self.current_view;
                d.go_today();
                let date = d.selected_date();
                d.set_loading(harv_core::t("tui-app-loading-generic"));
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
                    self.fetch_entries(tx, date);
                }
                self.date_picker = None;
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
                    let _ = tx.send(Action::RefreshEntries);
                });
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

            let time_api = client.time_entries();
            let projects_api = client.projects();
            let (entries_result, assignments_result) = tokio::join!(
                time_api.list(&params),
                projects_api.my_assignments(force_assignments),
            );

            match entries_result {
                Ok(mut entries) => {
                    if let Ok(assignments) = &assignments_result {
                        let code_map: std::collections::HashMap<u64, &str> = assignments
                            .iter()
                            .filter_map(|a| {
                                a.project_code
                                    .as_ref()
                                    .map(|c| (a.project.id, c.as_str()))
                            })
                            .collect();
                        for e in &mut entries {
                            e.project_code =
                                code_map.get(&e.project.id).map(|&c| c.to_string());
                        }
                    }
                    let total: f64 = entries.iter().filter_map(|e| e.hours).sum();
                    let _ = tx.send(Action::TodayEntriesUpdate(entries, total));
                }
                Err(e) => {
                    let _ = tx.send(Action::Error(e.user_message()));
                }
            }
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
            Constraint::Length(2),
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

        if let Some(ref mut picker) = self.date_picker {
            picker.render(area, f, &self.theme);
        }

        if let Some((ref msg, _)) = self.pending_confirm {
            render_confirm_dialog(area, f, msg, &self.theme);
        }
    }

    fn render_top_bar(&self, area: Rect, f: &mut Frame) {
        let layout = Layout::horizontal([Constraint::Min(0), Constraint::Length(12)]).split(area);

        let version = env!("CARGO_PKG_VERSION");
        let mut spans = vec![Span::styled(
            format!(" {} ", harv_core::t("tui-app-title")),
            Style::new()
                .fg(self.theme.primary)
                .add_modifier(Modifier::BOLD),
        )];

        if let Some(ref name) = self.user_name {
            spans.push(Span::styled(
                format!("{} ", name),
                Style::new().fg(self.theme.fg),
            ));
        }

        spans.push(Span::styled(
            format!("v{} ", version),
            Style::new().fg(self.theme.muted),
        ));

        let left = Line::from(spans);

        let status = if self.current_view.timer_running() {
            Span::styled(
                format!(" {}", harv_core::t("tui-app-running")),
                Style::new().fg(self.theme.success),
            )
        } else {
            Span::styled(
                format!(" {}", harv_core::t("tui-app-idle")),
                Style::new().fg(self.theme.muted),
            )
        };

        f.render_widget(
            Paragraph::new(left).style(Style::new().bg(self.theme.bg)),
            layout[0],
        );

        f.render_widget(
            Paragraph::new(Line::from(status)).style(Style::new().bg(self.theme.bg)),
            layout[1],
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
            .border_style(Style::new().fg(self.theme.border));

        let paragraph = Paragraph::new(Line::from(spans))
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
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
