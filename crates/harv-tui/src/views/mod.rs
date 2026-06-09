pub mod dashboard;
pub mod form;
pub mod help;

use crate::action::{Action, ViewId};
use crate::theme::Theme;
use ratatui::layout::Rect;
use ratatui::Frame;

use self::dashboard::Dashboard;
use self::help::Help;

#[allow(clippy::large_enum_variant)]
pub enum View {
    Dashboard(Dashboard),
    #[allow(dead_code)]
    Help(Help),
}

impl View {
    #[allow(dead_code)]
    pub fn id(&self) -> ViewId {
        match self {
            View::Dashboard(_) => ViewId::Dashboard,
            View::Help(_) => todo!(),
        }
    }

    pub fn render(&mut self, area: Rect, f: &mut Frame, theme: &Theme, tick: u64) {
        match self {
            View::Dashboard(v) => v.render(area, f, theme, tick),
            View::Help(v) => v.render(area, f, theme),
        }
    }

    pub fn handle_key(&mut self, key: &ratatui::crossterm::event::KeyEvent) -> Vec<Action> {
        match self {
            View::Dashboard(v) => v.handle_key(key),
            View::Help(v) => v.handle_key(key),
        }
    }

    pub fn timer_running(&self) -> bool {
        match self {
            View::Dashboard(d) => d.has_running(),
            View::Help(_) => false,
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
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_view_default_is_dashboard() {
        let view = View::default();
        assert!(matches!(view, View::Dashboard(_)));
    }

    #[test]
    fn test_view_dashboard_id() {
        let view = View::Dashboard(Dashboard::default());
        assert_eq!(view.id(), ViewId::Dashboard);
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
}
