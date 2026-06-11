use chrono::{Datelike, Duration, NaiveDate};
use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Clear, Padding, Row, Table};

use crate::action::Action;
use crate::theme::Theme;

pub struct DatePicker {
    current_month: NaiveDate,
    cursor: NaiveDate,
    visible: bool,
}

impl DatePicker {
    pub fn new(initial: NaiveDate) -> Self {
        let current_month = NaiveDate::from_ymd_opt(initial.year(), initial.month(), 1)
            .expect("valid initial date");
        Self {
            current_month,
            cursor: initial,
            visible: true,
        }
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn dismiss(&mut self) {
        self.visible = false;
    }

    pub fn render(&mut self, area: Rect, f: &mut Frame, theme: &Theme) {
        if !self.visible {
            return;
        }

        let popup_area = crate::popup::centered_rect_fixed(31, 10, area);
        f.render_widget(Clear, popup_area);

        let today = harv_core::datetime::today();

        let month_title = self.current_month.format("%B %Y").to_string();
        let month_start = self.current_month;
        let first_weekday = month_start.weekday().num_days_from_sunday() as i64;
        let grid_start = month_start - Duration::days(first_weekday);

        // Header row — abbreviated day names
        let day_names = [
            harv_core::t("tui-datepicker-sun"),
            harv_core::t("tui-datepicker-mon"),
            harv_core::t("tui-datepicker-tue"),
            harv_core::t("tui-datepicker-wed"),
            harv_core::t("tui-datepicker-thu"),
            harv_core::t("tui-datepicker-fri"),
            harv_core::t("tui-datepicker-sat"),
        ];
        let header_cells: Vec<Cell> = day_names
            .iter()
            .map(|name| {
                Cell::from(name.as_str())
                    .style(Style::new().fg(theme.muted).add_modifier(Modifier::BOLD))
            })
            .collect();

        let header = Row::new(header_cells).height(1);

        // Day rows — 6 weeks
        let mut day_rows: Vec<Row> = Vec::new();

        for week in 0..6 {
            let mut cells: Vec<Cell> = Vec::new();
            for day in 0..7 {
                let date = grid_start + Duration::days(week * 7 + day);
                let in_month =
                    date.month() == month_start.month() && date.year() == month_start.year();
                let is_today = date == today;
                let is_cursor = date == self.cursor;

                let text = if in_month {
                    format!("{:2}", date.day())
                } else {
                    String::from("  ")
                };

                let style = if is_cursor {
                    Style::new()
                        .fg(theme.bg)
                        .bg(theme.highlight)
                        .add_modifier(Modifier::BOLD)
                } else if is_today {
                    Style::new()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                } else if !in_month {
                    Style::new().fg(theme.muted)
                } else {
                    Style::new().fg(theme.fg)
                };

                cells.push(Cell::from(text).style(style));
            }
            day_rows.push(Row::new(cells));
        }

        let block = Block::new()
            .title(format!(" {} ", month_title))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::new().fg(theme.primary))
            .padding(Padding::new(1, 1, 0, 0))
            .style(Style::new().bg(theme.surface));

        let table = Table::new(day_rows, [Constraint::Length(3); 7])
            .header(header)
            .block(block)
            .column_spacing(1);

        f.render_widget(table, popup_area);
    }

    pub fn handle_key(&mut self, key: &ratatui::crossterm::event::KeyEvent) -> Vec<Action> {
        match key.code {
            KeyCode::Esc => {
                self.visible = false;
                vec![Action::CloseDatePicker]
            }
            KeyCode::Enter => {
                self.visible = false;
                vec![Action::SelectDate(self.cursor)]
            }
            KeyCode::Left => {
                self.cursor -= Duration::days(1);
                self.sync_month();
                vec![]
            }
            KeyCode::Right => {
                self.cursor += Duration::days(1);
                self.sync_month();
                vec![]
            }
            KeyCode::Up => {
                self.cursor -= Duration::days(7);
                self.sync_month();
                vec![]
            }
            KeyCode::Down => {
                self.cursor += Duration::days(7);
                self.sync_month();
                vec![]
            }
            KeyCode::Char(',') | KeyCode::Char('<') if key.modifiers == KeyModifiers::NONE => {
                self.current_month = previous_month(self.current_month);
                self.cursor = self.current_month;
                vec![]
            }
            KeyCode::Char('.') | KeyCode::Char('>') if key.modifiers == KeyModifiers::NONE => {
                self.current_month = next_month(self.current_month);
                self.cursor = self.current_month;
                vec![]
            }
            _ => vec![],
        }
    }

    fn sync_month(&mut self) {
        self.current_month = NaiveDate::from_ymd_opt(self.cursor.year(), self.cursor.month(), 1)
            .unwrap_or(self.current_month);
    }
}

fn previous_month(date: NaiveDate) -> NaiveDate {
    if date.month() == 1 {
        NaiveDate::from_ymd_opt(date.year() - 1, 12, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() - 1, 1).unwrap()
    }
}

