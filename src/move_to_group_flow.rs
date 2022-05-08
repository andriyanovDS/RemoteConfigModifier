use crate::error::Result;
use crate::io::{InputReader, InputString};
use crate::network::NetworkService;
use crate::remote_config::{Parameter, ParameterGroup, RemoteConfig};
use crate::MoveTo;
use color_eyre::owo_colors::OwoColorize;
use colored::{ColoredString, Colorize};
use std::collections::HashMap;
use std::mem;
use tracing::{error, info, warn};

pub struct MoveToGroupFlow {
    parameter_name: String,
    group_name: Option<String>,
    network_service: NetworkService,
}

impl MoveToGroupFlow {
    pub fn new(arguments: MoveTo) -> Self {
        Self {
            parameter_name: arguments.parameter,
            group_name: arguments.group,
            network_service: NetworkService::new(),
        }
    }

    pub async fn start_flow(mut self) {
        if let Err(error) = self.move_parameter().await {
            error!("{}", error.message.red());
        }
    }

    async fn move_parameter(&mut self) -> Result<()> {
        let mut response = self.network_service.get_remote_config().await?;
        let config = &mut response.data;
        let map_with_parameter = config.get_map_for_existing_parameter(&self.parameter_name);

        if map_with_parameter.is_none() {
            let message = format!(
                "Parameter with name {} does not exist!",
                self.parameter_name
            );
            warn!("{}", message.yellow());
            return Ok(());
        }
        let parameter = map_with_parameter
            .unwrap()
            .remove(&self.parameter_name)
            .unwrap();

        let result = match self.group_name.take() {
            None => self.unknown_group_flow(config, parameter).await,
            Some(name) => self.known_group_flow(config, name, parameter).await,
        }?;
        if result.is_some() {
            self.network_service
                .update_remote_config(response.data, response.etag)
                .await?;
        }
        Ok(())
    }

    async fn unknown_group_flow(
        &mut self,
        config: &mut RemoteConfig,
        parameter: Parameter,
    ) -> Result<Option<()>> {
        println!();
        info!(
            "{}",
            "Select the group you want to move the parameter to:".green()
        );
        println!();
        let create_new_group_option = Some("Create new group");
        let groups_count = config
            .parameter_groups
            .as_ref()
            .map(|groups| groups.len())
            .unwrap_or(0);
        let index = match config.parameter_groups.as_ref() {
            Some(groups) => {
                let keys = groups.keys().map(|name| name.as_str());
                InputReader::request_select_item_in_list(keys, create_new_group_option).await
            }
            None => {
                InputReader::request_select_item_in_list(
                    std::iter::empty(),
                    create_new_group_option,
                )
                .await
            }
        }?;
        if index.is_none() {
            return Ok(None);
        }

        let index = index.unwrap();
        if index == groups_count {
            self.add_parameter_to_new_group(config, parameter).await?;
            return Ok(Some(()));
        }
        let mut parameters = config
            .parameter_groups
            .as_mut()
            .unwrap()
            .values_mut()
            .skip(index);
        let group = parameters.next().unwrap();
        RemoteConfig::insert_param_to_group(group, mem::take(&mut self.parameter_name), parameter);
        Ok(Some(()))
    }

    async fn add_parameter_to_new_group(
        &mut self,
        config: &mut RemoteConfig,
        parameter: Parameter,
    ) -> Result<()> {
        let (name, description) = MoveToGroupFlow::create_new_group_name().await?;
        info!(
            "Parameter {} will be moved to {} group",
            self.parameter_name, &name
        );

        let mut parameters = HashMap::new();
        parameters.insert(mem::take(&mut self.parameter_name), parameter);
        config.insert_group(name, description, parameters);
        Ok(())
    }

    async fn create_new_group_name() -> Result<(String, Option<String>)> {
        let provide_name_msg = "Enter group name: ".green();
        let name = InputReader::request_user_input::<InputString, ColoredString>(&provide_name_msg)
            .await?;
        let provide_description_msg = "Enter group description (Optional):".green();
        let description =
            InputReader::request_user_input::<InputString, ColoredString>(&provide_description_msg)
                .await?;
        let description = if description.0.is_empty() {
            None
        } else {
            Some(description.0)
        };
        Ok((name.0, description))
    }

    async fn known_group_flow(
        &mut self,
        config: &mut RemoteConfig,
        group_name: String,
        parameter: Parameter,
    ) -> Result<Option<()>> {
        let group = config
            .parameter_groups
            .as_mut()
            .unwrap()
            .iter_mut()
            .find_map(|(name, group)| {
                if name == &group_name {
                    Some(group)
                } else {
                    None
                }
            });
        let parameter_name = mem::take(&mut self.parameter_name);
        match group {
            None => {
                let message = format!(
                    "Group with name {} does not exist! Do you want to create it? [Y, n]",
                    &group_name
                );
                if !InputReader::ask_confirmation(&message.yellow()).await? {
                    return Ok(None);
                } else {
                    let mut parameters = HashMap::new();
                    parameters.insert(parameter_name, parameter);
                    config.insert_group(group_name, None, parameters);
                }
            }
            Some(group) => {
                RemoteConfig::insert_param_to_group(group, parameter_name, parameter);
            }
        }
        Ok(Some(()))
    }
}

impl RemoteConfig {
    fn insert_group(
        &mut self,
        name: String,
        description: Option<String>,
        parameters: HashMap<String, Parameter>,
    ) {
        match self.parameter_groups.as_mut() {
            Some(groups) => {
                groups.insert(
                    name,
                    ParameterGroup {
                        description,
                        parameters: Some(parameters),
                    },
                );
            }
            None => {
                let mut map = HashMap::new();
                map.insert(
                    name,
                    ParameterGroup {
                        description,
                        parameters: Some(parameters),
                    },
                );
                self.parameter_groups = Some(map);
            }
        }
    }

    fn insert_param_to_group(
        group: &mut ParameterGroup,
        parameter_name: String,
        parameter: Parameter,
    ) {
        match group.parameters.as_mut() {
            None => {
                let mut map = HashMap::new();
                map.insert(parameter_name, parameter);
                group.parameters = Some(map);
            }
            Some(params) => {
                params.insert(parameter_name, parameter);
            }
        }
    }
}
