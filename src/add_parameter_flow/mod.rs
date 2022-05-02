use color_eyre::owo_colors::OwoColorize;
use crate::error::{Error, Result};
use crate::network::NetworkService;
use crate::remote_config::{Parameter};
use crate::io::{InputReader};
use remote_config_builder::RemoteConfigBuilder;

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

    pub async fn start_flow(&mut self) {
        let result = match RemoteConfigBuilder::start_flow().await {
            Ok((name, parameter)) => {
                self.add_parameter(name, parameter).await
            },
            Err(message) => Err(Error { message })
        };
        if let Err(error) = result {
            eprintln!("{}", &error.message.red());
        }
    }

    async fn add_parameter(&mut self, name: String, parameter: Parameter) -> Result<()> {
        println!();
        println!("Downloading remote config...");
        let mut remote_config = self.network_service.get_remote_config().await?;
        if remote_config.parameters.contains_key(&name) {
            let message = format!("Parameter with name {} already exists! Do you want te replace it? (y,n)", name)
                .yellow()
                .to_string();
            if !InputReader::ask_confirmation(&message).await? {
                return Ok(());
            }
        }
        println!();
        println!("-----------------------");
        println!();
        println!("New parameter will be added:");
        println!("{}", format!("{name}: {:#}", parameter).green());

        if !InputReader::ask_confirmation("Confirm: y/n").await? {
            return Ok(());
        }
        remote_config.parameters.insert(name, parameter);
        Ok(())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self {
            message: format!("{}", error)
        }
    }
}
