use crate::theme::Theme;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph};
use ratatui::Frame;

const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn render_spinner(area: Rect, f: &mut Frame, tick: u64, msg: &str, theme: &Theme) {
    let frame = FRAMES[(tick % FRAMES.len() as u64) as usize];
    let centered = crate::popup::centered_rect(30, 15, area);

    let line = Line::from(vec![
        Span::styled(format!("{} ", frame), Style::new().fg(theme.primary)),
        Span::styled(msg, Style::new().fg(theme.muted)),
    ]);

    let paragraph = Paragraph::new(line).alignment(Alignment::Center);

    f.render_widget(Clear, centered);
    f.render_widget(paragraph, centered);
}
