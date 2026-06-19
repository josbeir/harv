pub mod dashboard;
pub mod date_picker;
pub mod form;
pub mod help;

use crate::action::Action;
use crate::theme::Theme;
use ratatui::Frame;
use ratatui::layout::Rect;

use self::dashboard::Dashboard;

#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
pub enum View {
    Dashboard(Dashboard),
}

impl View {
    pub fn render(&mut self, area: Rect, f: &mut Frame, theme: &Theme, tick: u64) {
        match self {
            View::Dashboard(v) => v.render(area, f, theme, tick),
        }
    }

    pub fn handle_key(&mut self, key: &ratatui::crossterm::event::KeyEvent) -> Vec<Action> {
        match self {
            View::Dashboard(v) => v.handle_key(key),
        }
    }

    pub fn timer_running(&self) -> bool {
        match self {
            View::Dashboard(d) => d.has_running(),
        }
    }

    pub fn selected_date(&self) -> chrono::NaiveDate {
        match self {
            View::Dashboard(d) => d.selected_date(),
        }
    }
}

impl Default for View {
    fn default() -> Self {
        View::Dashboard(Dashboard::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_view_default_is_dashboard() {
        let view = View::default();
        assert!(matches!(view, View::Dashboard(_)));
    }

    #[test]
    fn test_view_dashboard_render_does_not_panic() {
        let theme = Theme::default();
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut view = View::Dashboard(Dashboard::default());

        terminal
            .draw(|f| {
                view.render(f.area(), f, &theme, 0);
            })
            .unwrap();
    }

    #[test]
    fn test_view_dashboard_handle_key() {
        let mut view = View::Dashboard(Dashboard::default());
        let key = ratatui::crossterm::event::KeyEvent::new(
            ratatui::crossterm::event::KeyCode::Char('r'),
            ratatui::crossterm::event::KeyModifiers::NONE,
        );
        let actions = view.handle_key(&key);
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], Action::Refresh));
    }

    #[test]
    fn test_view_timer_running() {
        let d = Dashboard::default();
        assert!(!d.has_running());
        let view = View::Dashboard(d);
        assert!(!view.timer_running());
    }

    #[test]
    fn test_view_selected_date() {
        let d = Dashboard::default();
        let view = View::Dashboard(d);
        assert_eq!(view.selected_date(), harv_core::datetime::today());
    }
}
