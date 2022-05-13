use super::add_command::parameter_builder::ParameterBuilder;
use crate::commands::command::Command;
use crate::commands::AddCommand;
use crate::config::Project;
use crate::error::{Error, Result};
use crate::io::InputReader;
use crate::network::{NetworkService, ResponseWithEtag};
use crate::remote_config::{Parameter, RemoteConfig};
use async_trait::async_trait;
use color_eyre::owo_colors::OwoColorize;
use std::collections::HashMap;
use tracing::{info, warn};

pub struct UpdateCommand {
    name: String,
    network_service: Option<NetworkService>,
}

impl UpdateCommand {
    pub fn new(name: String) -> Self {
        Self {
            name,
            network_service: Some(NetworkService::new()),
        }
    }

    fn find_parameter_source<'a>(
        &self,
        config: &'a RemoteConfig,
    ) -> Option<(ParameterSource, &'a Parameter)> {
        match config.find_parameter_source(&self.name) {
            None => {
                let message = format!("Could not find {} parameter!", &self.name);
                warn!("{}", message.yellow());
                None
            }
            Some((source, parameter)) => {
                println!();
                match &source {
                    ParameterSource::Root => parameter.preview(&self.name, "Parameter found", None),
                    ParameterSource::Group(name) => {
                        parameter.preview(&self.name, "Parameter found", Some(name))
                    }
                }
                Some((source, parameter))
            }
        }
    }

    async fn update_parameter(
        &mut self,
        name: String,
        parameter: Parameter,
        mut response: ResponseWithEtag<RemoteConfig>,
        source: &ParameterSource,
        project: &Project,
    ) -> Result<()> {
        match source {
            ParameterSource::Root => {
                parameter.preview(&self.name, "Parameter will be updated", None)
            }
            ParameterSource::Group(group_name) => {
                parameter.preview(&self.name, "Parameter will be updated", Some(group_name))
            }
        }
        if !InputReader::ask_confirmation("Confirm: [Y,n]").await {
            return Ok(());
        }
        let params = response.data.find_source_params(source);
        params.insert(name, parameter);
        self.network_service
            .as_mut()
            .unwrap()
            .update_remote_config(project, response.data, response.etag)
            .await
            .map_err(Error::from)
    }
}

#[async_trait]
impl Command for UpdateCommand {
    async fn run_for_single_project(mut self, project: &Project) -> Result<()> {
        info!("Running for {} project", &project.name);
        let network_service = self.network_service.as_mut().unwrap();
        let mut response = network_service.get_remote_config(project).await?;
        let config = &mut response.data;

        let source = self.find_parameter_source(config);
        if source.is_none() {
            return Ok(());
        }
        let (source, parameter) = source.unwrap();
        let description = parameter.description.clone();
        let (name, parameter) = ParameterBuilder::start_flow(
            Some(std::mem::take(&mut self.name)),
            description,
            &config.conditions,
        )
        .await;
        self.update_parameter(name, parameter, response, &source, project)
            .await
    }

    async fn run_for_multiple_projects(mut self, projects: &[Project]) -> Result<()> {
        assert!(!projects.is_empty(), "Projects must not be empty");
        let main_project = projects.first().unwrap();
        let network_service = self.network_service.as_mut().unwrap();
        info!("Running for {} project", &main_project.name);
        let response = network_service.get_remote_config(main_project).await?;
        let source = self.find_parameter_source(&response.data);

        if source.is_none() {
            return Ok(());
        }
        let (_, parameter) = source.unwrap();
        let description = parameter.description.clone();
        let (name, parameter) = ParameterBuilder::start_flow(
            Some(std::mem::take(&mut self.name)),
            description,
            &response.data.conditions,
        )
        .await;

        let mut add_command =
            AddCommand::new_with_network_service(None, None, self.network_service.take().unwrap());
        add_command
            .apply_parameter_to_projects(name, parameter, projects, response, true)
            .await
    }
}

impl RemoteConfig {
    fn find_parameter_source(&self, name: &str) -> Option<(ParameterSource, &Parameter)> {
        match self.parameters.get(name) {
            Some(param) => Some((ParameterSource::Root, param)),
            None => {
                self.parameter_groups
                    .iter()
                    .find_map(|(group_name, group)| {
                        group.parameters.get(name).map(|parameter| {
                            (ParameterSource::Group(group_name.clone()), parameter)
                        })
                    })
            }
        }
    }

    fn find_source_params(&mut self, source: &ParameterSource) -> &mut HashMap<String, Parameter> {
        match source {
            ParameterSource::Root => &mut self.parameters,
            ParameterSource::Group(name) => self
                .parameter_groups
                .iter_mut()
                .find_map(|(group_name, group)| {
                    if group_name == name {
                        Some(&mut group.parameters)
                    } else {
                        None
                    }
                })
                .expect("Parameters must exist"),
        }
    }
}

enum ParameterSource {
    Root,
    Group(String),
}
