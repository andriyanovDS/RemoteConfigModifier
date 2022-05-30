use crate::commands::command::Command;
use crate::config::Project;
use crate::editor::Editor;
use crate::error::{Error, Result};
use crate::io::InputReader;
use crate::network::{NetworkService, ResponseWithEtag};
use crate::remote_config::{Condition, Parameter, RemoteConfig};
use async_trait::async_trait;
use color_eyre::owo_colors::OwoColorize;
use parameter_builder::ParameterBuilder;
use std::collections::HashMap;
use tracing::info;

mod expression_builder;
mod operator;
pub mod parameter_builder;

pub struct AddCommand<NS: NetworkService, E: Editor> {
    name: Option<String>,
    description: Option<String>,
    network_service: NS,
    input_reader: InputReader<E>,
}

impl<NS: NetworkService, E: Editor> AddCommand<NS, E> {
    pub fn new(
        name: Option<String>,
        description: Option<String>,
        network_service: NS,
        input_reader: InputReader<E>,
    ) -> Self {
        Self {
            name,
            description,
            network_service,
            input_reader,
        }
    }

    pub async fn apply_parameter_to_projects(
        &mut self,
        name: String,
        parameter: Parameter,
        projects: &[Project],
        response: ResponseWithEtag<RemoteConfig>,
        is_update: bool,
    ) -> Result<()> {
        let mut projects_iter = projects.iter().enumerate();
        let main_project = projects_iter.next().as_ref().unwrap().1;

        let mut selected_conditions = response.data.selected_conditions_map(&parameter);
        self.add_parameter(
            name.clone(),
            parameter.clone(),
            response,
            main_project,
            is_update,
        )
        .await?;

        let message = "Do you want to add same values to all projects? [Y,n]";
        if self.input_reader.ask_confirmation(message) {
            for (index, project) in projects_iter {
                info!("Running for {} project", &project.name);
                let mut response = self.network_service.get_remote_config(project).await?;
                response.data.extend_conditions(
                    &mut selected_conditions,
                    index + 1,
                    &project.app_ids,
                )?;
                self.add_parameter(
                    name.clone(),
                    parameter.clone(),
                    response,
                    project,
                    is_update,
                )
                .await?;
            }
        } else {
            for (index, project) in projects_iter {
                info!("Running for {} project", &project.name);
                let mut conditions = Vec::new();
                let builder = ParameterBuilder::new_from_parameter(
                    name.clone(),
                    &parameter,
                    &mut self.input_reader,
                    &[],
                    &mut conditions,
                );
                let selected_condition_names = parameter
                    .conditional_values
                    .iter()
                    .map(|(name, _)| name.as_str());
                let (name, parameter) = builder.add_values(selected_condition_names)?;
                let mut response = self.network_service.get_remote_config(project).await?;
                response.data.extend_conditions(
                    &mut selected_conditions,
                    index + 1,
                    &project.app_ids,
                )?;
                self.add_parameter(name, parameter, response, project, is_update)
                    .await?;
            }
        }
        Ok(())
    }

    async fn add_parameter(
        &mut self,
        name: String,
        parameter: Parameter,
        mut response: ResponseWithEtag<RemoteConfig>,
        project: &Project,
        is_update: bool,
    ) -> Result<()> {
        let remote_config = &mut response.data;
        let map_with_parameter = remote_config.get_map_for_existing_parameter(&name);
        match (map_with_parameter.as_ref(), is_update) {
            (Some(_), false) => {
                let message = format!(
                    "Parameter with name {} already exists! Do you want te replace it? [Y,n]",
                    name
                );
                let message = message.yellow().to_string();
                if !self.input_reader.ask_confirmation(&message) {
                    return Err(Error::new("Operation was canceled."));
                }
            }
            (Some(map), true) => {
                let parameter = map.get(&name).unwrap();
                parameter.preview(&name, "Previous parameter values", None);
            }
            _ => {}
        }

        let title = if is_update {
            "Updated parameter values"
        } else {
            "Parameter will be added"
        };
        parameter.preview(&name, title, None);
        if !self.input_reader.ask_confirmation("Confirm: [Y,n]") {
            return Err(Error::new("Operation was canceled."));
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
            .update_remote_config(project, response.data, response.etag)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl<NS: NetworkService + Send, E: Editor + Send> Command for AddCommand<NS, E> {
    async fn run_for_single_project(mut self, project: &Project) -> Result<()> {
        info!("Running for {} project", &project.name);
        let mut response = self.network_service.get_remote_config(project).await?;
        let (name, parameter) = ParameterBuilder::start_flow(
            self.name.take(),
            self.description.take(),
            &mut self.input_reader,
            &project.app_ids,
            &mut response.data.conditions,
        );
        self.add_parameter(name, parameter, response, project, false)
            .await
    }

    async fn run_for_multiple_projects(mut self, projects: &[Project]) -> Result<()> {
        assert!(!projects.is_empty(), "Projects must not be empty");

        let main_project = projects.first().unwrap();
        info!("Running for {} project", &main_project.name);
        let mut response = self.network_service.get_remote_config(main_project).await?;

        let (name, parameter) = ParameterBuilder::start_flow(
            self.name.take(),
            self.description.take(),
            &mut self.input_reader,
            &main_project.app_ids,
            &mut response.data.conditions,
        );

        self.apply_parameter_to_projects(name, parameter, projects, response, false)
            .await
    }
}

impl RemoteConfig {
    fn selected_conditions_map(
        &self,
        parameter: &Parameter,
    ) -> HashMap<String, GenerationalCondition> {
        let mut selected_conditions = HashMap::with_capacity(parameter.conditional_values.len());
        for condition in self.conditions.iter() {
            if parameter.conditional_values.contains_key(&condition.name) {
                let gen_condition = GenerationalCondition {
                    generation: 0,
                    condition: condition.clone(),
                };
                selected_conditions.insert(condition.name.clone(), gen_condition);
            }
        }
        selected_conditions
    }

    fn extend_conditions(
        &mut self,
        new_conditions: &mut HashMap<String, GenerationalCondition>,
        generation: usize,
        app_ids: &[String],
    ) -> Result<()> {
        for condition in self.conditions.iter() {
            if let Some(gen_condition) = new_conditions.get_mut(&condition.name) {
                gen_condition.generation = generation;
            }
        }
        for gen_condition in new_conditions
            .values()
            .filter(|v| v.generation < generation)
        {
            let mut condition = gen_condition.condition.clone();
            expression_builder::replace_app_id(&mut condition.expression, app_ids)?;
            self.conditions.push(condition);
        }
        Ok(())
    }
}

struct GenerationalCondition {
    generation: usize,
    condition: Condition,
}
