use crate::action::{Action, FormMode};
use crate::theme::Theme;
use chrono::NaiveDate;
use harv_core::TimeEntry;
use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Alignment, Constraint, Layout, Rect, Spacing};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::merge::MergeStrategy;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

const HOURS_COL_WIDTH: usize = 5;

pub struct Dashboard {
    entries: Vec<TimeEntry>,
    running_entry: Option<TimeEntry>,
    daily_total: f64,
    selected_index: usize,
    loaded: bool,
    loading_msg: String,
    selected_date: NaiveDate,
    project_count: usize,
}

impl Default for Dashboard {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            running_entry: None,
            daily_total: 0.0,
            selected_index: 0,
            loaded: false,
            loading_msg: harv_core::t("tui-app-loading-generic"),
            selected_date: harv_core::datetime::today(),
            project_count: 0,
        }
    }
}

impl Dashboard {
    pub fn set_loading(&mut self, msg: String) {
        self.loaded = false;
        self.loading_msg = msg;
    }

    pub fn set_loading_msg(&mut self, msg: String) {
        self.loading_msg = msg;
    }

    pub fn update_entries(&mut self, entries: Vec<TimeEntry>, project_count: usize) {
        self.loaded = true;
        self.running_entry = entries.iter().find(|e| e.is_running).cloned();
        self.daily_total = entries
            .iter()
            .filter(|e| !e.is_running)
            .filter_map(|e| e.hours)
            .sum();
        self.entries = entries;
        self.project_count = project_count;
        self.selected_index = 0;
    }

    pub fn update_running(&mut self, entries: Vec<TimeEntry>) {
        self.running_entry = entries.into_iter().next();
    }

    pub fn selected_entry(&self) -> Option<&TimeEntry> {
        self.entries.get(self.selected_index)
    }

    pub fn selected_date(&self) -> NaiveDate {
        self.selected_date
    }

    pub fn project_count(&self) -> usize {
        self.project_count
    }

    pub fn set_date(&mut self, date: NaiveDate) {
        self.selected_date = date;
    }

    pub fn prev_day(&mut self) {
        self.selected_date -= chrono::Duration::days(1);
    }

    pub fn next_day(&mut self) {
        self.selected_date += chrono::Duration::days(1);
    }

    pub fn go_today(&mut self) {
        self.selected_date = harv_core::datetime::today();
    }

    pub fn render(&mut self, area: Rect, f: &mut Frame, theme: &Theme, tick: u64) {
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).split(area);

        if !self.loaded {
            crate::loading::render_harv_loading(layout[1], f, tick, &self.loading_msg, theme);
            return;
        }

        if self.entries.is_empty() {
            let inner = Layout::vertical([Constraint::Min(0), Constraint::Length(3)])
                .spacing(Spacing::Overlap(1))
                .split(layout[1]);
            render_harv_header(inner[0], f, theme, self.selected_date);
            self.render_stats_footer(inner[1], f, theme);
            return;
        }

        let body = Layout::vertical([Constraint::Min(0), Constraint::Length(3)])
            .spacing(Spacing::Overlap(1))
            .split(layout[1]);

