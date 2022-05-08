use clap::Parser;
use color_eyre::Report;
use remote_config_modifier::add_parameter_flow::AddParameterFlow;
use remote_config_modifier::delete_parameter_flow::DeleteParameterFlow;
use remote_config_modifier::move_to_group_flow::MoveToGroupFlow;
use remote_config_modifier::{Cli, Command};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt;

#[tokio::main]
async fn main() -> Result<(), Report> {
    setup()?;
    let cli = Cli::parse();

    match cli.command {
        Command::Add(arguments) => {
            let add_parameter_flow = AddParameterFlow::new(arguments.name, arguments.description);
            add_parameter_flow.start_flow().await;
        }
        Command::Update { name: _ } => {
            todo!()
        }
        Command::Delete { name } => {
            let delete_parameter_flow = DeleteParameterFlow::new(&name);
            delete_parameter_flow.start_flow().await;
        }
        Command::MoveTo(arguments) => {
            MoveToGroupFlow::new(arguments).start_flow().await;
        }
        Command::MoveOut { parameter_name: _ } => {}
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
