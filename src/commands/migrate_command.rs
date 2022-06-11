use crate::config::Project;
use crate::editor::Editor;
use crate::error::{Error, Result};
use crate::io::InputReader;
use crate::network::NetworkService;
use crate::remote_config::{Parameter, ParameterGroup, RemoteConfig};
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use term_table::row::Row;
use tracing::{debug, info};

pub struct MigrateCommand<'a, NS: NetworkService, E: Editor> {
    source_project: &'a Project,
    destinations: Vec<&'a Project>,
    network_service: NS,
    input_reader: InputReader<E>,
}

struct NewParameter<'a> {
    group: Option<NewParameterGroup<'a>>,
    name: String,
    parameter: Parameter,
}

struct NewParameterGroup<'a> {
    name: &'a str,
    description: Option<&'a str>,
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
        source_project_name: String,
        projects: &'a [Project],
        network_service: NS,
        input_reader: InputReader<E>,
    ) -> Result<MigrateCommand<'a, NS, E>> {
        let source_project = projects
            .iter()
            .find(|project| project.name == source_project_name)
            .ok_or_else(|| Error {
                message: format!(
                    "Source project {source_project_name} was not found in configuration file"
                ),
            })?;

        let projects = projects
            .iter()
            .filter(|project| project.name != source_project_name)
            .collect();

        Ok(Self {
            source_project,
            destinations: projects,
            network_service,
            input_reader,
        })
    }

    pub fn new_for_selected_projects(
        source_project_name: String,
        destinations: Vec<String>,
        projects: &'a [Project],
        network_service: NS,
        input_reader: InputReader<E>,
    ) -> Result<MigrateCommand<'a, NS, E>> {
        let source_project = projects
            .iter()
            .find(|project| project.name == source_project_name)
            .ok_or_else(|| Error {
                message: format!(
                    "Source project {source_project_name} was not found in configuration file"
                ),
            })?;
        let destination_names = destinations.into_iter().collect::<HashSet<_>>();
        let destinations = projects
            .iter()
            .filter(|project| {
                destination_names.contains(&project.name) && project.name != source_project_name
            })
            .collect::<Vec<_>>();
        Ok(Self {
            source_project,
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
            .get_remote_config(self.source_project)
            .await?
            .data;

        for project in self.destinations {
            info!("Running for {} project", &project.name);
            let mut response = self.network_service.get_remote_config(project).await?;
            let destination = &mut response.data;
            let existing_names = destination.existing_parameter_names();
            let new_parameters = source.find_new_parameters(&existing_names);

            if new_parameters.is_empty() {
                println!("No new parameters was found.");
                continue;
            }
            destination.render(&project.name, &new_parameters);

            for parameter in new_parameters {
                let group = match parameter.group {
                    None => {
                        destination
                            .parameters
                            .insert(parameter.name, parameter.parameter);
                        continue;
                    }
                    Some(group) => destination
                        .parameter_groups
                        .get_mut(group.name)
                        .ok_or(group),
                };
                match group {
                    Ok(group) => {
                        group.parameters.insert(parameter.name, parameter.parameter);
                    }
                    Err(group) => {
                        let mut parameters = HashMap::<String, Parameter>::new();
                        parameters.insert(parameter.name, parameter.parameter);
                        destination.parameter_groups.insert(
                            group.name.to_string(),
                            ParameterGroup {
                                description: group.description.map(|name| name.to_string()),
                                parameters,
                            },
                        );
                    }
                };
            }

            if !self.input_reader.ask_confirmation("Confirm: [Y,n]") {
                continue;
            }
            self.network_service
                .update_remote_config(project, response.data, response.etag)
                .await?;
        }
        Ok(())
    }
}

impl RemoteConfig {
    fn existing_parameter_names(&self) -> HashSet<&str> {
        self.parameter_groups
            .values()
            .flat_map(|group| group.parameters.keys())
            .chain(self.parameters.keys())
            .fold(HashSet::new(), |mut names, name| {
                names.insert(name);
                names
            })
    }

    fn find_new_parameters<'b>(&self, existing_names: &'b HashSet<&str>) -> Vec<NewParameter> {
        let new_root_parameters = self.parameters.iter().filter_map(|(name, parameter)| {
            if existing_names.contains(name.as_str()) {
                None
            } else {
                Some(NewParameter {
                    group: None,
                    name: name.clone(),
                    parameter: parameter.clone_without_coniditional_values(),
                })
            }
        });

        let new_group_parameters = self
            .parameter_groups
            .iter()
            .flat_map(|(group_name, group)| {
                group.parameters.iter().filter_map(|(name, parameter)| {
                    if existing_names.contains(name.as_str()) {
                        None
                    } else {
                        let parameter_group = NewParameterGroup {
                            name: group_name.as_str(),
                            description: group.description.as_deref(),
                        };
                        Some(NewParameter {
                            group: Some(parameter_group),
                            name: name.clone(),
                            parameter: parameter.clone_without_coniditional_values(),
                        })
                    }
                })
            });

        new_root_parameters
            .chain(new_group_parameters)
            .collect::<Vec<_>>()
    }

    fn render(&self, project_name: &str, new_parameters: &Vec<NewParameter>) {
        let new_parameter_rows = new_parameters.iter().flat_map(|param| param.make_rows());

        let mut table = self.build_table(project_name);
        let rows = &mut table.rows;
        rows.reserve(new_parameters.len());
        let mut condition_rows = rows
            .split_off(rows.len() - self.conditions.len());
        rows.extend(new_parameter_rows);
        rows.append(&mut condition_rows);

        println!("Updated configuration:");
        println!("{}", table.render());
    }
}

impl Parameter {
    fn clone_without_coniditional_values(&self) -> Self {
        Parameter {
            default_value: self.default_value.clone(),
            description: self.description.clone(),
            value_type: self.value_type,
            conditional_values: HashMap::new(),
        }
    }
}

impl<'a> NewParameter<'a> {
    fn make_rows(&self) -> Vec<Row> {
        let group_name = self.group.as_ref().map(|v| v.name);
        self.parameter.make_row(self.name.green(), group_name)
    }
}
