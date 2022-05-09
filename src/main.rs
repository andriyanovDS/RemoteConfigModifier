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
        Command::Add(arguments) => AddCommand::new(arguments).start_flow().await,
        Command::Update { name } => UpdateCommand::new(name).start_flow().await,
        Command::Delete(arguments) => {
            CommandRunner::new(DeleteCommand::new(arguments.name))
                .run(arguments.project)
                .await
        }
        Command::MoveTo(arguments) => MoveToCommand::new(arguments).start_flow().await,
        Command::MoveOut { parameter } => MoveOutCommand::new(parameter).start_flow().await,
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
