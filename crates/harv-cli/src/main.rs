use clap::{CommandFactory, Parser};
use harv_cli::commands;
use harv_cli::{AliasCommand, Cli, Commands};

const GOODBYES: &[&str] = &[
    "Alright, catch you later!",
    "Time tracking dodged. Again.",
    "Your timesheet weeps.",
    "See you tomorrow. Maybe.",
    "Ctrl+C → Ctrl+V → nevermind.",
    "Until next time, time bandit.",
    "Coward. (I respect it.)",
    "Escaped the timesheet. For now.",
    "harv stop — oh wait, that was Ctrl+C.",
    "Your hours remain a mystery.",
];

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    harv_cli::setup_tracing();

    let cli = Cli::parse();

    let rt = tokio::runtime::Runtime::new()?;
    let result: color_eyre::eyre::Result<()> = rt.block_on(async {
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
    });

    match result {
        Ok(()) => Ok(()),
        Err(e)
            if matches!(
                e.downcast_ref::<inquire::InquireError>(),
                Some(inquire::InquireError::OperationInterrupted)
            ) =>
        {
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as usize;
            let msg = GOODBYES[seed % GOODBYES.len()];
            eprintln!("\n{msg}");
            std::process::exit(130)
        }
        Err(e) => Err(e),
    }
}
