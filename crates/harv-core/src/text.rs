use crate::types::{Reference, TimeEntry};

/// Truncate a string to `max_chars` characters, appending "..." if truncated.
///
/// Uses char boundaries (not bytes) so it's safe with Unicode.
pub fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let end = max_chars.saturating_sub(3);
    if end == 0 {
        return "...".to_string();
    }
    format!("{}...", s.chars().take(end).collect::<String>())
}

/// Format hours as a short string, e.g. `"2.50h"`.
pub fn format_hours(hours: f64) -> String {
    format!("{:.2}h", hours)
}

/// Append a new note to existing notes, separated by a blank line.
pub fn append_notes(existing: &str, new: &str) -> String {
    format!("{}\n\n{}", existing, new)
}

/// Format a `TimeEntry` as a one-line summary suitable for selection lists.
pub fn format_timer_line(entry: &TimeEntry) -> String {
    format!(
        "[{}] {} => {} => {}",
        entry
            .timer_started_at
            .map(|ts| crate::datetime::format_local(ts, true))
            .unwrap_or_default(),
        client_name_or_default(&entry.client),
        entry.project.name,
        entry.task.name,
    )
}

/// Returns the client name, or `"No client"` if none.
pub fn client_name_or_default(client: &Option<Reference>) -> &str {
    client
        .as_ref()
        .map(|c| c.name.as_str())
        .unwrap_or("No client")
}

/// Format elapsed seconds as `HH:MM:SS`.
pub fn format_elapsed_hms(total_secs: i64) -> String {
    let secs = total_secs.max(0);
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

/// Fuzzy-score a pattern against text. Returns -1 if no match.
///
/// Characters in the pattern must appear in order in the text
/// (not necessarily consecutively). Higher scores indicate
/// better matches (consecutive matches score higher).
pub fn fuzzy_score(pattern: &str, text: &str) -> i32 {
    let pattern = pattern.to_lowercase();
    let text = text.to_lowercase();
    let mut score = 0;
    let mut text_chars = text.chars();
    for p in pattern.chars() {
        loop {
            match text_chars.next() {
                Some(t) if t == p => {
                    score += 1;
                    break;
                }
                Some(_) => {}
                None => return -1,
            }
        }
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;

    mod truncate {
        use super::*;

        #[test]
        fn no_truncation_needed() {
            assert_eq!(truncate("hello", 10), "hello");
        }

        #[test]
        fn truncation_with_ellipsis() {
            assert_eq!(truncate("hello world this is long", 12), "hello wor...");
        }

        #[test]
        fn exact_length() {
            assert_eq!(truncate("hello", 5), "hello");
        }

        #[test]
        fn very_short_max() {
            assert_eq!(truncate("hello", 3), "...");
            assert_eq!(truncate("hello", 2), "...");
        }

        #[test]
        fn unicode_chars() {
            assert_eq!(truncate("héllo wörld", 9), "héllo ...");
        }

        #[test]
        fn empty() {
            assert_eq!(truncate("", 5), "");
        }
    }

    #[test]
    fn test_format_hours() {
        assert_eq!(format_hours(2.5), "2.50h");
        assert_eq!(format_hours(0.0), "0.00h");
    }

    #[test]
    fn test_append_notes() {
        assert_eq!(append_notes("old", "new"), "old\n\nnew");
    }

    #[test]
    fn test_append_notes_empty_existing() {
        assert_eq!(append_notes("", "new"), "\n\nnew");
    }

    #[test]
    fn test_client_name_or_default_some() {
        let r = Reference {
            id: 1,
            name: "Acme".into(),
        };
        assert_eq!(client_name_or_default(&Some(r)), "Acme");
    }

    #[test]
    fn test_client_name_or_default_none() {
        assert_eq!(client_name_or_default(&None), "No client");
    }

    #[test]
    fn test_format_elapsed_hms() {
        assert_eq!(format_elapsed_hms(0), "00:00:00");
        assert_eq!(format_elapsed_hms(3661), "01:01:01");
        assert_eq!(format_elapsed_hms(86399), "23:59:59");
    }

    #[test]
    fn test_fuzzy_score_exact() {
        assert!(fuzzy_score("dev", "Development") > 0);
    }

    #[test]
    fn test_fuzzy_score_no_match() {
        assert_eq!(fuzzy_score("xyz", "Development"), -1);
    }

    #[test]
    fn test_fuzzy_score_substring() {
        assert!(fuzzy_score("De", "Development") > 0);
    }
}
