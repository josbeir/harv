use chrono::{DateTime, Local, NaiveDate, Timelike, Utc};

/// Returns today's date in the local timezone.
pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

/// Formats a NaiveDate as an ISO 8601 date string (YYYY-MM-DD).
pub fn format_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Locale-aware date header for the TUI date navigation bar.
///
/// Uses ICU4X for locale-appropriate format (weekday + date). `locale` should
/// be a region-qualified BCP-47 string like "en-US" or "nl-NL".
/// English: "Thu, Jun 11, 2026" — Dutch: "do 11 jun 2026"
pub fn format_date_header(date: NaiveDate, locale: &str) -> String {
    let Ok(locale) = locale.parse::<icu::locale::Locale>() else {
        return date.format("%a, %b %e, %Y").to_string();
    };
    let Ok(formatter) = icu::datetime::DateTimeFormatter::try_new(
        locale.into(),
        icu::datetime::fieldsets::YMDE::medium(),
    ) else {
        return date.format("%a, %b %e, %Y").to_string();
    };
    formatter.format(&date).to_string()
}

/// Locale-aware short date for dashboard entries.
///
/// `locale` should be a region-qualified BCP-47 string like "en-US" or "nl-NL".
/// English: "Jun 11, 2026" — Dutch: "11 jun 2026"
pub fn format_date_short(date: NaiveDate, locale: &str) -> String {
    let Ok(locale) = locale.parse::<icu::locale::Locale>() else {
        return date.format("%b %e, %Y").to_string();
    };
    let Ok(formatter) = icu::datetime::DateTimeFormatter::try_new(
        locale.into(),
        icu::datetime::fieldsets::YMD::medium(),
    ) else {
        return date.format("%b %e, %Y").to_string();
    };
    formatter.format(&date).to_string()
}

/// Locale-aware month+year for the date picker title.
///
/// `locale` should be a region-qualified BCP-47 string like "en-US" or "nl-NL".
/// English: "June 2026" — Dutch: "juni 2026"
pub fn format_date_month_year(date: NaiveDate, locale: &str) -> String {
    let Ok(locale) = locale.parse::<icu::locale::Locale>() else {
        return date.format("%B %Y").to_string();
    };
    let Ok(formatter) = icu::datetime::DateTimeFormatter::try_new(
        locale.into(),
        icu::datetime::fieldsets::YM::long(),
    ) else {
        return date.format("%B %Y").to_string();
    };
    formatter.format(&date).to_string()
}

/// Formats a DateTime<Utc> as a localized display string.
///
/// Example: "Jun 8, 2026" or "Jun 8, 2026 at 14:30"
pub fn format_local(dt: DateTime<Utc>, include_time: bool) -> String {
    let local = dt.with_timezone(&Local);
    if include_time {
        local.format("%b %-d, %Y at %H:%M").to_string()
    } else {
        local.format("%b %-d, %Y").to_string()
    }
}

/// Parses a date string in ISO 8601 format (YYYY-MM-DD).
pub fn parse_date(s: &str) -> Result<NaiveDate, crate::HarvError> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|_| crate::HarvError::InvalidDate(s.to_string()))
}

/// Parses a date string and validates it is not in the future.
pub fn parse_date_not_future(s: &str) -> Result<NaiveDate, crate::HarvError> {
    let date = parse_date(s)?;
    if date > today() {
        return Err(crate::HarvError::InvalidDate(format!(
            "Date {} is in the future",
            s
        )));
    }
    Ok(date)
}

/// Formats a time in "HH:MMam/pm" format (e.g., "2:30pm").
///
/// This matches the Harvest API format for `started_time` and `ended_time`.
pub fn format_time(dt: DateTime<Utc>) -> String {
    let local = dt.with_timezone(&Local);
    let hour = local.hour();
    let ampm = if hour < 12 { "am" } else { "pm" };
    let hour12 = if hour == 0 {
        12
    } else if hour > 12 {
        hour - 12
    } else {
        hour
    };
    format!("{}:{:02}{}", hour12, local.minute(), ampm)
}

/// Parse hours from decimal (`1.5`) or HH:MM (`1:30`) format.
///
/// Returns decimal hours as `f64`.
pub fn parse_hours(input: &str) -> Result<f64, String> {
    if input.contains(':') {
        let parts: Vec<&str> = input.split(':').collect();
        if parts.len() != 2 {
            return Err("Use HH:MM format (e.g. 1:30)".into());
        }
        let hours: f64 = parts[0]
            .parse()
            .map_err(|_| "Invalid hours in HH:MM".to_string())?;
        let minutes: f64 = parts[1]
            .parse()
            .map_err(|_| "Invalid minutes in HH:MM".to_string())?;
        if !(0.0..60.0).contains(&minutes) {
            return Err("Minutes must be 0-59".into());
        }
        if hours < 0.0 {
            return Err("Hours must be non-negative".into());
        }
        Ok(hours + minutes / 60.0)
    } else {
        let h: f64 = input
            .parse()
            .map_err(|_| "Enter a valid number or HH:MM format (e.g. 1:30)".to_string())?;
        if h < 0.0 {
            return Err("Hours must be non-negative".into());
        }
        Ok(h)
    }
}