        self.render_entry_list(body[0], f, theme);
        self.render_stats_footer(body[1], f, theme);
    }

    fn render_entry_list(&mut self, area: Rect, f: &mut Frame, theme: &Theme) {
        let border_style = Style::new().fg(theme.border);
        let muted_style = Style::new().fg(theme.muted);
        let col_pad = " ".repeat(HOURS_COL_WIDTH + 2);

        let constraints: Vec<Constraint> = self
            .entries
            .iter()
            .map(|e| {
                let has_notes = e.notes.as_deref().is_some_and(|n| !n.is_empty());
                let lines: u16 = if has_notes { 3 } else { 2 };
                Constraint::Length(lines + 2) // +2 for TOP + BOTTOM borders
            })
            .collect();

        if constraints.is_empty() {
            return;
        }

        let entry_areas = Layout::vertical(constraints)
            .spacing(Spacing::Overlap(1))
            .split(area);

        for (i, (entry, entry_area)) in self.entries.iter().zip(entry_areas.iter()).enumerate() {
            let has_notes = entry.notes.as_deref().is_some_and(|n| !n.is_empty());
            let is_selected = i == self.selected_index;

            let block = if is_selected {
                Block::new()
                    .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM | Borders::RIGHT)
                    .border_style(border_style)
                    .merge_borders(MergeStrategy::Exact)
                    .style(Style::new().bg(theme.surface))
            } else {
                Block::new()
                    .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM | Borders::RIGHT)
                    .border_style(border_style)
                    .merge_borders(MergeStrategy::Exact)
            };
            let inner = block.inner(*entry_area);

            let running = entry.is_running;
            let hours_display = if running {
                format_timer_compact(entry)
            } else {
                let s = entry.hours.map_or_else(
                    || "0:00".to_string(),
                    harv_core::text::decimal_hours_to_hhmm,
                );
                format!("{:>5}", s)
            };
            let hours_style = if running {
                Style::new().fg(theme.success).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(theme.fg)
            };

            let display_name = harv_core::text::format_project_display(
                &entry.project.name,
                entry.project_code.as_deref(),
            );

            let mut line1_spans = vec![
                Span::styled(hours_display, hours_style),
                Span::raw("  "),
                Span::styled(display_name, Style::new().fg(theme.fg)),
            ];
            if let Some(ref client) = entry.client {
                line1_spans.push(Span::styled(format!(" | {}", client.name), muted_style));
            }

            let line2_spans = vec![
                Span::raw(&col_pad),
                Span::styled(format!("└─ {}", entry.task.name), muted_style),
            ];

            let mut entry_lines = vec![Line::from(line1_spans), Line::from(line2_spans)];

            if has_notes {
                let line3_spans = vec![
                    Span::raw(&col_pad),
                    Span::styled(
                        format!("└─ {}", entry.notes.as_deref().unwrap()),
                        muted_style,
                    ),
                ];
                entry_lines.push(Line::from(line3_spans));
            }

            f.render_widget(block, *entry_area);
            let para_style = if is_selected {
                Style::new().bg(theme.surface)
            } else {
                Style::default()
            };
            f.render_widget(Paragraph::new(entry_lines).style(para_style), inner);
        }
    }

    fn render_stats_footer(&self, area: Rect, f: &mut Frame, theme: &Theme) {
        let block = Block::new()
            .borders(Borders::TOP | Borders::BOTTOM)
            .border_style(Style::new().fg(theme.border))
            .merge_borders(MergeStrategy::Exact);
        let inner = block.inner(area);

        let hours = format!(
            "{} {}",
            harv_core::text::decimal_hours_to_hhmm(self.daily_total),
            harv_core::t("tui-dash-stats-total"),
        );
        let projects = format!(
            "{} {}",
            self.project_count,
            harv_core::t("tui-dash-projects"),
        );

        let line = Line::from(vec![
            Span::styled(
                hours,
                Style::new().fg(theme.primary).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  ·  "),
            Span::styled(projects, Style::new().fg(theme.muted)),
        ]);

        let paragraph = Paragraph::new(line).alignment(Alignment::Center);

        f.render_widget(block, area);
        if inner.height > 0 {
            f.render_widget(paragraph, inner);
        }
    }

    pub fn handle_key(&mut self, key: &ratatui::crossterm::event::KeyEvent) -> Vec<Action> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let max = self.entries.len().saturating_sub(1);
                self.selected_index = (self.selected_index + 1).min(max);
                vec![]
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected_index = self.selected_index.saturating_sub(1);
                vec![]
            }
            KeyCode::Char('s') => {
                if let Some(ref entry) = self.running_entry {
                    let desc = format!(
                        "{} · {}",
                        harv_core::text::format_project_display(
                            &entry.project.name,
                            entry.project_code.as_deref()
                        ),
                        entry.task.name
                    );
                    vec![Action::ConfirmStopAndStart {
                        entry_id: entry.id,
                        entry_desc: desc,
                    }]
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
                        is_running: false,
                    }]
                }
            }
            KeyCode::Char('x') => {
                if let Some(ref entry) = self.running_entry {
                    vec![Action::StopTimer { entry_id: entry.id }]
                } else {
                    vec![]
                }
            }
            KeyCode::Char('d') => {
                if let Some(entry) = self.selected_entry() {
                    let entry_desc = format!(
                        "{} · {} ({})",
                        harv_core::text::format_project_display(
                            &entry.project.name,
                            entry.project_code.as_deref()
                        ),
                        entry.task.name,
                        if entry.is_running {
                            harv_core::t("tui-dash-desc-running")
                        } else {
                            harv_core::t("tui-dash-desc-stopped")
                        }
                    );
                    return vec![Action::ConfirmDelete {
                        entry_id: entry.id,
                        entry_desc,
                    }];
                }
                vec![]
            }
            KeyCode::Char('e') | KeyCode::Enter => match self.selected_entry() {
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
                        is_running: entry.is_running,
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
                    is_running: false,
                }]
            }
            KeyCode::Char('r') => vec![Action::Refresh],
            KeyCode::Char('h') | KeyCode::Left => vec![Action::NavigateDayPrev],
            KeyCode::Char('l') | KeyCode::Right => vec![Action::NavigateDayNext],
            KeyCode::Char('T') => vec![Action::NavigateDayToday],
            KeyCode::Char('g') => vec![Action::OpenDatePicker],
            _ => vec![],
        }
    }
}

