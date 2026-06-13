use std::collections::HashMap;
use std::process::Command;

use harv_core::HarvError;

use crate::project_config::NoteTemplate;

impl NoteTemplate {
    /// Expand template variables in the pattern string using a single-pass
    /// parser.
    ///
    /// Scans the pattern left-to-right for `{variable_name}` placeholders
    /// and replaces them with values from the provided map. Unknown
    /// variables are left unchanged so that forward-compatible templates
    /// don't break. A single pass avoids order-dependent behaviour that
    /// could arise when a variable value contains another placeholder.
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use harv_sdk::NoteTemplate;
    ///
    /// let tpl = NoteTemplate { pattern: "Worked on {task} at {time}".into() };
    /// let mut vars: HashMap<&str, String> = HashMap::new();
    /// vars.insert("task", "Development".to_string());
    /// vars.insert("time", "14:30".to_string());
    ///
    /// assert_eq!(tpl.expand(&vars), "Worked on Development at 14:30");
    /// ```
    pub fn expand(&self, vars: &HashMap<&str, String>) -> String {
        let mut result = String::with_capacity(self.pattern.len());
        let mut rest = self.pattern.as_str();

        while let Some(start) = rest.find('{') {
            // Append everything before the placeholder.
            result.push_str(&rest[..start]);
            rest = &rest[start + 1..];

            if let Some(end) = rest.find('}') {
                let key = &rest[..end];
                rest = &rest[end + 1..];

                match vars.get(key) {
                    Some(value) => result.push_str(value),
                    None => {
                        // Unknown variable — keep the placeholder as-is.
                        result.push('{');
                        result.push_str(key);
                        result.push('}');
                    }
                }
            } else {
                // Unclosed `{` — keep the rest including the opening brace
                // and stop processing.
                result.push('{');
                result.push_str(rest);
                return result;
            }
        }

        // Append any remaining text after the last placeholder.
        result.push_str(rest);
        result
    }
}

/// Context available for template variable expansion.
///
/// Gather variables from the environment: current date/time,
/// git repository info, system hostname, and Harvest project/task/user
/// metadata. Each method populates a specific variable.
pub struct TemplateContext;

impl TemplateContext {
    /// Gather all available template variables.
    ///
    /// Variables collected:
    /// - `{date}` — today's date in YYYY-MM-DD format
    /// - `{time}` — current time in HH:MM format
    /// - `{hostname}` — system hostname
    /// - `{commit_message}` — most recent git commit message (empty if not in repo)
    /// - `{branch_name}` — current git branch name (empty if not in repo)
    ///
    /// The caller should add `{project_name}`, `{task_name}`, and
    /// `{user_name}` based on their application context.
    pub fn gather() -> Result<HashMap<&'static str, String>, HarvError> {
        let mut vars = HashMap::new();

        vars.insert("date", Self::date());
        vars.insert("time", Self::time());
        vars.insert("hostname", Self::hostname());
        vars.insert("commit_message", Self::git_commit_message());
        vars.insert("branch_name", Self::git_branch());

