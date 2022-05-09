use crate::error::{Error, Result};
use crate::io::InputReader;
use crate::network::{NetworkService, ResponseWithEtag};
use crate::remote_config::{Parameter, RemoteConfig};
use crate::Add;
use crate::projects::Project;
use color_eyre::owo_colors::OwoColorize;
use parameter_builder::ParameterBuilder;

pub mod parameter_builder;

pub struct AddCommand {
    name: Option<String>,
    description: Option<String>,
    network_service: NetworkService,
}
impl Default for AddCommand {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            network_service: NetworkService::new(),
        }
    }
}

impl AddCommand {
    pub fn new(arguments: Add) -> Self {
        Self {
            name: arguments.name,
            description: arguments.description,
            network_service: NetworkService::new(),
        }
    }

    pub async fn start_flow(mut self) -> Result<()> {
        let project = Project::stub();
        let response = self.network_service.get_remote_config(&project).await?;
        let future = ParameterBuilder::start_flow(
            self.name.take(),
            self.description.take(),
            &response.data.conditions,
        );
        match future.await {
            Ok((name, parameter)) => self.add_parameter(name, parameter, response, &project).await,
            Err(message) => Err(Error { message }),
        }
    }

    async fn add_parameter(
        &mut self,
        name: String,
        parameter: Parameter,
        mut response: ResponseWithEtag<RemoteConfig>,
        project: &Project,
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

        parameter.preview(&name, "Parameter will be added", None);
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
            .update_remote_config(&project, response.data, response.etag)
            .await?;
        Ok(())
    }
}