fn format_timer_compact(entry: &TimeEntry) -> String {
    if let Some(started) = entry.timer_started_at {
        let secs = (chrono::Utc::now() - started).num_seconds().max(0);
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        if h > 0 {
            format!("{:>4}:{:02}", h, m)
        } else {
            format!("{:>4}:{:02}", m, s)
        }
    } else {
        "--:--".to_string()
    }
}

fn render_harv_header(area: Rect, f: &mut Frame, theme: &Theme, date: NaiveDate) {
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
    let version_text = format!("{} v{}", harv_core::t("tui-app-title"), version);
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
        Span::styled(" ".repeat(pad), Style::default()),
        Span::styled(
            harv_core::t("tui-app-title"),
            Style::new()
                .fg(Color::Rgb(250, 93, 0))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" v{}", version), Style::new().fg(version_color)),
        Span::styled(
            " ".repeat(21usize.saturating_sub(version_text.len() + pad)),
            Style::default(),
        ),
    ]));
    let empty_msg = if date == harv_core::datetime::today() {
        harv_core::t("tui-dash-empty-today")
    } else {
        harv_core::t_args(
            "tui-dash-empty-past",
            &[(
                "date",
                harv_core::datetime::format_date_short(date, &harv_core::current_langid()),
            )],
        )
    };

    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        empty_msg,
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
            project_code: None,
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
    fn test_project_count_tracks_value() {
        let mut d = Dashboard::default();
        assert_eq!(d.project_count(), 0);
        d.update_entries(vec![], 42);
        assert_eq!(d.project_count(), 42);
    }

    #[test]
    fn test_update_entries_sets_data() {
        let mut d = Dashboard::default();
        let entries = vec![
            entry(1, 10, 20, Some(2.5), false),
            entry(2, 11, 21, None, true),
        ];
        d.update_entries(entries, 0);
        assert!(d.loaded);
        assert_eq!(d.entries.len(), 2);
        assert!(d.running_entry.is_some());
        assert_eq!(d.running_entry.unwrap().id, 2);
    }

    #[test]
    fn test_daily_total_excludes_running() {
        let mut d = Dashboard::default();
        d.update_entries(
            vec![
                entry(1, 10, 20, Some(2.5), false),
                entry(2, 11, 21, Some(1.5), false),
                entry(3, 12, 22, None, true),
            ],
            0,
        );
        assert!((d.daily_total - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_update_running_preserves_entries() {
        let mut d = Dashboard::default();
        d.update_entries(vec![entry(1, 10, 20, Some(2.5), false)], 0);
        d.update_running(vec![entry(99, 10, 20, None, true)]);
        assert_eq!(d.entries.len(), 1);
        assert!(d.running_entry.is_some());
        assert_eq!(d.running_entry.unwrap().id, 99);
    }

    #[test]
    fn test_selected_entry() {
        let mut d = Dashboard::default();
        d.update_entries(
            vec![
                entry(1, 10, 20, Some(1.0), false),
                entry(2, 11, 21, Some(2.0), false),
            ],
            0,
        );
        d.selected_index = 0;
        assert_eq!(d.selected_entry().unwrap().id, 1);
        d.selected_index = 1;
        assert_eq!(d.selected_entry().unwrap().id, 2);
    }

    #[test]
    fn test_set_loading() {
        let mut d = Dashboard::default();
        d.update_entries(vec![entry(1, 10, 20, Some(1.0), false)], 0);
        assert!(d.loaded);
        d.set_loading("Test".to_string());
        assert!(!d.loaded);
        assert_eq!(d.loading_msg, "Test");
    }

    #[test]
    fn test_navigate_down() {
        let mut d = Dashboard::default();
        d.update_entries(
            vec![
                entry(1, 10, 20, Some(1.0), false),
                entry(2, 11, 21, Some(2.0), false),
                entry(3, 12, 22, Some(3.0), false),
            ],
            0,
        );
        d.handle_key(&key_press(KeyCode::Char('j')));
        assert_eq!(d.selected_index, 1);
        d.handle_key(&key_press(KeyCode::Char('j')));
        assert_eq!(d.selected_index, 2);
        d.handle_key(&key_press(KeyCode::Char('j')));
        assert_eq!(d.selected_index, 2);
    }

    #[test]
    fn test_navigate_up() {
        let mut d = Dashboard::default();
        d.update_entries(
            vec![
                entry(1, 10, 20, Some(1.0), false),
                entry(2, 11, 21, Some(2.0), false),
            ],
            0,
        );
        d.selected_index = 1;
        d.handle_key(&key_press(KeyCode::Char('k')));
        assert_eq!(d.selected_index, 0);
        d.handle_key(&key_press(KeyCode::Char('k')));
        assert_eq!(d.selected_index, 0);
    }

    #[test]
    fn test_s_stops_running() {
        let mut d = Dashboard::default();
        d.update_entries(vec![entry(1, 10, 20, None, true)], 0);
        let actions = d.handle_key(&key_press(KeyCode::Char('s')));
        assert!(matches!(
            actions[0],
            Action::ConfirmStopAndStart { entry_id: 1, .. }
        ));
    }

    #[test]
    fn test_s_no_timer_opens_start_form() {
        let mut d = Dashboard::default();
        d.update_entries(vec![entry(1, 10, 20, Some(1.0), false)], 0);
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
    fn test_x_stops_running_timer() {
        let mut d = Dashboard::default();
        d.update_entries(vec![entry(1, 10, 20, None, true)], 0);
        let actions = d.handle_key(&key_press(KeyCode::Char('x')));
        assert!(matches!(actions[0], Action::StopTimer { entry_id: 1 }));
    }

    #[test]
    fn test_x_idle_does_nothing() {
        let mut d = Dashboard::default();
        d.update_entries(vec![entry(1, 10, 20, Some(1.0), false)], 0);
        let actions = d.handle_key(&key_press(KeyCode::Char('x')));
        assert!(actions.is_empty());
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
        d.update_entries(vec![entry(42, 10, 20, Some(2.0), false)], 0);
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
        d.update_entries(vec![entry(42, 10, 20, Some(2.0), false)], 0);
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
    fn test_enter_on_running_opens_edit() {
        let mut d = Dashboard::default();
        d.update_entries(vec![entry(1, 10, 20, None, true)], 0);
        let actions = d.handle_key(&key_press(KeyCode::Enter));
        assert!(matches!(
            actions[0],
            Action::OpenForm {
                mode: FormMode::Edit,
                entry_id: Some(1),
                ..
            }
        ));
    }

    #[test]
    fn test_enter_on_running_passes_is_running_true() {
        let mut d = Dashboard::default();
        d.update_entries(vec![entry(1, 10, 20, None, true)], 0);
        let actions = d.handle_key(&key_press(KeyCode::Enter));
        assert!(matches!(
            actions[0],
            Action::OpenForm {
                is_running: true,
                ..
            }
        ));
    }

    #[test]
    fn test_e_on_stopped_passes_is_running_false() {
        let mut d = Dashboard::default();
        d.update_entries(vec![entry(1, 10, 20, Some(2.0), false)], 0);
        let actions = d.handle_key(&key_press(KeyCode::Char('e')));
        assert!(matches!(
            actions[0],
            Action::OpenForm {
                is_running: false,
                ..
            }
        ));
    }

    #[test]
    fn test_enter_no_selection_returns_empty() {
        let mut d = Dashboard::default();
        let actions = d.handle_key(&key_press(KeyCode::Enter));
        assert!(actions.is_empty());
    }

    #[test]
    fn test_h_navigates_prev_day() {
        let mut d = Dashboard::default();
        let actions = d.handle_key(&key_press(KeyCode::Char('h')));
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], Action::NavigateDayPrev));
    }

    #[test]
    fn test_l_navigates_next_day() {
        let mut d = Dashboard::default();
        let actions = d.handle_key(&key_press(KeyCode::Char('l')));
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], Action::NavigateDayNext));
    }

    #[test]
    fn test_left_arrow_navigates_prev_day() {
        let mut d = Dashboard::default();
        let actions = d.handle_key(&key_press(KeyCode::Left));
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], Action::NavigateDayPrev));
    }

    #[test]
    fn test_right_arrow_navigates_next_day() {
        let mut d = Dashboard::default();
        let actions = d.handle_key(&key_press(KeyCode::Right));
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], Action::NavigateDayNext));
    }

    #[test]
    fn test_shift_t_goes_to_today() {
        let mut d = Dashboard::default();
        let actions = d.handle_key(&key_press(KeyCode::Char('T')));
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], Action::NavigateDayToday));
    }

    #[test]
    fn test_g_opens_date_picker() {
        let mut d = Dashboard::default();
        let actions = d.handle_key(&key_press(KeyCode::Char('g')));
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], Action::OpenDatePicker));
    }

    #[test]
    fn test_selected_date_defaults_to_today() {
        let d = Dashboard::default();
        assert_eq!(d.selected_date(), harv_core::datetime::today());
    }

    #[test]
    fn test_prev_day_decrements_date() {
        let mut d = Dashboard::default();
        let before = d.selected_date();
        d.prev_day();
        assert_eq!(d.selected_date(), before - chrono::Duration::days(1));
    }

    #[test]
    fn test_next_day_increments_date() {
        let mut d = Dashboard::default();
        let before = d.selected_date();
        d.next_day();
        assert_eq!(d.selected_date(), before + chrono::Duration::days(1));
    }

    #[test]
    fn test_go_today_sets_to_today() {
        let mut d = Dashboard::default();
        d.next_day();
        d.go_today();
        assert_eq!(d.selected_date(), harv_core::datetime::today());
    }

    #[test]
    fn test_set_date_changes_selected_date() {
        let mut d = Dashboard::default();
        let target = chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        d.set_date(target);
        assert_eq!(d.selected_date(), target);
    }
}
