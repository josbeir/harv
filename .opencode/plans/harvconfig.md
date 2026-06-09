# .harvconfig — Per-Project Configuration

Place a `.harvconfig` TOML file in a project root to override defaults, add project-scoped aliases, and auto-fill notes with git info.

## File Format

```toml
# ~/code/myproject/.harvconfig

[defaults]
project_id = 12345
task_id = 67890

[notes]
template = "[{commit}] {message}"

[[aliases]]
name = "meeting"
project_id = 12345
task_id = 68000
```

## Notes Template Tokens

| Token | Source | Example |
|-------|--------|---------|
| `{branch}` | `git rev-parse --abbrev-ref HEAD` | `feat/login` |
| `{commit}` | Short hash, latest commit (7 chars) | `a3f8b2c` |
| `{message}` | Latest commit subject line | `fix: resolve login loop` |
| `{ticket}` | Ticket ID from branch name, then commit message | `PROJ-123` |
| `{repo}` | Basename from `git remote get-url origin` | `harv` |
| `{dir}` | Basename of current working directory | `harv-rust` |

Templates only apply when opening a NEW entry and the notes field is empty — never overwrite existing notes. Single-expansion, no nesting.

### `{ticket}` Smart Extraction

Searches branch name first, then commit message for patterns like `PROJ-123`:

```rust
fn extract_ticket(branch: &str, commit_msg: &str) -> Option<String> {
    let re = regex::Regex::new(r"([A-Z]{2,}-\d+)").unwrap();
    re.find(branch)
        .or_else(|| re.find(commit_msg))
        .map(|m| m.as_str().to_string())
}
```

## Precedence

```
CLI flags (--project-id, -p)
  > .harvconfig [defaults]
  > ~/.config/harv/config.json (last_project_id)
```

## Discovery

Walk upward from CWD, first `.harvconfig` found wins:

```
/home/user/code/frontend/src/
  → /home/user/code/frontend/src/.harvconfig  (try)
  → /home/user/code/frontend/.harvconfig       (found!)
```

## Architecture

### New Types (`harv-sdk/src/config.rs`)

```rust
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ProjectConfig {
    #[serde(default)]
    pub defaults: Option<ProjectDefaults>,
    #[serde(default)]
    pub aliases: Option<Vec<ProjectAlias>>,
    #[serde(default)]
    pub notes: Option<NoteTemplate>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectDefaults {
    pub project_id: Option<u64>,
    pub task_id: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectAlias {
    pub name: String,
    pub project_id: u64,
    pub task_id: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NoteTemplate {
    pub template: String,
}
```

### Loading & Resolution

```rust
impl ProjectConfig {
    /// Walk up from CWD, find nearest .harvconfig
    pub fn discover() -> Option<Self> { ... }

    /// Expand notes template with git tokens
    pub fn expand_notes(&self) -> Option<String> { ... }
}
```

### Git Helpers (internal)

```rust
fn git_branch() -> Option<String> { ... }     // git rev-parse --abbrev-ref HEAD
fn git_commit_short() -> Option<String> { ... } // git rev-parse --short=7 HEAD
fn git_commit_message() -> Option<String> { ... } // git log -1 --format=%s
fn git_repo_name() -> Option<String> { ... }  // git remote get-url origin → basename
```

### Merge with Global Config

```rust
impl HarvConfig {
    pub fn with_project(&self) -> MergedConfig {
        let proj = ProjectConfig::discover();
        MergedConfig { global: self, project: proj }
    }
}

pub struct MergedConfig<'a> {
    global: &'a HarvConfig,
    project: Option<ProjectConfig>,
}

impl MergedConfig<'_> {
    pub fn project_id(&self) -> Option<u64> { ... }
    pub fn task_id(&self) -> Option<u64> { ... }
    pub fn notes_template(&self) -> Option<String> { ... }
    pub fn resolve_alias(&self, name: &str) -> Option<&Alias> { ... }
}
```

## Integration Points

| File | Change |
|------|--------|
| `harv-sdk/Cargo.toml` | Add `toml = "0.8"` |
| `harv-sdk/src/config.rs` | Add `ProjectConfig`, `discover()`, `expand_notes()`, `MergedConfig` |
| `harv-cli/src/prompts.rs` | Use `MergedConfig` for default project/task selection |
| `harv-cli/src/commands/track.rs` | Check project-scoped aliases before global aliases |
| `harv-tui/src/views/form.rs` | Pre-fill notes from template if empty |
| `harv-tui/src/app.rs` | Use `MergedConfig` in `OpenForm` handler |

## Testing

| Test | What |
|------|------|
| `test_discover_finds_nearest` | Walk up from nested dir, verify right file found |
| `test_discover_not_found` | CWD with no `.harvconfig` returns None |
| `test_discover_parses_valid` | Parse a complete `.harvconfig` fixture |
| `test_expand_branch` | Template `"{branch}"` → actual git branch |
| `test_expand_ticket_from_branch` | Branch `feat/PROJ-123-login` → `PROJ-123` |
| `test_expand_ticket_from_commit` | Commit `[PROJ-456] Add feature` → `PROJ-456` |
| `test_expand_no_git_repo` | Graceful fallback when not in a git repo |
| `test_merged_config_precedence` | CLI flags > project config > global config |
| `test_project_alias_overrides_global` | Same alias name, project version wins |

## Implementation Order

| # | Step |
|---|------|
| 1 | Add `toml = "0.8"` to `harv-sdk/Cargo.toml` |
| 2 | Add `ProjectConfig` types + `discover()` to `harv-sdk/src/config.rs` |
| 3 | Add git helper functions + `expand_notes()` |
| 4 | Add `MergedConfig` struct with precedence logic |
| 5 | Integrate into `harv-cli` (prompts, track, alias) |
| 6 | Integrate into `harv-tui` (form, app) |
| 7 | Write tests |
| 8 | Update README with `.harvconfig` documentation |
