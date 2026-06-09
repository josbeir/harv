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
            render_harv_header(area, f, theme);
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

fn render_harv_header(area: Rect, f: &mut Frame, theme: &Theme) {
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

    let version_color = match theme.mode {
        crate::theme::ThemeMode::Dark => Color::Rgb(160, 160, 160),
        crate::theme::ThemeMode::Light => Color::Rgb(90, 90, 90),
    };

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
        Span::styled(format!(" v{}", version), Style::new().fg(version_color)),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        "No entries today. Press n to start tracking!",
        Style::new().fg(version_color),
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

impl Dashboard {
    #[doc(hidden)]
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    #[doc(hidden)]
    pub fn has_running(&self) -> bool {
        self.running_entry.is_some()
    }

    #[doc(hidden)]
    pub fn set_loaded(&mut self, v: bool) {
        self.loaded = v;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use harv_core::Reference;

    fn ref_(id: u64, name: &str) -> Reference {
        Reference {
            id,
            name: name.into(),
        }
    }

    fn entry(
        id: u64,
        project_id: u64,
        task_id: u64,
        hours: Option<f64>,
        is_running: bool,
    ) -> TimeEntry {
        TimeEntry {
            id,
            spent_date: None,
            hours,
            notes: None,
            is_running,
            timer_started_at: if is_running { Some(Utc::now()) } else { None },
            started_time: None,
            ended_time: None,
            project: ref_(project_id, "Project"),
            task: ref_(task_id, "Task"),
            user: ref_(1, "User"),
            client: None,
            is_billed: false,
            billable: true,
            billable_rate: None,
            cost_rate: None,
            created_at: None,
            updated_at: None,
        }
    }

    fn key_press(code: KeyCode) -> ratatui::crossterm::event::KeyEvent {
        ratatui::crossterm::event::KeyEvent::new(
            code,
            ratatui::crossterm::event::KeyModifiers::NONE,
        )
    }

    #[test]
    fn test_default_state() {
        let d = Dashboard::default();
        assert!(!d.loaded);
        assert!(d.entries.is_empty());
        assert!(d.running_entry.is_none());
        assert_eq!(d.daily_total, 0.0);
    }

    #[test]
    fn test_update_entries_sets_data() {
        let mut d = Dashboard::default();
        let entries = vec![
            entry(1, 10, 20, Some(2.5), false),
            entry(2, 11, 21, None, true),
        ];
        d.update_entries(entries);
        assert!(d.loaded);
        assert_eq!(d.entries.len(), 2);
        assert!(d.running_entry.is_some());
        assert_eq!(d.running_entry.unwrap().id, 2);
    }

    #[test]
    fn test_daily_total_excludes_running() {
        let mut d = Dashboard::default();
        d.update_entries(vec![
            entry(1, 10, 20, Some(2.5), false),
            entry(2, 11, 21, Some(1.5), false),
            entry(3, 12, 22, None, true),
        ]);
        assert!((d.daily_total - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_update_running_preserves_entries() {
        let mut d = Dashboard::default();
        d.update_entries(vec![entry(1, 10, 20, Some(2.5), false)]);
        d.update_running(vec![entry(99, 10, 20, None, true)]);
        assert_eq!(d.entries.len(), 1);
        assert!(d.running_entry.is_some());
        assert_eq!(d.running_entry.unwrap().id, 99);
    }

    #[test]
    fn test_selected_entry() {
        let mut d = Dashboard::default();
        d.update_entries(vec![
            entry(1, 10, 20, Some(1.0), false),
            entry(2, 11, 21, Some(2.0), false),
        ]);
        d.list_state.select(Some(0));
        assert_eq!(d.selected_entry().unwrap().id, 1);
        d.list_state.select(Some(1));
        assert_eq!(d.selected_entry().unwrap().id, 2);
        d.list_state.select(None);
        assert!(d.selected_entry().is_none());
    }

    #[test]
    fn test_set_loading() {
        let mut d = Dashboard::default();
        d.update_entries(vec![entry(1, 10, 20, Some(1.0), false)]);
        assert!(d.loaded);
        d.set_loading();
        assert!(!d.loaded);
    }

    #[test]
    fn test_navigate_down() {
        let mut d = Dashboard::default();
        d.update_entries(vec![
            entry(1, 10, 20, Some(1.0), false),
            entry(2, 11, 21, Some(2.0), false),
            entry(3, 12, 22, Some(3.0), false),
        ]);
        d.handle_key(&key_press(KeyCode::Char('j')));
        assert_eq!(d.list_state.selected(), Some(1));
        d.handle_key(&key_press(KeyCode::Char('j')));
        assert_eq!(d.list_state.selected(), Some(2));
        d.handle_key(&key_press(KeyCode::Char('j')));
        assert_eq!(d.list_state.selected(), Some(2));
    }

    #[test]
    fn test_navigate_up() {
        let mut d = Dashboard::default();
        d.update_entries(vec![
            entry(1, 10, 20, Some(1.0), false),
            entry(2, 11, 21, Some(2.0), false),
        ]);
        d.list_state.select(Some(1));
        d.handle_key(&key_press(KeyCode::Char('k')));
        assert_eq!(d.list_state.selected(), Some(0));
        d.handle_key(&key_press(KeyCode::Char('k')));
        assert_eq!(d.list_state.selected(), Some(0));
    }

    #[test]
    fn test_s_stops_running() {
        let mut d = Dashboard::default();
        d.update_entries(vec![entry(1, 10, 20, None, true)]);
        let actions = d.handle_key(&key_press(KeyCode::Char('s')));
        assert!(matches!(actions[0], Action::StopTimer { entry_id: 1 }));
    }

    #[test]
    fn test_s_no_timer_opens_start_form() {
        let mut d = Dashboard::default();
        d.update_entries(vec![entry(1, 10, 20, Some(1.0), false)]);
        let actions = d.handle_key(&key_press(KeyCode::Char('s')));
        assert!(matches!(
            actions[0],
            Action::OpenForm {
                mode: FormMode::Start,
                ..
            }
        ));
    }

    #[test]
    fn test_n_opens_create_form() {
        let mut d = Dashboard::default();
        let actions = d.handle_key(&key_press(KeyCode::Char('n')));
        assert!(matches!(
            actions[0],
            Action::OpenForm {
                mode: FormMode::Create,
                ..
            }
        ));
    }

    #[test]
    fn test_e_opens_edit_form() {
        let mut d = Dashboard::default();
        d.update_entries(vec![entry(42, 10, 20, Some(2.0), false)]);
        d.list_state.select(Some(0));
        let actions = d.handle_key(&key_press(KeyCode::Char('e')));
        assert!(matches!(
            actions[0],
            Action::OpenForm {
                mode: FormMode::Edit,
                entry_id: Some(42),
                ..
            }
        ));
    }

    #[test]
    fn test_d_triggers_confirm_delete() {
        let mut d = Dashboard::default();
        d.update_entries(vec![entry(42, 10, 20, Some(2.0), false)]);
        d.list_state.select(Some(0));
        let actions = d.handle_key(&key_press(KeyCode::Char('d')));
        assert!(matches!(
            actions[0],
            Action::ConfirmDelete { entry_id: 42, .. }
        ));
    }

    #[test]
    fn test_r_returns_refresh() {
        let mut d = Dashboard::default();
        let actions = d.handle_key(&key_press(KeyCode::Char('r')));
        assert!(matches!(actions[0], Action::Refresh));
    }

    #[test]
    fn test_enter_on_running_stops() {
        let mut d = Dashboard::default();
        d.update_entries(vec![entry(1, 10, 20, None, true)]);
        let actions = d.handle_key(&key_press(KeyCode::Enter));
        assert!(matches!(actions[0], Action::StopTimer { entry_id: 1 }));
    }

    #[test]
    fn test_enter_no_selection_returns_empty() {
        let mut d = Dashboard::default();
        let actions = d.handle_key(&key_press(KeyCode::Enter));
        assert!(actions.is_empty());
    }
}
