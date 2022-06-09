use crate::config::Project;
use crate::editor::Editor;
use crate::error::{Error, Result};
use crate::io::InputReader;
use crate::network::NetworkService;
use crate::remote_config::{Parameter, ParameterGroup, RemoteConfig};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, warn};

pub struct MigrateCommand<'a, NS: NetworkService, E: Editor> {
    source_project: &'a Project,
    destinations: Vec<&'a Project>,
    network_service: NS,
    input_reader: InputReader<E>,
}

struct NewParameter<'a> {
    group: Option<(&'a str, Option<&'a str>)>,
    name: String,
    parameter: Parameter,
}

impl<'a, NS: NetworkService, E: Editor> MigrateCommand<'a, NS, E> {
    pub fn new(
        source_project: &'a Project,
        destinations: Vec<&'a Project>,
        network_service: NS,
        input_reader: InputReader<E>,
    ) -> Self {
        Self {
            source_project,
            destinations,
            network_service,
            input_reader,
        }
    }

    pub fn new_for_all_projects(
        source_project: String,
        projects: &'a Vec<Project>,
        network_service: NS,
        input_reader: InputReader<E>,
    ) -> Result<MigrateCommand<'a, NS, E>> {
        let source = projects
            .iter()
            .find(|project| project.name == source_project);

        if source.is_none() {
            return Err(Error {
                message: format!(
                    "Source project {} was not found in configuration file",
                    source_project
                ),
            });
        };
        let projects = projects
            .iter()
            .filter(|project| project.name != source_project)
            .collect();

        Ok(Self {
            source_project: source.unwrap(),
            destinations: projects,
            network_service,
            input_reader,
        })
    }

    pub fn new_from_projects(
        source_project: String,
        destinations: Vec<String>,
        projects: &'a Vec<Project>,
        network_service: NS,
        input_reader: InputReader<E>,
    ) -> Result<MigrateCommand<'a, NS, E>> {
        let source = projects
            .iter()
            .find(|project| project.name == source_project);

        if source.is_none() {
            return Err(Error {
                message: format!(
                    "Source project {} was not found in configuration file",
                    source_project
                ),
            });
        };
        let source = source.unwrap();
        let destinations = destinations
            .into_iter()
            .filter_map(|destination| {
                match projects.iter().find(|project| destination == project.name) {
                    None => {
                        warn!("Destination project {destination} was not found in configuration file!");
                        None
                    }
                    Some(project) => Some(project)
                }
            })
            .collect::<Vec<_>>();

        Ok(Self {
            source_project: source,
            destinations,
            network_service,
            input_reader,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        if self.destinations.is_empty() {
            debug!("Destinations list is empty. Migration will not be performed.");
            return Ok(());
        }
        let source = self
            .network_service
            .get_remote_config(&self.source_project)
            .await?
            .data;

        for project in self.destinations {
            info!("Running for {} project", &project.name);
            let mut response = self.network_service.get_remote_config(&project).await?;
            let destination = &mut response.data;
            let existing_names = destination.existing_parameter_names();
            let new_parameters = source.find_new_parameters(&existing_names);

            if new_parameters.is_empty() {
                println!("No new parameters was found.");
                continue;
            }

            for parameter in new_parameters {
                let group = match parameter.group {
                    None => {
                        destination
                            .parameters
                            .insert(parameter.name, parameter.parameter);
                        continue;
                    }
                    Some((group_name, group_description)) => destination
                        .parameter_groups
                        .get_mut(group_name)
                        .ok_or((group_name, group_description)),
                };
                match group {
                    Ok(group) => {
                        group.parameters.insert(parameter.name, parameter.parameter);
                    }
                    Err((group_name, group_description)) => {
                        let mut parameters = HashMap::<String, Parameter>::new();
                        parameters.insert(parameter.name, parameter.parameter);
                        destination.parameter_groups.insert(
                            group_name.to_string(),
                            ParameterGroup {
                                description: group_description.map(|name| name.to_string()),
                                parameters,
                            },
                        );
                    }
                };
            }
            let table = destination.build_table(&project.name);
            println!("Updated configuration:");
            println!("{}", table.render());
            if !self.input_reader.ask_confirmation("Confirm: [Y,n]") {
                continue;
            }
            self.network_service
                .update_remote_config(&project, response.data, response.etag)
                .await?;
        }
        Ok(())
    }
}

impl RemoteConfig {
    fn existing_parameter_names(&self) -> HashSet<&str> {
        let mut names = HashSet::<&str>::new();
        self.parameter_groups
            .values()
            .flat_map(|group| group.parameters.keys())
            .chain(self.parameters.keys())
            .for_each(|name| {
                names.insert(name);
            });
        names
    }

    fn find_new_parameters<'b>(&self, existing_names: &'b HashSet<&str>) -> Vec<NewParameter> {
        let mut new_parameters = Vec::<NewParameter>::new();
        self.parameters
            .iter()
            .filter(|(name, _)| !existing_names.contains(name.as_str()))
            .for_each(|(name, parameter)| {
                new_parameters.push(NewParameter {
                    group: None,
                    name: name.clone(),
                    parameter: Parameter {
                        default_value: parameter.default_value.clone(),
                        description: parameter.description.clone(),
                        value_type: parameter.value_type,
                        conditional_values: HashMap::new(),
                    },
                });
            });
        self.parameter_groups
            .iter()
            .for_each(|(group_name, group)| {
                group
                    .parameters
                    .iter()
                    .filter(|(name, _)| !existing_names.contains(name.as_str()))
                    .for_each(|(name, parameter)| {
                        let description = parameter.description.as_ref().map(|v| v.as_str());
                        new_parameters.push(NewParameter {
                            group: Some((group_name.as_str(), description)),
                            name: name.clone(),
                            parameter: Parameter {
                                default_value: parameter.default_value.clone(),
                                description: parameter.description.clone(),
                                value_type: parameter.value_type,
                                conditional_values: HashMap::new(),
                            },
                        });
                    });
            });
        new_parameters
    }
}
