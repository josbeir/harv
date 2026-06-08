use chrono::{DateTime, Local, NaiveDate, Timelike, Utc};

/// Returns today's date in the local timezone.
pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

/// Formats a NaiveDate as an ISO 8601 date string (YYYY-MM-DD).
pub fn format_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
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
}
