use crate::action::{Action, FormMode};
use crate::theme::Theme;
use harv_core::TimeEntry;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListState, Paragraph};
use ratatui::Frame;

pub struct Dashboard {
    entries: Vec<TimeEntry>,
    running_entry: Option<TimeEntry>,
    daily_total: f64,
    list_state: ListState,
    loaded: bool,
}

impl Default for Dashboard {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            running_entry: None,
            daily_total: 0.0,
            list_state: ListState::default().with_selected(Some(0)),
            loaded: false,
        }
    }
}

impl Dashboard {
    pub fn set_loading(&mut self) {
        self.loaded = false;
    }

    pub fn update_entries(&mut self, entries: Vec<TimeEntry>) {
        self.loaded = true;
        self.running_entry = entries.iter().find(|e| e.is_running).cloned();
        self.daily_total = entries
            .iter()
            .filter(|e| !e.is_running)
            .filter_map(|e| e.hours)
            .sum();
        self.entries = entries;
    }

    pub fn update_running(&mut self, entries: Vec<TimeEntry>) {
        self.running_entry = entries.into_iter().next();
    }

    pub fn selected_entry(&self) -> Option<&TimeEntry> {
        self.list_state.selected().and_then(|i| self.entries.get(i))
    }

    pub fn render(&mut self, area: Rect, f: &mut Frame, theme: &Theme, tick: u64) {
        if !self.loaded {
            crate::loading::render_harv_loading(area, f, tick, "Loading entries...", theme);
            return;
        }

        if self.entries.is_empty() {
            render_harv_header(area, f);
            return;
        }

        let layout = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

        self.render_timer_header(layout[0], f, theme);
        self.render_entry_list(layout[1], f, theme);
    }

    fn render_timer_header(&self, area: Rect, f: &mut Frame, theme: &Theme) {
        let (indicator, status_text, elapsed) = if let Some(ref entry) = self.running_entry {
            let elapsed = format_timer_elapsed(entry);
            ("●", format!("RUNNING {}", elapsed), true)
        } else {
            ("○", "IDLE".into(), false)
        };

        let color = if elapsed { theme.success } else { theme.muted };

        let line = if let Some(ref entry) = self.running_entry {
            Line::from(vec![
                Span::styled(
                    format!(" {} ", indicator),
                    Style::new().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {} ", status_text),
                    Style::new().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {} · {} ", entry.project.name, entry.task.name),
                    Style::new().fg(theme.fg),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled(format!(" {} ", indicator), Style::new().fg(color)),
                Span::styled(format!(" {} ", status_text), Style::new().fg(color)),
                Span::styled(" No timer running ", Style::new().fg(theme.muted)),
            ])
        };

        let block = Block::new()
            .borders(Borders::BOTTOM)
            .border_style(Style::new().fg(theme.border));

        let paragraph = Paragraph::new(line)
            .block(block)
            .style(Style::new().bg(theme.bg));
        f.render_widget(paragraph, area);
    }

    fn render_entry_list(&mut self, area: Rect, f: &mut Frame, theme: &Theme) {
        let items: Vec<Line> = self
            .entries
            .iter()
            .map(|entry| format_entry_line(entry, theme))
            .collect();

        let title = if self.entries.is_empty() {
            "Today — No entries yet"
        } else {
            "Today"
        };

        let block = Block::new()
            .title(title)
            .title_bottom(format!(" {:.2}h total ", self.daily_total))
            .borders(Borders::RIGHT)
            .border_style(Style::new().fg(theme.border));

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::new()
                    .fg(theme.highlight)
                    .bg(theme.surface)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    pub fn handle_key(&mut self, key: &ratatui::crossterm::event::KeyEvent) -> Vec<Action> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let i = self
                    .list_state
                    .selected()
                    .map_or(0, |i| (i + 1).min(self.entries.len().saturating_sub(1)));
                self.list_state.select(Some(i));
                vec![]
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let i = self
                    .list_state
                    .selected()
                    .map_or(0, |i| i.saturating_sub(1));
                self.list_state.select(Some(i));
                vec![]
            }
            KeyCode::Char('s') => {
                if let Some(ref entry) = self.running_entry {
                    vec![Action::StopTimer { entry_id: entry.id }]
                } else {
                    vec![Action::OpenForm {
                        last_project_id: None,
                        last_task_id: None,
                        project_name: None,
                        mode: FormMode::Start,
                        entry_id: None,
                        entry_date: None,
                        entry_hours: None,
                        entry_notes: None,
                    }]
                }
            }
            KeyCode::Char('d') => {
                if let Some(entry) = self.selected_entry() {
                    let desc = format!(
                        "{} · {} ({})",
                        entry.project.name,
                        entry.task.name,
                        if entry.is_running {
                            "running"
                        } else {
                            "stopped"
                        }
                    );
                    return vec![Action::ConfirmDelete {
                        entry_id: entry.id,
                        entry_desc: desc,
                    }];
                }
                vec![]
            }
            KeyCode::Char('e') | KeyCode::Enter => match self.selected_entry() {
                Some(entry) if entry.is_running => {
                    vec![Action::StopTimer { entry_id: entry.id }]
                }
                Some(entry) => {
                    let date = entry
                        .spent_date
                        .map(harv_core::datetime::format_date)
                        .unwrap_or_else(|| {
                            harv_core::datetime::format_date(harv_core::datetime::today())
                        });
                    let hours = entry.hours.map(|h| format!("{:.2}", h));
                    let notes = entry.notes.clone();
                    vec![Action::OpenForm {
                        last_project_id: Some(entry.project.id),
                        last_task_id: Some(entry.task.id),
                        project_name: Some(entry.project.name.clone()),
                        mode: FormMode::Edit,
                        entry_id: Some(entry.id),
                        entry_date: Some(date),
                        entry_hours: hours,
                        entry_notes: notes,
                    }]
                }
                None => vec![],
            },
            KeyCode::Char('n') | KeyCode::Char('t') => {
                vec![Action::OpenForm {
                    last_project_id: None,
                    last_task_id: None,
                    project_name: None,
                    mode: FormMode::Create,
                    entry_id: None,
                    entry_date: None,
                    entry_hours: None,
                    entry_notes: None,
                }]
            }
            KeyCode::Char('r') => vec![Action::Refresh],
            _ => vec![],
        }
    }
}

