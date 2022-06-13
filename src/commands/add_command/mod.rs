use crate::commands::command::Command;
use crate::config::Project;
use crate::editor::Editor;
use crate::error::{Error, Result};
use crate::io::{self, InputReader};
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

#[derive(Copy, PartialEq, Clone)]
pub enum Action {
    Add,
    Update,
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
        action: Action,
    ) -> Result<()> {
        let mut projects_iter = projects.iter().enumerate();
        let main_project = projects_iter.next().as_ref().unwrap().1;

        let mut selected_conditions = response.data.selected_conditions_map(&parameter);
        let new_parameter = NewParameter {
            name: name.clone(),
            parameter: parameter.clone(),
        };
        self.add_parameter(new_parameter.clone(), response, main_project, action)
            .await?;
        if projects.len() == 1 {
            return Ok(());
        }
        let options = [
            "Add only default value",
            "Add default and conditional values",
            "Add custom values",
        ];
        let selected_option = io::request_select_item_in_list(
            "Select how to apply new parameter to other projects:",
            options.iter().copied(),
            None,
        );
        match selected_option {
            None => {}
            Some(0) => {
                self.add_parameter_with_default_value_to_projects(
                    new_parameter,
                    projects_iter,
                    action,
                )
                .await?;
            }
            Some(1) => {
                self.add_parameter_with_conditions_to_projects(
                    new_parameter,
                    &mut selected_conditions,
                    projects_iter,
                    action,
                )
                .await?;
            }
            Some(2) => {
                self.add_parameter_with_custom_values(
                    new_parameter,
                    &mut selected_conditions,
                    projects_iter,
                    action,
                )
                .await?;
            }
            Some(_) => panic!("Unexpected option was selected!"),
        }
        Ok(())
    }

    async fn add_parameter_with_conditions_to_projects(
        &mut self,
        new_parameter: NewParameter,
        selected_conditions: &mut HashMap<String, GenerationalCondition>,
        projects: impl Iterator<Item = (usize, &Project)>,
        action: Action,
    ) -> Result<()> {
        for (index, project) in projects {
            info!("Running for {} project", &project.name);
            let mut response = self.network_service.get_remote_config(project).await?;
            response
                .data
                .extend_conditions(selected_conditions, index + 1, &project.app_ids)?;
            self.add_parameter(new_parameter.clone(), response, project, action)
                .await?;
        }
        Ok(())
    }

    async fn add_parameter_with_default_value_to_projects(
        &mut self,
        mut new_parameter: NewParameter,
        projects: impl Iterator<Item = (usize, &Project)>,
        action: Action,
    ) -> Result<()> {
        new_parameter.parameter.conditional_values = HashMap::new();
        for (_, project) in projects {
            info!("Running for {} project", &project.name);
            let response = self.network_service.get_remote_config(project).await?;
            self.add_parameter(new_parameter.clone(), response, project, action)
                .await?;
        }
        Ok(())
    }

    async fn add_parameter_with_custom_values(
        &mut self,
        new_parameter: NewParameter,
        selected_conditions: &mut HashMap<String, GenerationalCondition>,
        projects: impl Iterator<Item = (usize, &Project)>,
        action: Action,
    ) -> Result<()> {
        for (index, project) in projects {
            info!("Running for {} project", &project.name);
            let mut conditions = Vec::new();
            let builder = ParameterBuilder::new_from_parameter(
                new_parameter.name.clone(),
                &new_parameter.parameter,
                &mut self.input_reader,
                &[],
                &mut conditions,
            );
            let selected_condition_names = new_parameter
                .parameter
                .conditional_values
                .iter()
                .map(|(name, _)| name.as_str());
            let (name, parameter) = builder.add_values(selected_condition_names)?;
            let mut response = self.network_service.get_remote_config(project).await?;
            response
                .data
                .extend_conditions(selected_conditions, index + 1, &project.app_ids)?;
            let new_parameter = NewParameter { name, parameter };
            self.add_parameter(new_parameter, response, project, action)
                .await?;
        }
        Ok(())
    }

    async fn add_parameter(
        &mut self,
        new_parameter: NewParameter,
        mut response: ResponseWithEtag<RemoteConfig>,
        project: &Project,
        action: Action,
    ) -> Result<()> {
        let remote_config = &mut response.data;
        let parameter_name = &new_parameter.name;
        let parameter = new_parameter.parameter;
        let map_with_parameter = remote_config.get_map_for_existing_parameter(parameter_name);
        match (map_with_parameter.as_ref(), action) {
            (Some(_), Action::Add) => {
                let message = format!(
                    "Parameter with name {} already exists! Do you want te replace it? [Y,n]",
                    parameter_name
                );
                let message = message.yellow().to_string();
                if !self.input_reader.ask_confirmation(&message) {
                    return Err(Error::new("Operation was canceled."));
                }
            }
            (Some(map), Action::Update) => {
                let parameter = map.get(parameter_name).unwrap();
                parameter.preview(parameter_name, "Previous parameter values", None);
            }
            _ => {}
        }

        let title = match action {
            Action::Add => "Parameter will be added",
            Action::Update => "Updated parameter values",
        };
        parameter.preview(parameter_name, title, None);
        if !self.input_reader.ask_confirmation("Confirm: [Y,n]") {
            return Err(Error::new("Operation was canceled."));
        }
        match map_with_parameter {
            Some(map) => {
                map.insert(new_parameter.name, parameter);
            }
            None => {
                remote_config
                    .parameters
                    .insert(new_parameter.name, parameter);
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
        let new_parameter = NewParameter { name, parameter };
        self.add_parameter(new_parameter, response, project, Action::Add)
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

        self.apply_parameter_to_projects(name, parameter, projects, response, Action::Add)
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

#[derive(Clone)]
struct NewParameter {
    name: String,
    parameter: Parameter,
}

struct GenerationalCondition {
    generation: usize,
    condition: Condition,
}
