use std::collections::HashMap;
use std::sync::Mutex;
use unic_langid::LanguageIdentifier;

pub const SUPPORTED_LANGS: &[&str] = include!(concat!(env!("OUT_DIR"), "/supported_langs.rs"));

/// If a LanguageIdentifier has no region, derive the default region.
/// For "en" the PRD region is "US"; for all other languages the
/// two-letter language code uppercased is the region (nl→NL, fr→FR, etc.).
fn with_default_region(mut lid: LanguageIdentifier) -> LanguageIdentifier {
    if lid.region.is_none() {
        let lang = lid.language.as_str();
        let region_str = if lang == "en" { "US" } else { lang };
        if let Ok(region) = region_str.parse() {
            lid.region = Some(region);
        }
    }
    lid
}

static MESSAGES: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);
static CURRENT_LANG: Mutex<String> = Mutex::new(String::new());

/// Initialize the locale system. Called once at startup. Can be called multiple
/// times (e.g. after loading config with a locale override). Safe from any thread.
pub fn init(override_locale: Option<&str>) {
    let langid = resolve_locale(override_locale);
    let messages = load_messages(&langid);

    let mut msgs = MESSAGES.lock().unwrap_or_else(|e| e.into_inner());
    *msgs = Some(messages);
    let mut lang = CURRENT_LANG.lock().unwrap_or_else(|e| e.into_inner());
    *lang = langid.to_string();
}

/// Look up a message by key. Returns the key itself if locale is not
/// initialized or the key is not found.
pub fn t(key: &str) -> String {
    match MESSAGES.lock() {
        Ok(guard) => match guard.as_ref() {
            Some(map) => map.get(key).cloned().unwrap_or_else(|| key.to_string()),
            None => key.to_string(),
        },
        Err(_) => key.to_string(),
    }
}

/// Look up a message with arguments. Arguments are `(name, value)` pairs.
/// Performs simple { $name } substitution on the resolved pattern.
pub fn t_args(key: &str, args: &[(&str, String)]) -> String {
    let mut result = t(key);
    for (name, value) in args {
        result = result.replace(&format!("{{ ${} }}", name), value);
    }
    result
}

/// Returns the current language code (e.g. "en", "nl"), or "en" if not initialized.
pub fn current_langid() -> String {
    CURRENT_LANG
        .lock()
        .ok()
        .filter(|s| !s.is_empty())
        .map(|s| s.clone())
        .unwrap_or_else(|| "en".into())
}

fn load_messages(langid: &LanguageIdentifier) -> HashMap<String, String> {
    // Load English first as fallback, then overlay requested locale
    let mut messages = parse_ftl(include_str!("../locales/en/main.ftl"));

    let lang_code = langid.language.as_str();
    if lang_code != "en" {
        include!(concat!(env!("OUT_DIR"), "/lang_load.rs"));
    }

    messages
}

fn parse_ftl(source: &str) -> HashMap<String, String> {
    let mut messages = HashMap::new();
    let mut current_key: Option<String> = None;
    let mut current_value = String::new();

    for line in source.lines() {
        let trimmed = line.trim();

        // Skip comments and blank lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Check if this is a new message definition: key = value
        if let Some(eq_pos) = trimmed.find('=') {
            // Flush previous message
            if let Some(key) = current_key.take() {
                messages.insert(key, current_value.trim().to_string());
                current_value.clear();
            }

            let key = trimmed[..eq_pos].trim().to_string();
            let value = trimmed[eq_pos + 1..].trim().to_string();
            current_key = Some(key);

            // Check if this is a multiline value (indented continuation)
            if value.is_empty() {
                current_value.clear();
            } else {
                current_value = value;
            }
        } else if current_key.is_some() {
            // Continuation of a multiline value
            if !current_value.is_empty() {
                current_value.push('\n');
            }
            current_value.push_str(trimmed);
        }
    }

    // Flush last message
    if let Some(key) = current_key {
        messages.insert(key, current_value.trim().to_string());
    }

    messages
}

