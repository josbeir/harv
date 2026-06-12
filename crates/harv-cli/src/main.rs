use clap::{CommandFactory, Parser};
use harv_cli::commands;
use harv_cli::{AliasCommand, Cli, Commands};

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    harv_cli::setup_tracing();

    harv_core::init_locale(None);

    let cli = Cli::parse();

    let requires_auth = !matches!(
        &cli.command,
        Some(Commands::Connect | Commands::Completion(_))
    );
    if requires_auth && !harv_sdk::HarvConfig::path().exists() {
        eprintln!("{}", harv_core::t("err-not-authenticated"));
        std::process::exit(1);
    }

    let result: color_eyre::eyre::Result<()> = match cli.command {
        Some(cmd) => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                // Apply locale override from config
                if let Ok(config) = harv_sdk::HarvConfig::load().await
                    && let Some(locale) = &config.locale
                {
                    harv_core::init_locale(Some(locale));
                }
                match cmd {
                    Commands::Connect => commands::connect::run().await?,
                    Commands::Disconnect => commands::disconnect::run().await?,
                    Commands::Config(args) => commands::config_cmd::execute(&args).await?,
                    Commands::Track(args) => {
                        commands::track::run(
                            args.project_id,
                            args.task_id,
                            args.hours,
                            args.notes,
                            args.editor,
                            args.date,
                            args.refresh,
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
                            args.refresh,
                        )
                        .await?
                    }
                    Commands::Stop(args) => {
                        commands::stop::run(args.notes, args.overwrite, args.editor).await?
                    }
                    Commands::Note(args) => {
                        commands::note::run(args.notes, args.overwrite, args.editor).await?
                    }
                    Commands::Edit(args) => {
                        commands::edit::run(
                            args.entry_id,
                            args.project_id,
                            args.task_id,
                            args.hours,
                            args.notes,
                            args.editor,
                            args.overwrite,
                            args.date,
                            args.refresh,
                        )
                        .await?
                    }
                    Commands::Status => commands::status::run(&cli.output).await?,
                    Commands::Whoami => commands::whoami::run(&cli.output).await?,
                    Commands::Projects(args) => {
                        commands::projects::run(args.search, args.refresh, &cli.output).await?
                    }
                    Commands::Tasks(args) => {
                        commands::tasks::run(args.project_id, &cli.output).await?
                    }
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
            })
        }
        None => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async { harv_tui::run().await })
        }
    };

    match result {
        Ok(()) => Ok(()),
        Err(e)
            if matches!(
                e.downcast_ref::<inquire::InquireError>(),
                Some(
                    inquire::InquireError::OperationInterrupted
                        | inquire::InquireError::OperationCanceled,
                )
            ) =>
        {
            std::process::exit(130)
        }
        Err(e) => Err(e),
    }
}
