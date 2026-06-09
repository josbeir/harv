use crate::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph};
use ratatui::Frame;

const ASCII: &[&str] = &[
    "▗▖ ▗▖ ▗▄▖ ▗▄▄▖ ▗▖  ▗▖",
    "▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌▐▌  ▐▌",
    "▐▛▀▜▌▐▛▀▜▌▐▛▀▚▖▐▌  ▐▌",
    "▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌ ▝▚▞▘ ",
];

const SHADES: &[(u8, u8, u8)] = &[
    (250, 210, 140),
    (250, 170, 90),
    (250, 130, 40),
    (250, 93, 0),
];

pub fn render_harv_loading(area: Rect, f: &mut Frame, tick: u64, msg: &str, theme: &Theme) {
    f.render_widget(Clear, area);

    let offset = (tick % 4) as usize;
    let version = env!("CARGO_PKG_VERSION");
    let version_text = format!("HARV CLI v{}", version);
    let pad = (21usize.saturating_sub(version_text.len())) / 2;

    let mut lines: Vec<Line> = Vec::new();

    for i in 0..4 {
        let (r, g, b) = SHADES[(offset + i) % 4];
        lines.push(Line::from(Span::styled(
            ASCII[i].to_string(),
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
        Span::styled(
            format!(" v{}", version),
            Style::new().fg(Color::Rgb(160, 160, 160)),
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(msg, Style::new().fg(theme.muted))));

    let content_height = lines.len() as u16;

    let v_margin = area.height.saturating_sub(content_height) / 2;
    let vertical = Layout::vertical([
        Constraint::Length(v_margin),
        Constraint::Length(content_height),
        Constraint::Min(v_margin),
    ])
    .split(area);

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