/// Calculates the `started_time` and `ended_time` values for a completed
/// time entry based on the current time and the number of hours worked.
///
/// Returns (started_time, ended_time) as "HH:MMam/pm" format strings.
pub fn time_window(hours: f64) -> (String, String) {
    let now = Utc::now();
    let seconds = (hours * 3600.0) as i64;
    let start = now - chrono::Duration::seconds(seconds);
    (format_time(start), format_time(now))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_today_is_valid() {
        let date = today();
        // Should always be some reasonable year
        assert!(date.year() >= 2024);
    }

    #[test]
    fn test_format_date() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 8).unwrap();
        assert_eq!(format_date(date), "2026-06-08");
    }

    #[test]
    fn test_parse_date_valid() {
        let date = parse_date("2026-06-08").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2026, 6, 8).unwrap());
    }

    #[test]
    fn test_parse_date_invalid() {
        assert!(parse_date("not-a-date").is_err());
        assert!(parse_date("2026-13-01").is_err());
    }

    #[test]
    fn test_parse_date_future() {
        let far_future = format_date(today() + chrono::Duration::days(365));
        assert!(parse_date_not_future(&far_future).is_err());
    }

    #[test]
    fn test_format_time() {
        // 14:30 UTC = depends on local timezone, but format should be consistent
        let dt = Utc::now();
        let formatted = format_time(dt);
        assert!(formatted.contains(':'));
        assert!(formatted.ends_with("am") || formatted.ends_with("pm"));
    }

    #[test]
    fn test_time_window() {
        let (start, end) = time_window(2.0);
        assert!(start.contains(':'));
        assert!(end.contains(':'));
        assert!(start.ends_with("am") || start.ends_with("pm"));
        assert!(end.ends_with("am") || end.ends_with("pm"));
    }

    #[test]
    fn test_time_window_zero_hours() {
        let (start, end) = time_window(0.0);
        let start_clean = &start[..start.len() - 2];
        let end_clean = &end[..end.len() - 2];
        assert_eq!(start_clean, end_clean);
    }

    #[test]
    fn test_parse_date_not_future_allows_today() {
        let today_str = format_date(today());
        assert!(parse_date_not_future(&today_str).is_ok());
    }

    #[test]
    fn test_format_local() {
        let dt = Utc::now();
        let formatted = format_local(dt, false);
        assert!(!formatted.contains("at"));
        let formatted = format_local(dt, true);
        assert!(formatted.contains("at"));
    }

    #[test]
    fn test_parse_date_invalid_format() {
        assert!(parse_date("06-08-2026").is_err());
        assert!(parse_date("June 8, 2026").is_err());
    }

    #[test]
    fn test_parse_hours_decimal() {
        assert!((parse_hours("1.5").unwrap() - 1.5).abs() < f64::EPSILON);
        assert!((parse_hours("0.0").unwrap() - 0.0).abs() < f64::EPSILON);
        assert!((parse_hours("0").unwrap() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_hours_hhmm() {
        assert!((parse_hours("1:30").unwrap() - 1.5).abs() < f64::EPSILON);
        assert!((parse_hours("0:30").unwrap() - 0.5).abs() < f64::EPSILON);
        assert!((parse_hours("2:15").unwrap() - 2.25).abs() < f64::EPSILON);
        assert!((parse_hours("0:00").unwrap() - 0.0).abs() < f64::EPSILON);
        assert!((parse_hours("00:45").unwrap() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_hours_invalid() {
        assert!(parse_hours("abc").is_err());
        assert!(parse_hours("1:75").is_err());
        assert!(parse_hours("-1:30").is_err());
        assert!(parse_hours(":").is_err());
        assert!(parse_hours("1:30:00").is_err());
    }

    #[test]
    fn test_format_date_header_nl() {
        let d = NaiveDate::from_ymd_opt(2026, 6, 11).unwrap();
        let s = format_date_header(d, "nl-NL");
        assert!(s.contains("jun"), "should contain Dutch month: {s}");
        assert!(s.contains("do"), "should contain Dutch weekday: {s}");
    }

    #[test]
    fn test_format_date_short_nl() {
        let d = NaiveDate::from_ymd_opt(2026, 6, 11).unwrap();
        let s = format_date_short(d, "nl-NL");
        assert!(s.contains("jun"), "should contain Dutch month: {s}");
        assert!(s.contains("11"), "should contain day: {s}");
    }

    #[test]
    fn test_format_date_month_year_nl() {
        let d = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let s = format_date_month_year(d, "nl-NL");
        assert!(s.contains("juni"), "should contain full Dutch month: {s}");
        assert!(s.contains("2026"), "should contain year: {s}");
        assert!(!s.contains("1"), "should not contain day: {s}");
    }

    #[test]
    fn test_icu4x_formatters_output_localized() {
        use icu::datetime::DateTimeFormatter;
        use icu::datetime::fieldsets;

        let date = NaiveDate::from_ymd_opt(2026, 6, 11).unwrap();

        // Create formatter for `en` and verify English output
        let en: icu::locale::Locale = "en-US".parse().unwrap();
        let fmt = DateTimeFormatter::try_new(en.into(), fieldsets::YMDE::medium()).unwrap();
        let s = fmt.format(&date).to_string();
        assert!(
            s.contains("Jun") || s.contains("Thu,"),
            "ICU4X en-US should contain English month or weekday, got: {s}"
        );

        // Create formatter for `nl` (bare) and `nl-NL` (region-qualified)
        let nl: icu::locale::Locale = "nl".parse().unwrap();
        let fmt = DateTimeFormatter::try_new(nl.into(), fieldsets::YMDE::medium()).unwrap();
        let s = fmt.format(&date).to_string();
        assert!(
            s.contains("jun") || s.contains("do"),
            "ICU4X bare nl should contain Dutch month or weekday, got: {s}"
        );

        let nl_nl: icu::locale::Locale = "nl-NL".parse().unwrap();
        let fmt = DateTimeFormatter::try_new(nl_nl.into(), fieldsets::YMDE::medium()).unwrap();
        let s = fmt.format(&date).to_string();
        assert!(
            s.contains("jun") || s.contains("do"),
            "ICU4X nl-NL should contain Dutch month or weekday, got: {s}"
        );
    }
}