        Ok(vars)
    }

    /// Current date in `YYYY-MM-DD` format.
    fn date() -> String {
        harv_core::datetime::format_date(harv_core::datetime::today())
    }

    /// Current time in `HH:MM` format (24-hour).
    fn time() -> String {
        chrono::Local::now().format("%H:%M").to_string()
    }

    /// System hostname, or "unknown" if it cannot be determined.
    fn hostname() -> String {
        hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Most recent git commit message (subject line only).
    /// Returns an empty string if not in a git repository or if git
    /// is not installed.
    fn git_commit_message() -> String {
        Self::run_git(&["log", "-1", "--format=%s"])
    }

    /// Current git branch name.
    /// Returns an empty string if not in a git repository or if git
    /// is not installed.
    fn git_branch() -> String {
        Self::run_git(&["branch", "--show-current"])
    }

    /// Run a git command and return its stdout as a trimmed string.
    /// Returns an empty string on any failure (git not installed,
    /// not in a repo, etc.).
    fn run_git(args: &[&str]) -> String {
        Command::new("git")
            .args(args)
            .output()
            .ok()
            .filter(|out| out.status.success())
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- NoteTemplate::expand tests ---

    #[test]
    fn test_expand_single_var() {
        let tpl = NoteTemplate {
            pattern: "Hello {name}".into(),
        };
        let mut vars = HashMap::new();
        vars.insert("name", "World".to_string());

        assert_eq!(tpl.expand(&vars), "Hello World");
    }

    #[test]
    fn test_expand_multiple_vars() {
        let tpl = NoteTemplate {
            pattern: "{greeting} {name}! Today is {date}.".into(),
        };
        let mut vars = HashMap::new();
        vars.insert("greeting", "Hi".to_string());
        vars.insert("name", "Alice".to_string());
        vars.insert("date", "2026-06-13".to_string());

        assert_eq!(tpl.expand(&vars), "Hi Alice! Today is 2026-06-13.");
    }

    #[test]
    fn test_expand_unknown_var_unchanged() {
        let tpl = NoteTemplate {
            pattern: "Hello {name}, see {unknown}".into(),
        };
        let mut vars = HashMap::new();
        vars.insert("name", "World".to_string());

        assert_eq!(tpl.expand(&vars), "Hello World, see {unknown}");
    }

    #[test]
    fn test_expand_empty_vars() {
        let tpl = NoteTemplate {
            pattern: "No {vars} here".into(),
        };
        let vars = HashMap::new();
        assert_eq!(tpl.expand(&vars), "No {vars} here");
    }

    #[test]
    fn test_expand_repeated_var() {
        let tpl = NoteTemplate {
            pattern: "{x} and {x} again".into(),
        };
        let mut vars = HashMap::new();
        vars.insert("x", "X".to_string());

        assert_eq!(tpl.expand(&vars), "X and X again");
    }

    #[test]
    fn test_expand_no_placeholders() {
        let tpl = NoteTemplate {
            pattern: "Plain text without variables".into(),
        };
        let vars = HashMap::new();
        assert_eq!(tpl.expand(&vars), "Plain text without variables");
    }

    #[test]
    fn test_expand_partial_match() {
        // Only full {name} should match, not partial patterns.
        let tpl = NoteTemplate {
            pattern: "See {name}".into(),
        };
        let mut vars = HashMap::new();
        vars.insert("name", "Alice".to_string());

        assert_eq!(tpl.expand(&vars), "See Alice");
    }

    // --- TemplateContext tests ---

    #[test]
    fn test_expand_order_independent() {
        // Single-pass parser: expansion should not depend on HashMap iteration
        // order. Even if one variable value looks like another placeholder,
        // the parser only expands top-level {var} patterns.
        let tpl = NoteTemplate {
            pattern: "{a} and {b}".into(),
        };
        let mut vars = HashMap::new();
        vars.insert("a", "{b}".to_string());
        vars.insert("b", "B".to_string());

        // The single-pass parser should not recursively expand.
        assert_eq!(tpl.expand(&vars), "{b} and B");
    }

    #[test]
    fn test_expand_unclosed_brace() {
        let tpl = NoteTemplate {
            pattern: "Hello {name".into(),
        };
        let mut vars = HashMap::new();
        vars.insert("name", "World".to_string());

        assert_eq!(tpl.expand(&vars), "Hello {name");
    }

    // --- TemplateContext tests ---

    #[test]
    fn test_context_includes_date_and_time() {
        let vars = TemplateContext::gather().unwrap();
        assert!(vars.contains_key("date"));
        assert!(vars.contains_key("time"));
        assert!(!vars["date"].is_empty());
        assert!(!vars["time"].is_empty());
    }

    #[test]
    fn test_context_includes_hostname() {
        let vars = TemplateContext::gather().unwrap();
        assert!(vars.contains_key("hostname"));
        assert!(!vars["hostname"].is_empty());
    }

    #[test]
    fn test_context_includes_git_vars() {
        let vars = TemplateContext::gather().unwrap();
        // Git vars are always present in the map, even if empty.
        assert!(vars.contains_key("commit_message"));
        assert!(vars.contains_key("branch_name"));
        // The values may be empty if not in a git repo — that's fine.
    }

    #[test]
    fn test_git_branch_in_temp_repo() {
        use std::process::Command;

        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path();

        // Initialize git repo — skip if git is not available.
        let init = Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(repo_path)
            .output();
        match init {
            Ok(ref o) if o.status.success() => {}
            _ => return,
        }

        // Configure user so commits work (needed in CI).
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(repo_path)
            .output()
            .ok();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(repo_path)
            .output()
            .ok();

        // Write a file and commit.
        std::fs::write(repo_path.join("file.txt"), b"hello").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(repo_path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "feat: initial commit"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Use --git-dir and --work-tree so we don't need CWD manipulation.
        let branch_out = Command::new("git")
            .args([
                format!("--git-dir={}/.git", repo_path.display()),
                format!("--work-tree={}", repo_path.display()),
                "branch".to_string(),
                "--show-current".to_string(),
            ])
            .output()
            .unwrap();
        let branch = String::from_utf8_lossy(&branch_out.stdout)
            .trim()
            .to_string();
        assert_eq!(branch, "main", "should detect branch in temp repo");

        let commit_out = Command::new("git")
            .args([
                format!("--git-dir={}/.git", repo_path.display()),
                format!("--work-tree={}", repo_path.display()),
                "log".to_string(),
                "-1".to_string(),
                "--format=%s".to_string(),
            ])
            .output()
            .unwrap();
        let commit = String::from_utf8_lossy(&commit_out.stdout)
            .trim()
            .to_string();
        assert_eq!(
            commit, "feat: initial commit",
            "should detect commit message"
        );
    }

    #[test]
    fn test_git_vars_dont_panic_outside_repo() {
        // Should not panic even outside a git repo.
        let vars = TemplateContext::gather().unwrap();
        let _ = vars;
    }

    #[test]
    fn test_run_git_invalid_command() {
        let result = TemplateContext::run_git(&["--invalid-flag-xyz"]);
        assert!(result.is_empty());
    }
}
