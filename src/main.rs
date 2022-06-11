use clap::Parser;
use color_eyre::{owo_colors::OwoColorize, Report};
use rcm::cli::{Cli, Command};
use rcm::commands::{
    AddCommand, CommandRunner, ConfigCommand, ConfigFile, DeleteCommand, MigrateCommand,
    MoveOutCommand, MoveToCommand, ShowCommand, UpdateCommand,
};
use rcm::io::InputReader;
use rcm::network::NetworkWorker;
use rustyline::Editor;
use std::ffi::OsStr;
use std::path::Path;
use tracing::error;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> Result<(), Report> {
    setup()?;
    let cli = Cli::parse();
    let app_name = app_name();
    let command_runner = CommandRunner::new(app_name.clone());
    let network_worker = NetworkWorker::new(app_name.clone());
    let input_reader = InputReader::new(Editor::new());
    let result = match cli.command {
        Command::Add(arguments) => {
            let command = AddCommand::new(
                arguments.name,
                arguments.description,
                network_worker,
                input_reader,
            );
            command_runner.run(command, arguments.project).await
        }
        Command::Update(arguments) => {
            let command = UpdateCommand::new(arguments.name, network_worker, input_reader);
            command_runner.run(command, arguments.project).await
        }
        Command::Delete(arguments) => {
            let command = DeleteCommand::new(arguments.name, network_worker, input_reader);
            command_runner.run(command, arguments.project).await
        }
        Command::MoveTo(arguments) => {
            let command = MoveToCommand::new(
                arguments.name,
                arguments.group,
                network_worker,
                input_reader,
            );
            command_runner.run(command, arguments.project).await
        }
        Command::MoveOut(arguments) => {
            let command = MoveOutCommand::new(arguments.name, network_worker);
            command_runner.run(command, arguments.project).await
        }
        Command::Show(arguments) => {
            command_runner
                .run(ShowCommand::new(network_worker), arguments)
                .await
        }
        Command::Config(arguments) => ConfigCommand::new(app_name, arguments).run(),
        Command::Migrate(arguments) => {
            let config_file = ConfigFile::new(app_name);
            let projects = config_file.load()?.projects;
            let command = match arguments.projects {
                Some(arg_projects) => MigrateCommand::new_for_selected_projects(
                    arguments.source,
                    arg_projects,
                    &projects,
                    network_worker,
                    input_reader,
                ),
                None => MigrateCommand::new_for_all_projects(
                    arguments.source,
                    &projects,
                    network_worker,
                    input_reader,
                ),
            }?;
            command.run().await
        }
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

fn app_name() -> String {
    std::env::args()
        .next()
        .as_ref()
        .map(Path::new)
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .map(String::from)
        .unwrap()
}
