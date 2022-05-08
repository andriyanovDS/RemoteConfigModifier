use crate::add_parameter_flow::parameter_builder::ParameterBuilder;
use crate::error::{Error, Result};
use crate::io::InputReader;
use crate::network::NetworkService;
use crate::remote_config::{Parameter, RemoteConfig};
use color_eyre::owo_colors::OwoColorize;
use std::collections::HashMap;
use tracing::{info, warn};

pub struct UpdateParameterFlow {
    name: String,
    network_service: NetworkService,
}

impl UpdateParameterFlow {
    pub fn new(name: String) -> Self {
        Self {
            name,
            network_service: NetworkService::new(),
        }
    }

    pub async fn start_flow(mut self) -> Result<()> {
        let mut response = self.network_service.get_remote_config().await?;
        let config = &mut response.data;
        let source_with_param = config.find_parameter_source(&self.name);
        let (name, parameter) = match source_with_param.as_ref() {
            None => {
                let message = format!("Could not find {} parameter!", &self.name);
                warn!("{}", message.yellow());
                return Ok(());
            }
            Some((_, parameter)) => {
                info!("{} parameter found", &self.name);
                info!("{:#}", &parameter);
                println!();

                let name = std::mem::take(&mut self.name);
                let description = parameter.description.clone();
                ParameterBuilder::start_flow(Some(name), description, &config.conditions)
                    .await
                    .map_err(|message| Error { message })?
            }
        };
        info!("{} parameter will be updated to:", &name);
        println!("{}", format!("{:#}", parameter).green());

        if !InputReader::ask_confirmation("Confirm: [Y,n]").await? {
            return Ok(());
        }

        let source = source_with_param.unwrap().0;
        let params = config.find_source_params(&source);
        params.insert(name, parameter);
        self.network_service
            .update_remote_config(response.data, response.etag)
            .await?;
        Ok(())
    }
}

impl RemoteConfig {
    fn find_parameter_source(&self, name: &str) -> Option<(ParameterSource, &Parameter)> {
        match (self.parameters.get(name), &self.parameter_groups) {
            (Some(param), _) => Some((ParameterSource::Root, param)),
            (_, Some(groups)) => groups.iter().find_map(|(name, group)| {
                group
                    .parameters
                    .as_ref()
                    .and_then(|params| params.get(name))
                    .map(|parameter| (ParameterSource::Group(name.clone()), parameter))
            }),
            _ => None,
        }
    }

    fn find_source_params(&mut self, source: &ParameterSource) -> &mut HashMap<String, Parameter> {
        match source {
            ParameterSource::Root => &mut self.parameters,
            ParameterSource::Group(name) => self
                .parameter_groups
                .as_mut()
                .unwrap()
                .iter_mut()
                .find_map(|(group_name, group)| {
                    if group_name == name {
                        group.parameters.as_mut()
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
