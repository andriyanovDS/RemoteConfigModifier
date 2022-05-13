use crate::commands::command::Command;
use crate::config::Project;
use crate::error::Result;
use crate::network::NetworkService;
use crate::remote_config::RemoteConfig;
use async_trait::async_trait;
use color_eyre::owo_colors::OwoColorize;
use term_table::{
    row::Row,
    table_cell::{Alignment, TableCell},
    Table, TableStyle,
};
use tracing::{error, info};

pub struct ShowCommand {
    network_service: NetworkService,
}

impl ShowCommand {
    pub fn new() -> Self {
        Self {
            network_service: NetworkService::new(),
        }
    }

    fn build_table<'a, 'b>(config: &'a mut RemoteConfig, project_name: &'b str) -> Table<'a> {
        let mut table = Table::new();
        table.max_column_width = 40;
        table.style = TableStyle::simple();

        let title = format!("{} parameters", project_name);
        table.add_row(ShowCommand::make_title_row(title));
        config
            .parameters
            .iter()
            .flat_map(|(name, parameter)| parameter.make_row(name, None))
            .for_each(|row| table.add_row(row));

        config
            .parameter_groups
            .iter()
            .flat_map(|(group_name, group)| {
                group
                    .parameters
                    .iter()
                    .flat_map(|(name, parameter)| parameter.make_row(name, Some(group_name)))
            })
            .for_each(|row| table.add_row(row));

        if !config.conditions.is_empty() {
            table.add_row(ShowCommand::make_title_row("Conditions".to_string()));
            config
                .conditions
                .iter_mut()
                .map(|condition| condition.make_row())
                .for_each(|row| table.add_row(row))
        }
        table
    }

    fn make_title_row(title: String) -> Row<'static> {
        Row::new(vec![TableCell::new_with_alignment(
            title,
            5,
            Alignment::Center,
        )])
    }
}

#[async_trait]
impl Command for ShowCommand {
    async fn run_for_single_project(mut self, project: &Project) -> Result<()> {
        info!("Running for {} project", &project.name);
        let mut response = self.network_service.get_remote_config(project).await?;
        let table = ShowCommand::build_table(&mut response.data, &project.name);
        println!("{}", table.render());
        Ok(())
    }

    async fn run_for_multiple_projects(mut self, projects: &[Project]) -> Result<()> {
        for project in projects {
            info!("Running for {} project", &project.name);
            match self.network_service.get_remote_config(project).await {
                Err(error) => {
                    error!("{}", error.to_string().red());
                }
                Ok(mut response) => {
                    let table = ShowCommand::build_table(&mut response.data, &project.name);
                    println!("{}", table.render());
                }
            }
        }
        Ok(())
    }
}

impl Default for ShowCommand {
    fn default() -> Self {
        Self::new()
    }
}
