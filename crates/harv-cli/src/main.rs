pub mod commands;
pub(crate) mod output;
pub(crate) mod prompts;
pub(crate) mod spinner;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

/// Harv — A CLI for Harvest time tracking.
#[derive(Parser)]
#[command(name = "harv", version, about, long_about = None)]
pub struct Cli {
    /// Global output format
    #[arg(short, long, global = true, default_value = "table")]
    pub output: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
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

    /// Generate shell completion script
    Completion(CompletionArgs),

    /// List your project assignments
    Projects(ProjectsArgs),

    /// List tasks for a project
    Tasks(TasksArgs),

    /// Manage project/task aliases
    #[command(subcommand)]
    Alias(AliasCommand),
}

/// Arguments for the track command
#[derive(clap::Args, Clone, Debug)]
pub struct TrackArgs {
    /// Project ID
    #[arg(short = 'p', long)]
    pub project_id: Option<u64>,

    /// Task ID
    #[arg(short = 't', long)]
    pub task_id: Option<u64>,

    /// Hours to log (omit or set to 0 to start a running timer)
    #[arg(short = 'H', long)]
    pub hours: Option<f64>,

    /// Notes for the time entry
    #[arg(short = 'n', long)]
    pub notes: Option<String>,

    /// Open $EDITOR for notes
    #[arg(short = 'e', long)]
    pub editor: bool,

    /// Spent date (YYYY-MM-DD), defaults to today
    #[arg(short = 'd', long)]
    pub date: Option<String>,

    /// Alias for project+task pair
    pub alias: Option<String>,
}

/// Arguments for the start command
#[derive(clap::Args, Clone, Debug)]
pub struct StartArgs {
    /// Project ID
    #[arg(short = 'p', long)]
    pub project_id: Option<u64>,

    /// Task ID
    #[arg(short = 't', long)]
    pub task_id: Option<u64>,

    /// Notes for the timer
    #[arg(short = 'n', long)]
    pub notes: Option<String>,

    /// Open $EDITOR for notes
    #[arg(short = 'e', long)]
    pub editor: bool,

    /// Spent date (YYYY-MM-DD), defaults to today
    #[arg(short = 'd', long)]
    pub date: Option<String>,

    /// Alias for project+task pair
    pub alias: Option<String>,
}

/// Arguments for the stop command
#[derive(clap::Args, Clone, Debug)]
pub struct StopArgs {
    /// Notes to append
    #[arg(short = 'n', long)]
    pub notes: Option<String>,

    /// Overwrite existing notes instead of appending
    #[arg(long)]
    pub overwrite: bool,

    /// Open $EDITOR for notes
    #[arg(short = 'e', long)]
    pub editor: bool,
}

/// Arguments for the log command
#[derive(clap::Args, Clone, Debug)]
pub struct LogArgs {
    /// Hours to log (decimal)
    pub hours: f64,

    /// Project ID
    #[arg(short = 'p', long)]
    pub project_id: Option<u64>,

    /// Task ID
    #[arg(short = 't', long)]
    pub task_id: Option<u64>,

    /// Notes for the time entry
    #[arg(short = 'n', long)]
    pub notes: Option<String>,

    /// Open $EDITOR for notes
    #[arg(short = 'e', long)]
    pub editor: bool,

    /// Spent date (YYYY-MM-DD), defaults to today
    #[arg(short = 'd', long)]
    pub date: Option<String>,

    /// Alias for project+task pair
    pub alias: Option<String>,
}

/// Arguments for the note command
#[derive(clap::Args, Clone, Debug)]
pub struct NoteArgs {
    /// Notes to append
    #[arg(short = 'n', long)]
    pub notes: Option<String>,

    /// Overwrite existing notes instead of appending
    #[arg(long)]
    pub overwrite: bool,

    /// Open $EDITOR for notes
    #[arg(short = 'e', long)]
    pub editor: bool,
}

/// Arguments for the projects command
#[derive(clap::Args, Clone, Debug)]
pub struct ProjectsArgs {
    /// Filter projects by name or client
    #[arg(short = 's', long)]
    pub search: Option<String>,
}

/// Arguments for the tasks command
#[derive(clap::Args, Clone, Debug)]
pub struct TasksArgs {
    /// Project ID to list tasks for
    pub project_id: u64,
}

/// Alias subcommands
#[derive(Subcommand, Clone, Debug)]
pub enum AliasCommand {
    /// Create a new alias
    Create {
        /// Alias name
        name: String,
    },
    /// List all aliases
    List,
    /// Delete an alias
    Delete {
        /// Alias name
        name: String,
    },
}

/// Arguments for the completion command
#[derive(clap::Args, Clone, Debug)]
pub struct CompletionArgs {
    /// Shell to generate completions for
    pub shell: Shell,
}

pub(crate) fn setup_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("harv=warn")),
        )
        .init();
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    setup_tracing();

    let cli = Cli::parse();

    match cli.command {
        Commands::Connect => commands::connect::run().await?,
        Commands::Config => commands::config_cmd::run().await?,
        Commands::Track(args) => {
            commands::track::run(
                args.project_id,
                args.task_id,
                args.hours,
                args.notes,
                args.editor,
                args.date,
                args.alias,
            )
            .await?
        }
        Commands::Start(args) => {
            commands::start::run(
                args.alias,
                args.project_id,
                args.task_id,
                args.notes,
                args.editor,
                args.date,
            )
            .await?
        }
        Commands::Stop(args) => {
            commands::stop::run(args.notes, args.overwrite, args.editor).await?
        }
        Commands::Log(args) => {
            commands::log::run(
                args.hours,
                args.alias,
                args.project_id,
                args.task_id,
                args.notes,
                args.editor,
                args.date,
            )
            .await?
        }
        Commands::Note(args) => {
            commands::note::run(args.notes, args.overwrite, args.editor).await?
        }
        Commands::Status => commands::status::run(&cli.output).await?,
        Commands::Projects(args) => commands::projects::run(args.search, &cli.output).await?,
        Commands::Tasks(args) => commands::tasks::run(args.project_id, &cli.output).await?,
        Commands::Alias(cmd) => match cmd {
            AliasCommand::Create { name } => commands::alias::create(name).await?,
            AliasCommand::List => commands::alias::list(&cli.output).await?,
            AliasCommand::Delete { name } => commands::alias::delete(name).await?,
        },
        Commands::Completion(args) => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            clap_complete::generate(args.shell, &mut cmd, name, &mut std::io::stdout());
        }
    }

    Ok(())
}
