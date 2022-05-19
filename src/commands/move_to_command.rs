use crate::commands::command::Command;
use crate::config::Project;
use crate::error::Result;
use crate::io::InputReader;
use crate::network::NetworkService;
use crate::remote_config::{Parameter, ParameterGroup, RemoteConfig};
use async_trait::async_trait;
use color_eyre::owo_colors::OwoColorize;
use colored::{ColoredString, Colorize};
use std::collections::HashMap;
use tracing::{error, info, warn};

pub struct MoveToCommand {
    parameter_name: String,
    group_name: Option<String>,
    network_service: NetworkService,
}

impl MoveToCommand {
    pub fn new(parameter_name: String, group_name: Option<String>) -> Self {
        Self {
            parameter_name,
            group_name,
            network_service: NetworkService::new(),
        }
    }

    async fn run(&mut self, project: &Project) -> Result<()> {
        info!("Running for {} project", &project.name);
        let mut response = self.network_service.get_remote_config(project).await?;
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

        let result = match self.group_name.as_ref() {
            None => self.unknown_group_flow(config, parameter).await,
            Some(name) => {
                let name = name.clone();
                self.known_group_flow(config, name, parameter).await
            }
        }?;
        if result.is_some() {
            self.network_service
                .update_remote_config(project, response.data, response.etag)
                .await?;
        }
        Ok(())
    }

    async fn unknown_group_flow(
        &mut self,
        config: &mut RemoteConfig,
        parameter: Parameter,
    ) -> Result<Option<()>> {
        let create_new_group_option = Some("Create new group");
        let groups_count = config.parameter_groups.len();
        let keys = config.parameter_groups.keys().map(|name| name.as_str());
        let label = "Select the group you want to move the parameter to:";
        let index =
            InputReader::request_select_item_in_list(label, keys, create_new_group_option, true)
                .await;

        if index.is_none() {
            return Ok(None);
        }

        let index = index.unwrap();
        if index == groups_count {
            self.add_parameter_to_new_group(config, parameter).await?;
            return Ok(Some(()));
        }
        let mut parameters = config.parameter_groups.values_mut().skip(index);
        let group = parameters.next().unwrap();
        group
            .parameters
            .insert(self.parameter_name.clone(), parameter);

        Ok(Some(()))
    }

    async fn add_parameter_to_new_group(
        &mut self,
        config: &mut RemoteConfig,
        parameter: Parameter,
    ) -> Result<()> {
        let (name, description) = MoveToCommand::create_new_group_name().await?;
        info!(
            "Parameter {} will be moved to {} group",
            self.parameter_name, &name
        );

        let mut parameters = HashMap::new();
        parameters.insert(self.parameter_name.clone(), parameter);
        config.parameter_groups.insert(
            name,
            ParameterGroup {
                description,
                parameters,
            },
        );
        Ok(())
    }

    async fn create_new_group_name() -> Result<(String, Option<String>)> {
        let provide_name_msg = "Enter group name: ".green();
        let name =
            InputReader::request_user_input_string::<ColoredString>(&provide_name_msg).await?;
        let provide_description_msg = "Enter group description (Optional):".green();
        let description =
            InputReader::request_user_input_string::<ColoredString>(&provide_description_msg)
                .await?;
        let description = if description.is_empty() {
            None
        } else {
            Some(description)
        };
        Ok((name, description))
    }

    async fn known_group_flow(
        &mut self,
        config: &mut RemoteConfig,
        group_name: String,
        parameter: Parameter,
    ) -> Result<Option<()>> {
        let group = config
            .parameter_groups
            .iter_mut()
            .find_map(|(name, group)| {
                if name == &group_name {
                    Some(group)
                } else {
                    None
                }
            });
        let parameter_name = self.parameter_name.clone();
        match group {
            None => {
                let message = format!(
                    "Group with name {} does not exist! Do you want to create it? [Y, n]",
                    &group_name
                );
                if !InputReader::ask_confirmation(&message.yellow()).await {
                    return Ok(None);
                } else {
                    let mut parameters = HashMap::new();
                    parameters.insert(parameter_name, parameter);
                    config.parameter_groups.insert(
                        group_name,
                        ParameterGroup {
                            description: None,
                            parameters,
                        },
                    );
                }
            }
            Some(group) => {
                group.parameters.insert(parameter_name, parameter);
            }
        }
        Ok(Some(()))
    }
}

#[async_trait]
impl Command for MoveToCommand {
    async fn run_for_single_project(mut self, project: &Project) -> Result<()> {
        self.run(project).await
    }

    async fn run_for_multiple_projects(mut self, projects: &[Project]) -> Result<()> {
        for project in projects {
            if let Err(error) = self.run(project).await {
                error!("{}", error.message.red());
            }
        }
        Ok(())
    }
}
