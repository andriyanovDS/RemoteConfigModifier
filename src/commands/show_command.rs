use crate::commands::command::Command;
use crate::config::Project;
use crate::error::Result;
use crate::network::NetworkService;
use async_trait::async_trait;
use color_eyre::owo_colors::OwoColorize;
use tracing::{error, info};

pub struct ShowCommand<NS: NetworkService> {
    network_service: NS,
}

impl<NS: NetworkService> ShowCommand<NS> {
    pub fn new(network_service: NS) -> Self {
        Self { network_service }
    }
}

#[async_trait]
impl<NS: NetworkService + Send> Command for ShowCommand<NS> {
    async fn run_for_single_project(mut self, project: &Project) -> Result<()> {
        info!("Running for {} project", &project.name);
        let response = self.network_service.get_remote_config(project).await?;
        let table = response.data.build_table(&project.name);
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
                Ok(response) => {
                    let table = response.data.build_table(&project.name);
                    println!("{}", table.render());
                }
            }
        }
        Ok(())
    }
}
