use colored::Colorize;
use remote_config_modifier::remote_config_builder::RemoteConfigBuilder;

#[tokio::main]
async fn main() {
    let parameter = RemoteConfigBuilder::start_flow().await;
    match parameter {
        Ok(parameter) => println!("parameter {:?}", parameter),
        Err(message) => eprintln!("{}", format!("{}", message.red()))
    }
}
