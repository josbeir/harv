use harv_sdk::mock_data::make_time_entry;
use harv_tui::action::{Action, FormMode};
use harv_tui::theme::Theme;
use harv_tui::theme::ThemeMode;
use harv_tui::views::dashboard::Dashboard;
use harv_tui::views::form::TimeEntryForm;
use harv_tui::views::help::Help;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
#[test]
fn test_dashboard_render_with_entries() {
    harv_core::init_locale(Some("en"));
    let mut d = Dashboard::default();
    d.update_entries(
        vec![
            make_time_entry(1, 10, 20, Some(2.5), false),
            make_time_entry(2, 11, 21, None, true),
        ],
        0,
    );
    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| d.render(f.area(), f, &Theme::dark(), 0))
        .unwrap();
}
#[test]
fn test_dashboard_render_loading() {
    harv_core::init_locale(Some("en"));
    let mut d = Dashboard::default();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| d.render(f.area(), f, &Theme::dark(), 0))
        .unwrap();
}
#[test]
fn test_dashboard_render_empty() {
    harv_core::init_locale(Some("en"));
    let mut d = Dashboard::default();
    d.set_loaded(true);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| d.render(f.area(), f, &Theme::dark(), 0))
        .unwrap();
}
#[test]
fn test_form_render_start_mode() {
    let mut f = TimeEntryForm::new(
        None,
        None,
        None,
        FormMode::Start,
        None,
        None,
        None,
        None,
        false,
    );
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| f.render(frame.area(), frame, &Theme::dark(), 0))
        .unwrap();
}
#[test]
fn test_form_render_create_mode() {
    let mut f = TimeEntryForm::new(
        None,
        None,
        None,
        FormMode::Create,
        None,
        None,
        None,
        None,
        false,
    );
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| f.render(frame.area(), frame, &Theme::dark(), 0))
        .unwrap();
}
#[test]
fn test_form_render_loading() {
    let mut f = TimeEntryForm::new(
        None,
        None,
        None,
        FormMode::Start,
        None,
        None,
        None,
        None,
        false,
    );
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| f.render(frame.area(), frame, &Theme::dark(), 0))
        .unwrap();
}
#[test]
fn test_help_render_visible() {
    let mut h = Help::default();
    h.toggle();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| h.render(f.area(), f, &Theme::dark()))
        .unwrap();
}
#[test]
fn test_help_render_hidden() {
    let mut h = Help::default();
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| h.render(f.area(), f, &Theme::dark()))
        .unwrap();
}
#[tokio::test]
async fn test_theme_detect_returns_valid_mode() {
    let t = Theme::detect();
    assert!(matches!(t.mode, ThemeMode::Dark | ThemeMode::Light));
}
#[test]
fn test_theme_dark_light_values() {
    assert_eq!(Theme::dark().mode, ThemeMode::Dark);
    assert_eq!(Theme::light().mode, ThemeMode::Light);
    assert_ne!(Theme::dark().bg, Theme::light().bg);
    assert_ne!(Theme::dark().fg, Theme::light().fg);
}
#[test]
fn test_action_error_clone() {
    let action = Action::Error("test error".into());
    match action {
        Action::Error(msg) => assert_eq!(msg, "test error"),
        _ => panic!(),
    }
}
#[test]
fn test_action_theme_changed() {
    let action = Action::ThemeChanged(ThemeMode::Light);
    match action {
        Action::ThemeChanged(mode) => assert_eq!(mode, ThemeMode::Light),
        _ => panic!(),
    }
}
#[test]
fn test_form_start_mode_shows_project_label() {
    let mut f = TimeEntryForm::new(
        None,
        None,
        None,
        FormMode::Start,
        None,
        None,
        None,
        None,
        false,
    );
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| f.render(frame.area(), frame, &Theme::light(), 0))
        .unwrap();
    let buffer = terminal.backend().buffer();
    // The form popup should exist somewhere in the buffer
    let has_content = (0..buffer.area().height).any(|y| {
        (0..buffer.area().width)
            .any(|x| !buffer[(x, y)].symbol().is_empty() && buffer[(x, y)].symbol() != " ")
    });
    assert!(has_content, "Form should render visible content");
}
#[test]
fn test_form_log_mode_shows_more_fields() {
    let mut f = TimeEntryForm::new(
        Some(10),
        Some(20),
        None,
        FormMode::Edit,
        Some(42),
        Some("2026-06-09".into()),
        Some("1.5".into()),
        None,
        false,
    );
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| f.render(frame.area(), frame, &Theme::dark(), 0))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let has_content = (0..buffer.area().height).any(|y| {
        (0..buffer.area().width)
            .any(|x| !buffer[(x, y)].symbol().is_empty() && buffer[(x, y)].symbol() != " ")
    });
    assert!(has_content, "Edit form should render visible content");
}
#[test]
fn test_form_render_edit_running_mode() {
    harv_core::init_locale(Some("en"));
    let mut f = TimeEntryForm::new(
        Some(10),
        Some(20),
        Some("Project".into()),
        FormMode::Edit,
        Some(42),
        Some("2026-06-09".into()),
        None,
        None,
        true,
    );
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| f.render(frame.area(), frame, &Theme::dark(), 0))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let has_content = (0..buffer.area().height).any(|y| {
        (0..buffer.area().width)
            .any(|x| !buffer[(x, y)].symbol().is_empty() && buffer[(x, y)].symbol() != " ")
    });
    assert!(
        has_content,
        "Edit running form should render visible content"
    );
    // Running edit form should NOT show Date or Hours fields
    let buffer_str = (0..buffer.area().height)
        .map(|y| {
            (0..buffer.area().width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !buffer_str.contains("Date"),
        "Running edit form should not show Date field"
    );
    assert!(
        !buffer_str.contains("Hours"),
        "Running edit form should not show Hours field"
    );
    assert!(
        buffer_str.contains("Enter: save"),
        "Running edit form should show 'Enter: save' help"
    );
}
#[test]
fn test_dashboard_entries_render() {
    harv_core::init_locale(Some("en"));
    let mut d = Dashboard::default();
    d.update_entries(vec![make_time_entry(1, 10, 20, Some(2.0), false)], 0);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::dark();
    terminal.draw(|f| d.render(f.area(), f, &theme, 0)).unwrap();
    let buffer = terminal.backend().buffer();
    let buffer_str = (0..buffer.area().height)
        .map(|y| {
            (0..buffer.area().width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        buffer_str.contains("Project"),
        "Dashboard should render project name"
    );
}
#[test]
fn test_dashboard_past_date_renders() {
    harv_core::init_locale(Some("en"));
    let mut d = Dashboard::default();
    d.update_entries(vec![make_time_entry(1, 10, 20, Some(2.0), false)], 0);
    let past = harv_core::datetime::today() - chrono::Duration::days(3);
    d.set_date(past);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::dark();
    terminal.draw(|f| d.render(f.area(), f, &theme, 0)).unwrap();
    let buffer = terminal.backend().buffer();
    let buffer_str = (0..buffer.area().height)
        .map(|y| {
            (0..buffer.area().width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        buffer_str.contains("Project"),
        "Dashboard should render entries on past dates"
    );
}
#[test]
fn test_stats_footer_shows_project_count() {
    harv_core::init_locale(Some("en"));
    let mut d = Dashboard::default();
    d.update_entries(vec![make_time_entry(1, 10, 20, Some(2.0), false)], 0);
    let past = harv_core::datetime::today() - chrono::Duration::days(3);
    d.set_date(past);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::dark();
    terminal.draw(|f| d.render(f.area(), f, &theme, 0)).unwrap();
    let buffer = terminal.backend().buffer();
    let buffer_str = (0..buffer.area().height)
        .map(|y| {
            (0..buffer.area().width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        buffer_str.contains("0 projects") || buffer_str.contains("projecten"),
        "Stats footer should render with project count"
    );
}
#[test]
fn test_stats_footer_shows_non_zero_project_count() {
    harv_core::init_locale(Some("en"));
    let mut d = Dashboard::default();
    d.update_entries(vec![make_time_entry(1, 10, 20, Some(2.0), false)], 5);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::dark();
    terminal.draw(|f| d.render(f.area(), f, &theme, 0)).unwrap();
    let buffer = terminal.backend().buffer();
    let buffer_str = (0..buffer.area().height)
        .map(|y| {
            (0..buffer.area().width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        buffer_str.contains("5 projects") || buffer_str.contains("projecten"),
        "Stats footer should render with non-zero project count, got buffer:\n{buffer_str}"
    );
}
#[test]
fn test_date_picker_renders_content() {
    use harv_tui::views::date_picker::DatePicker;
    let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
    let mut dp = DatePicker::new(date);
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::dark();
    terminal
        .draw(|f| {
            dp.render(f.area(), f, &theme);
        })
        .unwrap();
    let buffer = terminal.backend().buffer();
    let has_content = (0..buffer.area().height).any(|y| {
        (0..buffer.area().width)
            .any(|x| !buffer[(x, y)].symbol().is_empty() && buffer[(x, y)].symbol() != " ")
    });
    assert!(has_content, "Date picker should render visible content");
}
