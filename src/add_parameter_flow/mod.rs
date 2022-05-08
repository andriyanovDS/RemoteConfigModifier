use crate::error::{Error, Result};
use crate::io::InputReader;
use crate::network::{NetworkService, ResponseWithEtag};
use crate::remote_config::{Parameter, RemoteConfig};
use color_eyre::owo_colors::OwoColorize;
use parameter_builder::ParameterBuilder;
use tracing::{error, info};

mod parameter_builder;

pub struct AddParameterFlow {
    name: Option<String>,
    description: Option<String>,
    network_service: NetworkService,
}
impl Default for AddParameterFlow {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            network_service: NetworkService::new(),
        }
    }
}

impl AddParameterFlow {
    pub fn new(name: Option<String>, description: Option<String>) -> Self {
        Self {
            name,
            description,
            network_service: NetworkService::new(),
        }
    }

    pub async fn start_flow(mut self) {
        match self
            .network_service
            .get_remote_config()
            .await
            .map_err(Error::from)
        {
            Ok(response) => {
                let future = ParameterBuilder::start_flow(
                    &response.data,
                    self.name.take(),
                    self.description.take(),
                );
                let result = match future.await {
                    Ok((name, parameter)) => self.add_parameter(name, parameter, response).await,
                    Err(message) => Err(Error { message }),
                };
                if let Err(error) = result {
                    error!("{}", error.message.red());
                }
            }
            Err(error) => error!("{}", error.message.red()),
        }
    }

    async fn add_parameter(
        &mut self,
        name: String,
        parameter: Parameter,
        mut response: ResponseWithEtag<RemoteConfig>,
    ) -> Result<()> {
        let remote_config = &mut response.data;
        let map_with_parameter = remote_config.get_map_for_existing_parameter(&name);
        if map_with_parameter.is_some() {
            let message = format!(
                "Parameter with name {} already exists! Do you want te replace it? [Y,n]",
                name
            );
            let message = message.yellow().to_string();
            if !InputReader::ask_confirmation(&message).await? {
                return Ok(());
            }
        }
        info!("New parameter will be added:");
        println!("{}", format!("{name}: {:#}", parameter).green());

        if !InputReader::ask_confirmation("Confirm: [Y,n]").await? {
            return Ok(());
        }
        match map_with_parameter {
            Some(map) => {
                map.insert(name, parameter);
            }
            None => {
                remote_config.parameters.insert(name, parameter);
            }
        }
        self.network_service
            .update_remote_config(response.data, response.etag)
            .await?;
        Ok(())
    }
}
