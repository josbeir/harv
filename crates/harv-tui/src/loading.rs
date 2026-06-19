use crate::theme::Theme;
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph};

pub const ASCII_LOGO: &[&str] = &[
    "▗▖ ▗▖ ▗▄▖ ▗▄▄▖ ▗▖  ▗▖",
    "▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌▐▌  ▐▌",
    "▐▛▀▜▌▐▛▀▜▌▐▛▀▚▖▐▌  ▐▌",
    "▐▌ ▐▌▐▌ ▐▌▐▌ ▐▌ ▝▚▞▘ ",
];

pub const LOGO_SHADES: &[(u8, u8, u8)] = &[
    (250, 210, 140),
    (250, 170, 90),
    (250, 130, 40),
    (250, 93, 0),
];

pub fn render_harv_loading(area: Rect, f: &mut Frame, tick: u64, msg: &str, theme: &Theme) {
    f.render_widget(Clear, area);

    let offset = (tick % 4) as usize;
    let version = env!("CARGO_PKG_VERSION");
    let version_visible = format!("{} v{}", harv_core::t("tui-app-title"), version);
    let inner_pad = (21usize.saturating_sub(version_visible.len())) / 2;

    let mut lines: Vec<Line> = Vec::new();

    for (i, line) in ASCII_LOGO.iter().enumerate() {
        let spans: Vec<Span> = line
            .chars()
            .enumerate()
            .map(|(j, ch)| {
                let shade_idx = (offset + i + (j / 3)) % 4;
                let (r, g, b) = LOGO_SHADES[shade_idx];
                Span::styled(
                    ch.to_string(),
                    Style::new()
                        .fg(Color::Rgb(r, g, b))
                        .add_modifier(Modifier::BOLD),
                )
            })
            .collect();
        lines.push(Line::from(spans));
    }

    let version_color = match theme.mode {
        crate::theme::ThemeMode::Dark => Color::Rgb(160, 160, 160),
        crate::theme::ThemeMode::Light => Color::Rgb(90, 90, 90),
    };

    lines.push(Line::from(vec![
        Span::styled(" ".repeat(inner_pad), Style::default()),
        Span::styled(
            harv_core::t("tui-app-title"),
            Style::new()
                .fg(Color::Rgb(250, 93, 0))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" v{}", version), Style::new().fg(version_color)),
        Span::styled(
            " ".repeat(21usize.saturating_sub(version_visible.len() + inner_pad)),
            Style::default(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_render_harv_loading_does_not_panic() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::default();

        terminal
            .draw(|f| {
                render_harv_loading(f.area(), f, 0, "Loading...", &theme);
            })
            .unwrap();
    }

    #[test]
    fn test_render_harv_loading_with_tick() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::default();

        // Should not panic with different tick values
        for tick in 0..10 {
            terminal
                .draw(|f| {
                    render_harv_loading(f.area(), f, tick, "Loading...", &theme);
                })
                .unwrap();
        }
    }

    #[test]
    fn test_render_harv_loading_small_terminal() {
        let backend = TestBackend::new(30, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::default();

        terminal
            .draw(|f| {
                render_harv_loading(f.area(), f, 0, "Loading...", &theme);
            })
            .unwrap();
    }
}