fn resolve_locale(override_locale: Option<&str>) -> LanguageIdentifier {
    let requested = override_locale.and_then(|loc| {
        let trimmed = loc.trim();
        let lid: LanguageIdentifier = trimmed.parse().ok()?;
        if SUPPORTED_LANGS.contains(&lid.language.as_str()) {
            Some(lid)
        } else {
            None
        }
    });

    if let Some(lid) = requested {
        return with_default_region(lid);
    }

    if let Some(sys_loc) = sys_locale::get_locale()
        && let Ok(lid) = sys_loc.parse::<LanguageIdentifier>()
        && SUPPORTED_LANGS.contains(&lid.language.as_str())
    {
        return with_default_region(lid);
    }

    with_default_region("en".parse().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[serial]
    #[test]
    fn test_simple_lookup() {
        init(None);
        let msg = t("err-not-authenticated");
        assert!(msg.contains("harv connect"));
        assert!(!msg.is_empty());
    }

    #[serial]
    #[test]
    fn test_lookup_with_args() {
        init(None);
        let msg = t_args(
            "err-api",
            &[
                ("status", "422".into()),
                ("message", "Validation failed".into()),
            ],
        );
        assert!(msg.contains("422"));
        assert!(msg.contains("Validation failed"));
    }

    #[serial]
    #[test]
    fn test_missing_key_fallback() {
        init(None);
        let msg = t("this-key-does-not-exist");
        assert_eq!(msg, "this-key-does-not-exist");
    }

    #[serial]
    #[test]
    fn test_english_fallback() {
        init(None);
        let msg = t_args("err-no-task-assignments", &[("project_id", "42".into())]);
        assert!(msg.contains("42"));
        assert!(msg.contains("project"));
    }

    #[serial]
    #[test]
    fn test_override_locale_unsupported() {
        init(Some("jp"));
        let lid = current_langid();
        assert_ne!(lid, "jp", "unsupported locale should not be used");
    }

    #[serial]
    #[test]
    fn test_override_locale_supported() {
        init(Some("nl"));
        let lid = current_langid();
        assert_eq!(lid, "nl-NL");
    }

    #[serial]
    #[test]
    fn test_override_locale_trimmed() {
        init(Some("  fr  "));
        let lid = current_langid();
        assert_eq!(lid, "fr-FR");
    }

    #[serial]
    #[test]
    fn test_current_langid() {
        init(Some("en"));
        assert_eq!(current_langid(), "en-US");
    }

    #[serial]
    #[test]
    fn test_all_supported_locales_init() {
        for lang in SUPPORTED_LANGS {
            init(Some(lang));
            assert!(current_langid().starts_with(*lang));
        }
    }

    #[serial]
    #[test]
    fn test_non_english_locale_falls_back_to_en() {
        init(Some("nl"));
        let msg = t("err-not-authenticated");
        assert!(msg.contains("harv connect"));
        assert!(!msg.is_empty());
    }

    #[serial]
    #[test]
    fn test_translations_return_real_messages() {
        let key = "err-not-authenticated";
        init(Some("en"));
        let en_msg = t(key);

        for lang in SUPPORTED_LANGS {
            if *lang == "en" {
                continue;
            }
            init(Some(lang));
            let msg = t(key);
            assert!(
                !msg.is_empty(),
                "{lang}: translation for {key} should not be empty"
            );
            assert_ne!(
                msg, key,
                "{lang}: translation for {key} should not fall back to key"
            );
            assert_ne!(
                msg, en_msg,
                "{lang}: translation for {key} should differ from English"
            );
        }
    }

    #[serial]
    #[test]
    fn test_translations_include_harv_connect() {
        for lang in SUPPORTED_LANGS {
            init(Some(lang));
            let msg = t("err-not-authenticated");
            assert!(
                msg.contains("harv connect"),
                "{lang}: err-not-authenticated should reference `harv connect`"
            );
        }
    }

    #[serial]
    #[test]
    fn test_t_args_with_multiple_args() {
        init(None);
        let msg = t_args(
            "cli-edit-success",
            &[
                ("id", "42".into()),
                ("hours", "2.5h".into()),
                ("date", "2026-06-11".into()),
                ("project", "Test".into()),
                ("task", "Dev".into()),
            ],
        );
        assert!(msg.contains("42"));
        assert!(msg.contains("2.5h"));
    }
}
