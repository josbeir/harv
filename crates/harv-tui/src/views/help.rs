use crate::action::Action;
use crate::theme::Theme;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

#[derive(Default)]
pub struct Help {
    visible: bool,
}

impl Help {
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn render(&mut self, area: Rect, f: &mut Frame, theme: &Theme) {
        if !self.visible {
            return;
        }

        let popup_area = crate::popup::centered_rect(60, 80, area);

        // Clear behind the popup
        f.render_widget(Clear, popup_area);

        let shortcuts = vec![
            (
                "Navigation",
                vec![
                    ("j / ↓", "Move down"),
                    ("k / ↑", "Move up"),
                    ("Enter", "Select / confirm"),
                    ("Tab", "Next field"),
                    ("Esc", "Cancel / back"),
                ],
            ),
            (
                "Actions",
                vec![
                    ("s", "Start timer"),
                    ("n / t", "New entry (with hours)"),
                    ("Enter / e", "Edit entry"),
                    ("d", "Delete entry"),
                    ("r", "Refresh"),
                ],
            ),
            ("General", vec![("?", "Toggle help"), ("q", "Quit")]),
        ];

        let mut lines: Vec<Line> = Vec::new();
        for (section, keys) in &shortcuts {
            lines.push(Line::from(Span::styled(
                format!(" {} ", section),
                Style::new().fg(theme.primary).add_modifier(Modifier::BOLD),
            )));
            for (key, desc) in keys {
                lines.push(Line::from(vec![
                    Span::styled(format!("  {:12}", key), Style::new().fg(theme.highlight)),
                    Span::styled(desc.to_string(), Style::new().fg(theme.muted)),
                ]));
            }
            lines.push(Line::from(""));
        }

        let block = Block::new()
            .title(" Help ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::new().fg(theme.primary))
            .style(Style::new().bg(theme.surface));

        let paragraph = Paragraph::new(lines).block(block);
        f.render_widget(paragraph, popup_area);
    }

    pub fn handle_key(&mut self, key: &ratatui::crossterm::event::KeyEvent) -> Vec<Action> {
        match key.code {
            KeyCode::Char('?') | KeyCode::Esc => {
                self.toggle();
                vec![]
            }
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_help_renders_when_visible() {
        let mut help = Help::default();
        help.toggle();
        assert!(help.is_visible());

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::default();

        terminal
            .draw(|f| {
                help.render(f.area(), f, &theme);
            })
            .unwrap();
    }

    #[test]
    fn test_help_not_renders_when_hidden() {
        let mut help = Help::default();
        assert!(!help.is_visible());

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::default();

        terminal
            .draw(|f| {
                help.render(f.area(), f, &theme);
            })
            .unwrap();
    }

    #[test]
    fn test_help_toggle() {
        let mut help = Help::default();
        help.toggle();
        assert!(help.is_visible());
        help.toggle();
        assert!(!help.is_visible());
    }

    #[test]
    fn test_help_handle_key_toggle() {
        let mut help = Help::default();
        let actions = help.handle_key(&key_press(KeyCode::Char('?')));
        assert!(help.is_visible());
        assert!(actions.is_empty());

        let actions = help.handle_key(&key_press(KeyCode::Esc));
        assert!(!help.is_visible());
        assert!(actions.is_empty());
    }

    #[test]
    fn test_help_handle_key_other_returns_empty() {
        let mut help = Help::default();
        let actions = help.handle_key(&key_press(KeyCode::Char('x')));
        assert!(actions.is_empty());
    }

    fn key_press(code: KeyCode) -> ratatui::crossterm::event::KeyEvent {
        ratatui::crossterm::event::KeyEvent::new(
            code,
            ratatui::crossterm::event::KeyModifiers::NONE,
        )
    }
}