fn next_month(date: NaiveDate) -> NaiveDate {
    if date.month() == 12 {
        NaiveDate::from_ymd_opt(date.year() + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(date.year(), date.month() + 1, 1).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stores_initial_date() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let dp = DatePicker::new(date);
        assert_eq!(dp.cursor, date);
        assert!(dp.visible);
    }

    #[test]
    fn test_visible_and_dismiss() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let mut dp = DatePicker::new(date);
        assert!(dp.visible());
        dp.dismiss();
        assert!(!dp.visible());
    }

    #[test]
    fn test_enter_selects_date() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let mut dp = DatePicker::new(date);
        let key = ratatui::crossterm::event::KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let actions = dp.handle_key(&key);
        assert!(!dp.visible);
        assert!(matches!(actions[0], Action::SelectDate(ref d) if *d == date));
    }

    #[test]
    fn test_esc_dismisses() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let mut dp = DatePicker::new(date);
        let key = ratatui::crossterm::event::KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        let actions = dp.handle_key(&key);
        assert!(!dp.visible);
        assert!(matches!(actions[0], Action::CloseDatePicker));
    }

    #[test]
    fn test_right_arrow_moves_cursor() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let mut dp = DatePicker::new(date);
        let key = ratatui::crossterm::event::KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        let _ = dp.handle_key(&key);
        assert_eq!(dp.cursor, NaiveDate::from_ymd_opt(2026, 6, 11).unwrap());
    }

    #[test]
    fn test_left_arrow_moves_cursor() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let mut dp = DatePicker::new(date);
        let key = ratatui::crossterm::event::KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        let _ = dp.handle_key(&key);
        assert_eq!(dp.cursor, NaiveDate::from_ymd_opt(2026, 6, 9).unwrap());
    }

    #[test]
    fn test_up_arrow_moves_week() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let mut dp = DatePicker::new(date);
        let key = ratatui::crossterm::event::KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let _ = dp.handle_key(&key);
        assert_eq!(dp.cursor, NaiveDate::from_ymd_opt(2026, 6, 3).unwrap());
    }

    #[test]
    fn test_down_arrow_moves_week() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let mut dp = DatePicker::new(date);
        let key = ratatui::crossterm::event::KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let _ = dp.handle_key(&key);
        assert_eq!(dp.cursor, NaiveDate::from_ymd_opt(2026, 6, 17).unwrap());
    }

    #[test]
    fn test_cursor_auto_advances_month() {
        // June 1, 2026 — pressing left should go to May 31
        let date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let mut dp = DatePicker::new(date);
        let key = ratatui::crossterm::event::KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        let _ = dp.handle_key(&key);
        assert_eq!(dp.cursor, NaiveDate::from_ymd_opt(2026, 5, 31).unwrap());
        assert_eq!(dp.current_month.month(), 5);
    }

    #[test]
    fn test_angle_brackets_navigate_months() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let mut dp = DatePicker::new(date);

        let key = ratatui::crossterm::event::KeyEvent::new(KeyCode::Char('<'), KeyModifiers::NONE);
        let _ = dp.handle_key(&key);
        assert_eq!(dp.current_month.month(), 5);

        let key = ratatui::crossterm::event::KeyEvent::new(KeyCode::Char('>'), KeyModifiers::NONE);
        let _ = dp.handle_key(&key);
        assert_eq!(dp.current_month.month(), 6);

        let key = ratatui::crossterm::event::KeyEvent::new(KeyCode::Char('>'), KeyModifiers::NONE);
        let _ = dp.handle_key(&key);
        assert_eq!(dp.current_month.month(), 7);
    }

    #[test]
    fn test_render_does_not_panic() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let mut dp = DatePicker::new(date);
        let theme = Theme::default();
        let backend = ratatui::backend::TestBackend::new(80, 30);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                dp.render(f.area(), f, &theme);
            })
            .unwrap();
    }

    #[test]
    fn test_unknown_key_returns_empty() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let mut dp = DatePicker::new(date);
        let key = ratatui::crossterm::event::KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        let actions = dp.handle_key(&key);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_month_nav_across_year_boundary() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
        let mut dp = DatePicker::new(date);
        let key = ratatui::crossterm::event::KeyEvent::new(KeyCode::Char('<'), KeyModifiers::NONE);
        let _ = dp.handle_key(&key);
        assert_eq!(dp.current_month.month(), 12);
        assert_eq!(dp.current_month.year(), 2025);

        let key = ratatui::crossterm::event::KeyEvent::new(KeyCode::Char('>'), KeyModifiers::NONE);
        let _ = dp.handle_key(&key);
        assert_eq!(dp.current_month.month(), 1);
        assert_eq!(dp.current_month.year(), 2026);
    }

    #[test]
    fn test_cursor_auto_advances_december() {
        let date = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        let mut dp = DatePicker::new(date);
        let key = ratatui::crossterm::event::KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        let _ = dp.handle_key(&key);
        assert_eq!(dp.cursor, NaiveDate::from_ymd_opt(2027, 1, 1).unwrap());
        assert_eq!(dp.current_month.month(), 1);
        assert_eq!(dp.current_month.year(), 2027);
    }
}
