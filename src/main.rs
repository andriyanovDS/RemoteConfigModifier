use clap::Parser;
use color_eyre::{owo_colors::OwoColorize, Report};
use remote_config_modifier::commands::{
    AddCommand, CommandRunner, DeleteCommand, MoveOutCommand, MoveToCommand, ShowCommand,
    UpdateCommand,
};
use remote_config_modifier::{Cli, Command};
use tracing::error;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> Result<(), Report> {
    setup()?;
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Add(arguments) => {
            let command = AddCommand::new(arguments.name, arguments.description);
            CommandRunner::new(command).run(arguments.project).await
        }
        Command::Update(arguments) => {
            let command = UpdateCommand::new(arguments.name);
            CommandRunner::new(command).run(arguments.project).await
        }
        Command::Delete(arguments) => {
            let command = DeleteCommand::new(arguments.name);
            CommandRunner::new(command).run(arguments.project).await
        }
        Command::MoveTo(arguments) => {
            let command = MoveToCommand::new(arguments.parameter, arguments.group);
            CommandRunner::new(command).run(arguments.project).await
        }
        Command::MoveOut(arguments) => {
            let command = MoveOutCommand::new(arguments.parameter);
            CommandRunner::new(command).run(arguments.project).await
        }
        Command::Show(arguments) => CommandRunner::new(ShowCommand::new()).run(arguments).await,
    };
    if let Err(error) = result {
        error!("{}", error.message.red())
    }
    Ok(())
}

fn setup() -> Result<(), Report> {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    color_eyre::install()?;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info")
    }
    let format = fmt::format()
        .with_source_location(false)
        .with_file(false)
        .with_target(false)
        .with_timer(fmt::time::SystemTime::default())
        .compact();

    fmt::fmt()
        .event_format(format)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Ok(())
}
