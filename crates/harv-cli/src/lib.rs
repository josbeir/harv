pub mod commands;
pub(crate) mod output;
pub(crate) mod prompts;
pub(crate) mod resolution;
pub(crate) mod spinner;

use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

/// Harv — A CLI for Harvest time tracking.
#[derive(Parser)]
#[command(name = "harv", version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, global = true, default_value = "table")]
    pub output: OutputFormat,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Authenticate with Harvest via OAuth2
    Connect,
    /// Disconnect and remove stored credentials
    Disconnect,
    /// Show or modify configuration
    Config(ConfigArgs),
    /// Interactive time entry wizard
    #[command(visible_alias = "log")]
    Track(TrackArgs),
    /// Start a running timer
    Start(StartArgs),
    /// Stop the current running timer
    Stop(StopArgs),
    /// Edit notes on the running timer
    Note(NoteArgs),
    /// Edit an existing time entry
    Edit(EditArgs),
    /// Show current timer status and today's entries
    Status,
    /// Show authenticated user info and login status
    Whoami,
    /// List your project assignments
    Projects(ProjectsArgs),
    /// List tasks for a project
    Tasks(TasksArgs),
    /// Manage project/task aliases
    #[command(subcommand)]
    Alias(AliasCommand),
    /// Initialize a project config file (harv.toml) in the current directory
    Init(InitArgs),
    /// Generate shell completion script
    Completion(CompletionArgs),
}

#[derive(clap::Args, Clone, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: Option<ConfigAction>,
}

#[derive(Subcommand, Clone, Debug)]
pub enum ConfigAction {
    /// Set a configuration value
    Set { setting: String, value: String },
    /// Get a configuration value
    Get { setting: String },
}

#[derive(clap::Args, Clone, Debug)]
pub struct TrackArgs {
    #[arg(short = 'p', long)]
    pub project_id: Option<u64>,
    #[arg(short = 't', long)]
    pub task_id: Option<u64>,
    #[arg(short = 'H', long, value_parser = parse_hours_arg)]
    pub hours: Option<f64>,
    #[arg(short = 'n', long)]
    pub notes: Option<String>,
    #[arg(short = 'e', long)]
    pub editor: bool,
    #[arg(short = 'd', long)]
    pub date: Option<String>,
    #[arg(short = 'R', long)]
    pub refresh: bool,
    pub alias: Option<String>,
}

#[derive(clap::Args, Clone, Debug)]
pub struct StartArgs {
    #[arg(short = 'p', long)]
    pub project_id: Option<u64>,
    #[arg(short = 't', long)]
    pub task_id: Option<u64>,
    #[arg(short = 'n', long)]
    pub notes: Option<String>,
    #[arg(short = 'e', long)]
    pub editor: bool,
    #[arg(short = 'd', long)]
    pub date: Option<String>,
    #[arg(short = 'R', long)]
    pub refresh: bool,
    pub alias: Option<String>,
}

#[derive(clap::Args, Clone, Debug)]
pub struct StopArgs {
    #[arg(short = 'n', long)]
    pub notes: Option<String>,
    #[arg(long)]
    pub overwrite: bool,
    #[arg(short = 'e', long)]
    pub editor: bool,
}

#[derive(clap::Args, Clone, Debug)]
pub struct NoteArgs {
    #[arg(short = 'n', long)]
    pub notes: Option<String>,
    #[arg(long)]
    pub overwrite: bool,
    #[arg(short = 'e', long)]
    pub editor: bool,
}

#[derive(clap::Args, Clone, Debug)]
pub struct EditArgs {
    /// ID of the time entry to edit (if omitted, pick from list)
    pub entry_id: Option<u64>,
    #[arg(short = 'p', long)]
    pub project_id: Option<u64>,
    #[arg(short = 't', long)]
    pub task_id: Option<u64>,
    #[arg(short = 'H', long, value_parser = parse_hours_arg)]
    pub hours: Option<f64>,
    #[arg(short = 'n', long)]
    pub notes: Option<String>,
    #[arg(short = 'e', long)]
    pub editor: bool,
    #[arg(long)]
    pub overwrite: bool,
    #[arg(short = 'd', long)]
    pub date: Option<String>,
    #[arg(short = 'R', long)]
    pub refresh: bool,
}

#[derive(clap::Args, Clone, Debug)]
pub struct ProjectsArgs {
    #[arg(short = 's', long)]
    pub search: Option<String>,
    #[arg(short = 'R', long)]
    pub refresh: bool,
}

#[derive(clap::Args, Clone, Debug)]
pub struct TasksArgs {
    pub project_id: u64,
}

#[derive(Subcommand, Clone, Debug)]
pub enum AliasCommand {
    /// Create a new project/task alias (interactive if name omitted)
    Create {
        /// Alias name (prompts interactively if omitted)
        name: Option<String>,
    },
    /// List all aliases
    List,
    /// Delete an alias by name
    Delete {
        /// Alias name to delete
        name: String,
    },
}

#[derive(clap::Args, Clone, Debug)]
pub struct InitArgs {
    /// Default project ID for this project config
    #[arg(short = 'p', long)]
    pub project_id: Option<u64>,

    /// Default task ID for this project config
    #[arg(short = 't', long)]
    pub task_id: Option<u64>,

    /// Add a note template in name=pattern format (repeatable)
    #[arg(long = "template", value_name = "NAME=PATTERN")]
    pub template: Vec<String>,

    /// Add a project alias in name=project_id:task_id format (repeatable)
    #[arg(long = "alias", value_name = "NAME=PID:TID")]
    pub alias: Vec<String>,

    /// Overwrite existing harv.toml without confirmation
    #[arg(short = 'f', long)]
    pub force: bool,
}

#[derive(clap::Args, Clone, Debug)]
pub struct CompletionArgs {
    pub shell: Shell,
}

pub fn setup_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("harv=warn")),
        )
        .init();
}

/// In mock mode: write a minimal config.toml so the auth check
/// passes. The actual mock client is created via
/// `HarvClient::from_config_or_mock()` inside each command.
#[cfg(feature = "mock-mode")]
pub fn ensure_mock_config() -> color_eyre::eyre::Result<()> {
    if !harv_sdk::HarvConfig::path().exists() {
        let cfg = harv_sdk::mock_data::test_config();
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async { cfg.save().await })?;
    }
    Ok(())
}

/// Custom clap value parser for hours. Accepts decimal (1.5) or HH:MM (1:30).
fn parse_hours_arg(s: &str) -> Result<f64, String> {
    harv_core::datetime::parse_hours(s)
}
