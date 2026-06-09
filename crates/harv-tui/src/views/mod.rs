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
}

impl Default for View {
    fn default() -> Self {
        View::Dashboard(Dashboard::default())
    }
}
