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
                    ("n", "New entry (with hours)"),
                    ("Enter/e", "Log time on stopped entry"),
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
