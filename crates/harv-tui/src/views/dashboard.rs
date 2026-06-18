use crate::action::{Action, FormMode};
use crate::theme::Theme;
use chrono::NaiveDate;
use harv_core::TimeEntry;
use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph};

const HOURS_COL_WIDTH: usize = 5;

fn entry_to_item_index(entry_idx: usize) -> usize {
    entry_idx * 2
}

fn item_to_entry_index(item_idx: usize) -> usize {
    item_idx / 2
}

pub struct Dashboard {
    entries: Vec<TimeEntry>,
    running_entry: Option<TimeEntry>,
    daily_total: f64,
    list_state: ListState,
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
            list_state: ListState::default().with_selected(Some(0)),
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
        self.list_state.select(Some(0));
    }

    pub fn update_running(&mut self, entries: Vec<TimeEntry>) {
        self.running_entry = entries.into_iter().next();
    }

    pub fn selected_entry(&self) -> Option<&TimeEntry> {
        self.list_state.selected().and_then(|i| {
            let entry_idx = item_to_entry_index(i);
            self.entries.get(entry_idx)
        })
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
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

        self.render_date_nav(layout[1], f, theme);

        if !self.loaded {
            crate::loading::render_harv_loading(layout[3], f, tick, &self.loading_msg, theme);
            return;
        }

        if self.entries.is_empty() {
            render_harv_header(layout[3], f, theme, self.selected_date);
            self.render_stats_footer(layout[4], f, theme);
            return;
        }

        let header_rows = if self.running_entry.is_some() { 2 } else { 0 };
        let body = Layout::vertical([Constraint::Length(header_rows), Constraint::Min(0)])
            .split(layout[3]);

        if self.running_entry.is_some() {
            self.render_timer_header(body[0], f, theme);
        }
        self.render_entry_list(body[1], f, theme);
        self.render_stats_footer(layout[4], f, theme);
    }

    fn render_date_nav(&self, area: Rect, f: &mut Frame, theme: &Theme) {
        let is_today = self.selected_date == harv_core::datetime::today();
        let date_formatted = harv_core::datetime::format_date_header(
            self.selected_date,
            &harv_core::current_langid(),
        );

        let mut spans = vec![
            Span::styled(" < ", Style::new().fg(theme.muted)),
            Span::styled(
                date_formatted,
                Style::new().fg(theme.fg).add_modifier(Modifier::BOLD),
            ),
        ];
        if is_today {
            spans.push(Span::styled(
                format!(" {} ", harv_core::t("tui-dash-today")),
                Style::new().fg(theme.muted),
            ));
        }
        spans.push(Span::styled(" > ", Style::new().fg(theme.muted)));

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line).alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }

    fn render_timer_header(&self, area: Rect, f: &mut Frame, theme: &Theme) {
        let (indicator, status_text, elapsed) = if let Some(ref entry) = self.running_entry {
            let elapsed_str = format_timer_elapsed(entry);
            let status = harv_core::t_args("tui-dash-running-header", &[("elapsed", elapsed_str)]);
            (harv_core::t("tui-dash-running-prefix"), status, true)
        } else {
            ("○".to_string(), harv_core::t("tui-dash-idle-header"), false)
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
                    format!(
                        " {} · {} ",
                        harv_core::text::format_project_display(
                            &entry.project.name,
                            entry.project_code.as_deref()
                        ),
                        entry.task.name
                    ),
                    Style::new().fg(theme.fg),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled(format!(" {} ", indicator), Style::new().fg(color)),
                Span::styled(format!(" {} ", status_text), Style::new().fg(color)),
                Span::styled(
                    format!(" {} ", harv_core::t("tui-dash-no-timer")),
                    Style::new().fg(theme.muted),
                ),
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
        let hmargin = 2u16;
        let padded_area = Rect {
            x: area.x + hmargin,
            y: area.y,
            width: area.width.saturating_sub(hmargin * 2),
            height: area.height,
        };

        let block_title = if self.selected_date == harv_core::datetime::today() {
            harv_core::t("tui-dash-block-today")
        } else {
            harv_core::datetime::format_date_short(self.selected_date, &harv_core::current_langid())
        };
        let block = Block::new()
            .title(block_title)
            .borders(Borders::ALL)
            .border_style(Style::new().fg(theme.border))
            .style(Style::new().bg(theme.bg))
            .padding(Padding::horizontal(1));

        let inner = block.inner(padded_area);
        let content_width = inner.width;

        let border_style = Style::new().fg(theme.border);
        let muted_style = Style::new().fg(theme.muted);
        let sep_prefix = "─".repeat(HOURS_COL_WIDTH);
        let padding = " ".repeat(HOURS_COL_WIDTH);

        let mut items: Vec<ListItem> = Vec::new();
        let mut heights: Vec<u16> = Vec::new();

        for (i, entry) in self.entries.iter().enumerate() {
            let running = entry.is_running;
            let hours_display = if running {
                format!("{:>4} ", harv_core::t("tui-dash-running-prefix"))
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
                Span::styled("│ ", border_style),
                Span::styled(display_name, Style::new().fg(theme.fg)),
            ];
            if let Some(ref client) = entry.client {
                line1_spans.push(Span::styled(format!(" | {}", client.name), muted_style));
            }

            let line2_spans = vec![
                Span::raw(&padding),
                Span::styled("│ ", border_style),
                Span::styled(format!(" └─ {}", entry.task.name), muted_style),
            ];

            let mut entry_lines = vec![Line::from(line1_spans), Line::from(line2_spans)];

            let has_notes = entry.notes.as_deref().is_some_and(|n| !n.is_empty());
            if has_notes {
                let line3_spans = vec![
                    Span::raw(&padding),
                    Span::styled("│ ", border_style),
                    Span::styled(
                        format!(" └─ {}", entry.notes.as_deref().unwrap()),
                        muted_style,
                    ),
                ];
                entry_lines.push(Line::from(line3_spans));
            }

            items.push(ListItem::new(Text::from(entry_lines)));
            heights.push(if has_notes { 3 } else { 2 });

            if i < self.entries.len() - 1 {
                let sep_width = content_width.saturating_sub(HOURS_COL_WIDTH as u16 + 1);
                let sep_line = format!("{}┼{}", sep_prefix, "─".repeat(sep_width as usize),);
                items.push(ListItem::new(Text::from(Line::from(Span::styled(
                    sep_line,
                    border_style,
                )))));
                heights.push(1);
            }
        }

        self.ensure_selected_visible(&heights, inner.height);

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::new()
                    .fg(theme.highlight)
                    .bg(theme.surface)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("");

        f.render_stateful_widget(list, padded_area, &mut self.list_state);
    }

    fn ensure_selected_visible(&mut self, heights: &[u16], viewport_h: u16) {
        let Some(selected) = self.list_state.selected() else {
            return;
        };
        if heights.is_empty() {
            return;
        }
        let mut cum = vec![0u16];
        for &h in heights {
            cum.push(cum.last().unwrap() + h);
        }
        let sel_start = cum[selected.min(heights.len())];
        let sel_h = heights[selected.min(heights.len())];
        let offset = self.list_state.offset();
        let view_start = cum[offset.min(heights.len())];

        if sel_start < view_start {
            *self.list_state.offset_mut() = selected;
        } else if sel_start + sel_h > view_start + viewport_h {
            let target = (sel_start + sel_h).saturating_sub(viewport_h);
            let new_off = cum.iter().rposition(|&p| p <= target).unwrap_or(0);
            *self.list_state.offset_mut() = new_off;
        }
    }

    fn render_stats_footer(&self, area: Rect, f: &mut Frame, theme: &Theme) {
        let hmargin = 2u16;
        let padded = Rect {
            x: area.x + hmargin,
            y: area.y,
            width: area.width.saturating_sub(hmargin * 2),
            height: area.height,
        };

        let block = Block::new()
            .borders(Borders::ALL)
            .border_style(Style::new().fg(theme.border))
            .style(Style::new().bg(theme.bg));
        let inner = block.inner(padded);

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

        let paragraph = Paragraph::new(line)
            .alignment(Alignment::Center)
            .style(Style::new().bg(theme.bg));

        f.render_widget(block, padded);
        if inner.height > 0 {
            f.render_widget(paragraph, inner);
        }
    }

    pub fn handle_key(&mut self, key: &ratatui::crossterm::event::KeyEvent) -> Vec<Action> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let max = entry_to_item_index(self.entries.len().saturating_sub(1));
                let i = self.list_state.selected().map_or(0, |i| (i + 2).min(max));
                self.list_state.select(Some(i));
                vec![]
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let i = self
                    .list_state
                    .selected()
                    .map_or(0, |i| i.saturating_sub(2));
                self.list_state.select(Some(i));
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

fn format_timer_elapsed(entry: &TimeEntry) -> String {
    if let Some(started) = entry.timer_started_at {
        let elapsed = chrono::Utc::now() - started;
        harv_core::text::format_elapsed_hms(elapsed.num_seconds())
    } else {
        String::from("--:--:--")
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
        d.list_state.select(Some(0));
        assert_eq!(d.selected_entry().unwrap().id, 1);
        d.list_state.select(Some(2));
        assert_eq!(d.selected_entry().unwrap().id, 2);
        d.list_state.select(None);
        assert!(d.selected_entry().is_none());
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
        assert_eq!(d.list_state.selected(), Some(2));
        d.handle_key(&key_press(KeyCode::Char('j')));
        assert_eq!(d.list_state.selected(), Some(4));
        d.handle_key(&key_press(KeyCode::Char('j')));
        assert_eq!(d.list_state.selected(), Some(4));
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
        d.list_state.select(Some(2));
        d.handle_key(&key_press(KeyCode::Char('k')));
        assert_eq!(d.list_state.selected(), Some(0));
        d.handle_key(&key_press(KeyCode::Char('k')));
        assert_eq!(d.list_state.selected(), Some(0));
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
        d.update_entries(vec![entry(42, 10, 20, Some(2.0), false)], 0);
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
        d.list_state.select(Some(0));
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
