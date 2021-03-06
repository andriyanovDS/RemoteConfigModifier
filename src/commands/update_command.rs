use super::add_command::parameter_builder::ParameterBuilder;
use super::add_command::{Action, AddCommand};
use crate::commands::command::Command;
use crate::config::Project;
use crate::editor::Editor;
use crate::error::{Error, Result};
use crate::io::InputReader;
use crate::network::{NetworkService, ResponseWithEtag};
use crate::remote_config::{Parameter, RemoteConfig};
use async_trait::async_trait;
use color_eyre::owo_colors::OwoColorize;
use std::collections::HashMap;
use tracing::{info, warn};

pub struct UpdateCommand<NS: NetworkService, E: Editor> {
    name: String,
    network_service: Option<NS>,
    input_reader: Option<InputReader<E>>,
}

impl<NS: NetworkService, E: Editor> UpdateCommand<NS, E> {
    pub fn new(name: String, network_service: NS, input_reader: InputReader<E>) -> Self {
        Self {
            name,
            network_service: Some(network_service),
            input_reader: Some(input_reader),
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
        if !self
            .input_reader
            .as_mut()
            .unwrap()
            .ask_confirmation("Confirm: [Y,n]")
        {
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
impl<NS: NetworkService + Send, E: Editor + Send> Command for UpdateCommand<NS, E> {
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
            self.input_reader.as_mut().unwrap(),
            &project.app_ids,
            &mut config.conditions,
        );
        self.update_parameter(name, parameter, response, &source, project)
            .await
    }

    async fn run_for_multiple_projects(mut self, projects: &[Project]) -> Result<()> {
        assert!(!projects.is_empty(), "Projects must not be empty");
        let main_project = projects.first().unwrap();
        let network_service = self.network_service.as_mut().unwrap();
        info!("Running for {} project", &main_project.name);
        let mut response = network_service.get_remote_config(main_project).await?;
        let source = self.find_parameter_source(&response.data);

        if source.is_none() {
            return Ok(());
        }
        let (_, parameter) = source.unwrap();
        let description = parameter.description.clone();
        let (name, parameter) = ParameterBuilder::start_flow(
            Some(std::mem::take(&mut self.name)),
            description,
            self.input_reader.as_mut().unwrap(),
            &main_project.app_ids,
            &mut response.data.conditions,
        );

        let mut add_command = AddCommand::new(
            None,
            None,
            self.network_service.take().unwrap(),
            self.input_reader.take().unwrap(),
        );
        add_command
            .apply_parameter_to_projects(name, parameter, projects, response, Action::Update)
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
