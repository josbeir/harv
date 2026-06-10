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

/// Returns true if the character at `idx` starts a new word.
fn is_word_boundary(chars: &[char], idx: usize) -> bool {
    idx == 0 || chars[idx - 1] == ' ' || chars[idx - 1] == '-' || chars[idx - 1] == '_'
}

/// Fuzzy-score a pattern against text. Returns -1 if no match.
///
/// Pattern characters must appear in order in the text (not necessarily
/// consecutive). Scoring rewards contiguous runs, word-boundary matches,
/// and early first-character position; it penalizes gaps between matches.
pub fn fuzzy_score(pattern: &str, text: &str) -> i32 {
    if pattern.is_empty() {
        return 0;
    }

    let pattern_lower = pattern.to_lowercase();
    let text_lower = text.to_lowercase();
    let pattern_chars: Vec<char> = pattern_lower.chars().collect();
    let text_chars: Vec<char> = text_lower.chars().collect();

    let mut score = 0i32;
    let mut pi = 0;
    let mut last_match: Option<usize> = None;
    let mut consecutive_streak = 0i32;

    for (ti, &tc) in text_chars.iter().enumerate() {
        if pi >= pattern_chars.len() {
            break;
        }
        if tc != pattern_chars[pi] {
            continue;
        }

        // Base point
        score += 1;

        // Consecutive bonus — growing reward for continuous runs
        let is_consecutive = last_match.is_some_and(|lm| ti == lm + 1);
        if is_consecutive {
            consecutive_streak += 1;
            score += consecutive_streak * 2;
        } else if let Some(lm) = last_match {
            // Gap penalty (capped)
            let gap = (ti - lm) as i32;
            score -= gap.min(10);
            consecutive_streak = 0;
        }

        // Word-boundary and first-match position bonuses
        if is_word_boundary(&text_chars, ti) {
            score += if pi == 0 { 8 } else { 4 };
        }
        if pi == 0 && ti == 0 {
            score += 4;
        }

        last_match = Some(ti);
        pi += 1;
    }

    if pi < pattern_chars.len() { -1 } else { score }
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
    fn test_format_timer_line() {
        use crate::types::Reference;
        use crate::types::TimeEntry;
        use chrono::{DateTime, Utc};

        let entry = TimeEntry {
            id: 1,
            spent_date: None,
            hours: None,
            notes: None,
            is_running: true,
            timer_started_at: Some(DateTime::<Utc>::MIN_UTC),
            started_time: None,
            ended_time: None,
            project: Reference {
                id: 10,
                name: "Dev Project".into(),
            },
            task: Reference {
                id: 20,
                name: "Coding".into(),
            },
            user: Reference {
                id: 1,
                name: "User".into(),
            },
            client: Some(Reference {
                id: 5,
                name: "Acme".into(),
            }),
            is_billed: false,
            billable: true,
            billable_rate: None,
            cost_rate: None,
            created_at: None,
            updated_at: None,
        };

        let line = format_timer_line(&entry);
        assert!(line.contains("Acme"));
        assert!(line.contains("Dev Project"));
        assert!(line.contains("Coding"));
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

    #[test]
    fn test_fuzzy_score_empty_pattern() {
        assert_eq!(fuzzy_score("", "anything"), 0);
    }

    #[test]
    fn test_fuzzy_score_contiguous_beats_scattered() {
        let contiguous = fuzzy_score("demo", "Demo Project - Alpha Phase Design");
        let scattered = fuzzy_score("demo", "Data Entry Module Operations Platform");
        assert!(
            contiguous > scattered,
            "contiguous {contiguous} should beat scattered {scattered}"
        );
    }

    #[test]
    fn test_fuzzy_score_prefix_bonus() {
        let at_start = fuzzy_score("dev", "development website");
        let mid_word = fuzzy_score("dev", "web developer");
        assert!(
            at_start > mid_word,
            "start {at_start} should beat mid-word {mid_word}"
        );
    }

    #[test]
    fn test_fuzzy_score_word_boundary_bonus() {
        let at_boundary = fuzzy_score("imp", "important implementation");
        // "imp" in "important" at pos 0 — word boundary, start bonus
        // vs "imp" starting somewhere inside
        assert!(at_boundary > 0);
    }

    #[test]
    fn test_fuzzy_score_consecutive_bonus_grows() {
        let short_run = fuzzy_score("de", "development");
        let long_run = fuzzy_score("devel", "development");
        assert!(
            long_run > short_run,
            "longer consecutive run should score higher"
        );
    }

    #[test]
    fn test_fuzzy_score_case_insensitive() {
        assert_eq!(
            fuzzy_score("ACME", "Acme Corporation - Launch Campaign"),
            fuzzy_score("acme", "Acme Corporation - Launch Campaign"),
        );
    }

    #[test]
    fn test_fuzzy_score_no_match_on_missing_char() {
        let matched = fuzzy_score("acme", "Acme Corp - Alpha Project");
        let no_match = fuzzy_score("acme", "Basic Tools Workflow");
        assert!(matched > 0);
        assert_eq!(
            no_match, -1,
            "should not match when a pattern char is missing"
        );
    }

    #[test]
    fn test_fuzzy_score_single_char() {
        let s = fuzzy_score("t", "test");
        assert!(s > 0);
        // single char at start: base=1 + word_boundary(pi=0)+8 + ti=0+4 = 13
        assert!(
            s >= 10,
            "single char at start should have high score, got {s}"
        );
    }

    #[test]
    fn test_fuzzy_score_unicode_safe() {
        assert!(fuzzy_score("brûlée", "crème brûlée") > 0);
        assert_eq!(fuzzy_score("xyz", "crème brûlée"), -1);
    }
}
