pub mod commands;
pub(crate) mod output;
pub(crate) mod prompts;
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
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Authenticate with Harvest via OAuth2
    Connect,
    /// Show current configuration
    Config,
    /// Interactive time entry wizard
    Track(TrackArgs),
    /// Start a running timer
    Start(StartArgs),
    /// Stop the current running timer
    Stop(StopArgs),
    /// Log time with specified hours
    Log(LogArgs),
    /// Edit notes on the running timer
    Note(NoteArgs),
    /// Show current timer status and today's entries
    Status,
    /// List your project assignments
    Projects(ProjectsArgs),
    /// List tasks for a project
    Tasks(TasksArgs),
    /// Manage project/task aliases
    #[command(subcommand)]
    Alias(AliasCommand),
    /// Generate shell completion script
    Completion(CompletionArgs),
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
pub struct LogArgs {
    #[arg(value_parser = parse_hours_arg)]
    pub hours: f64,
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
    pub alias: Option<String>,
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
pub struct ProjectsArgs {
    #[arg(short = 's', long)]
    pub search: Option<String>,
}

#[derive(clap::Args, Clone, Debug)]
pub struct TasksArgs {
    pub project_id: u64,
}

#[derive(Subcommand, Clone, Debug)]
pub enum AliasCommand {
    Create { name: String },
    List,
    Delete { name: String },
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

/// Custom clap value parser for hours. Accepts decimal (1.5) or HH:MM (1:30).
fn parse_hours_arg(s: &str) -> Result<f64, String> {
    harv_core::datetime::parse_hours(s)
}
