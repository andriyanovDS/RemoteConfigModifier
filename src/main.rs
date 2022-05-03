use tracing_subscriber::filter::{EnvFilter};
use tracing_subscriber::fmt;
use color_eyre::Report;
use remote_config_modifier::add_parameter_flow::AddParameterFlow;

#[tokio::main]
async fn main() {
    setup();

    let mut add_parameter_flow = AddParameterFlow::new();
    add_parameter_flow.start_flow().await;
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
