use color_eyre::owo_colors::OwoColorize;
use crate::error::{Error, Result};
use crate::io::InputReader;
use crate::network::NetworkService;
use crate::remote_config::Parameter;
use colored::Colorize;
use remote_config_builder::RemoteConfigBuilder;
use tracing::{error, info};

mod remote_config_builder;

pub struct AddParameterFlow {
    network_service: NetworkService,
}
impl Default for AddParameterFlow {
    fn default() -> Self {
        Self::new()
    }
}

impl AddParameterFlow {
    pub fn new() -> Self {
        Self {
            network_service: NetworkService::new(),
        }
    }

    pub async fn start_flow(mut self) {
        let result = match RemoteConfigBuilder::start_flow().await {
            Ok((name, parameter)) => self.add_parameter(name, parameter).await,
            Err(message) => Err(Error { message }),
        };
        if let Err(error) = result {
            error!("{}", error.message.red());
        }
    }

    async fn add_parameter(&mut self, name: String, parameter: Parameter) -> Result<()> {
        println!();
        info!("Downloading remote config...");
        let mut response = self.network_service.get_remote_config().await?;
        let remote_config = &mut response.data;
        if remote_config.parameters.contains_key(&name) {
            let message = format!(
                "Parameter with name {} already exists! Do you want te replace it? (y,n)",
                name
            );
            let message = message.yellow().to_string();
            if !InputReader::ask_confirmation(&message).await? {
                return Ok(());
            }
        }
        info!("New parameter will be added:");
        info!("{}", format!("{name}: {:#}", parameter).green());

        if !InputReader::ask_confirmation("Confirm: y/n").await? {
            return Ok(());
        }
        remote_config.parameters.insert(name, parameter);
        info!("Uploading updated remote config...");
        self.network_service
            .update_remote_config(response.data, response.etag)
            .await?;
        info!("Uploading succeeded.");
        Ok(())
    }
}
