use super::add_command::parameter_builder::ParameterBuilder;
use crate::error::{Error, Result};
use crate::io::InputReader;
use crate::network::NetworkService;
use crate::remote_config::{Parameter, RemoteConfig};
use crate::projects::Project;
use color_eyre::owo_colors::OwoColorize;
use std::collections::HashMap;
use tracing::warn;

pub struct UpdateCommand {
    name: String,
    network_service: NetworkService,
}

impl UpdateCommand {
    pub fn new(name: String) -> Self {
        Self {
            name,
            network_service: NetworkService::new(),
        }
    }

    pub async fn start_flow(mut self) -> Result<()> {
        let project = Project::stub();
        let mut response = self.network_service.get_remote_config(&project).await?;
        let config = &mut response.data;
        let source_with_param = config.find_parameter_source(&self.name);
        let (name, parameter) = match source_with_param.as_ref() {
            None => {
                let message = format!("Could not find {} parameter!", &self.name);
                warn!("{}", message.yellow());
                return Ok(());
            }
            Some((source, parameter)) => {
                println!();
                match source {
                    ParameterSource::Root => parameter.preview(&self.name, "Parameter found", None),
                    ParameterSource::Group(name) => {
                        parameter.preview(&self.name, "Parameter found", Some(&name))
                    }
                }
                let name = std::mem::take(&mut self.name);
                let description = parameter.description.clone();
                ParameterBuilder::start_flow(Some(name), description, &config.conditions)
                    .await
                    .map_err(|message| Error { message })?
            }
        };

        let source = source_with_param.unwrap().0;
        match &source {
            ParameterSource::Root => parameter.preview(&name, "Parameter will be updated", None),
            ParameterSource::Group(group_name) => {
                parameter.preview(&name, "Parameter will be updated", Some(group_name))
            }
        }

        if !InputReader::ask_confirmation("Confirm: [Y,n]").await? {
            return Ok(());
        }

        let params = config.find_source_params(&source);
        params.insert(name, parameter);
        self.network_service
            .update_remote_config(&project, response.data, response.etag)
            .await?;
        Ok(())
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