fn format_entry_line(entry: &TimeEntry, theme: &Theme) -> Line<'static> {
    let is_running = entry.is_running;
    let bullet = if is_running { "●" } else { "○" };
    let bullet_color = if is_running {
        theme.success
    } else {
        theme.muted
    };

    let hours = match (is_running, entry.hours) {
        (true, _) => format_timer_elapsed(entry),
        (false, Some(h)) => harv_core::text::format_hours(h),
        (false, None) => "0.00h".into(),
    };

    let client = entry
        .client
        .as_ref()
        .map(|c| format!("{} · ", c.name))
        .unwrap_or_default();

    let text = format!(
        "{}{} · {}    {}",
        client, entry.project.name, entry.task.name, hours
    );

    let line = vec![
        Span::styled(
            format!(" {} ", bullet),
            Style::new().fg(bullet_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(text, Style::new().fg(theme.fg)),
    ];

    Line::from(line)
}

fn format_timer_elapsed(entry: &TimeEntry) -> String {
    if let Some(started) = entry.timer_started_at {
        let elapsed = chrono::Utc::now() - started;
        harv_core::text::format_elapsed_hms(elapsed.num_seconds())
    } else {
        String::from("--:--:--")
    }
}

fn render_harv_header(area: Rect, f: &mut Frame) {
    use ratatui::widgets::Clear;

    f.render_widget(Clear, area);

    let shades: [(u8, u8, u8); 4] = [
        (250, 210, 140),
        (250, 170, 90),
        (250, 130, 40),
        (250, 93, 0),
    ];

    let header_lines = [
        "▗▖ ▗▖ ▗▄▖ ▗▄▄▖ ▗▖  ▗▖",
        "▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌▐▌  ▐▌",
        "▐▛▀▜▌▐▛▀▜▌▐▛▀▚▖▐▌  ▐▌",
        "▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌ ▝▚▞▘ ",
    ];

    let version = env!("CARGO_PKG_VERSION");
    let version_text = format!("HARV CLI v{}", version);
    let pad = (21usize.saturating_sub(version_text.len())) / 2;

    let mut lines: Vec<Line> = Vec::new();

    for (i, header_line) in header_lines.iter().enumerate() {
        let (r, g, b) = shades[i];
        lines.push(Line::from(Span::styled(
            header_line.to_string(),
            Style::new()
                .fg(Color::Rgb(r, g, b))
                .add_modifier(Modifier::BOLD),
        )));
    }

    lines.push(Line::from(vec![
        Span::styled(
            format!("{}{}", " ".repeat(pad), "HARV CLI"),
            Style::new()
                .fg(Color::Rgb(250, 93, 0))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" v{}", version),
            Style::new().fg(Color::Rgb(160, 160, 160)),
        ),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        "No entries today. Press n to start tracking!",
        Style::new().fg(Color::Rgb(160, 160, 160)),
    )));

    let content_height = lines.len() as u16;

    // Vertically center the content within the area
    let v_margin = area.height.saturating_sub(content_height) / 2;
    let vertical = Layout::vertical([
        Constraint::Length(v_margin),
        Constraint::Length(content_height),
        Constraint::Min(v_margin),
    ])
    .split(area);

    // Horizontally center: 27 cols wide
    let h_margin = area.width.saturating_sub(27) / 2;
    let horizontal = Layout::horizontal([
        Constraint::Length(h_margin),
        Constraint::Length(27),
        Constraint::Min(h_margin),
    ])
    .split(vertical[1]);

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(paragraph, horizontal[1]);
}
