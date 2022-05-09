use crate::commands::command::Command;
use crate::error::Result;
use crate::io::InputReader;
use crate::network::NetworkService;
use crate::projects::Project;
use async_trait::async_trait;
use color_eyre::owo_colors::OwoColorize;
use tracing::{error, info, warn};

pub struct DeleteCommand {
    name: String,
    network_service: NetworkService,
}

impl<'a> DeleteCommand {
    pub fn new(name: String) -> Self {
        Self {
            name,
            network_service: NetworkService::new(),
        }
    }

    async fn run(&mut self, project: &Project) -> Result<()> {
        info!("Running for {} project", &project.name);
        let mut response = self.network_service.get_remote_config(&project).await?;
        let remote_config = &mut response.data;
        let map_with_parameter = remote_config.get_map_for_existing_parameter(&self.name);

        if map_with_parameter.is_none() {
            let message = format!("Parameter with name {} does not exists!", &self.name);
            warn!("{}", message.yellow());
            return Ok(());
        }
        let parameter = map_with_parameter.unwrap().remove(&self.name).unwrap();

        parameter.preview(&self.name, "Parameter will be deleted", None);
        if !InputReader::ask_confirmation("Confirm: [Y,n]").await? {
            info!("Operation was canceled.");
            return Ok(());
        }
        self.network_service
            .update_remote_config(&project, response.data, response.etag)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl Command for DeleteCommand {
    async fn run_for_single_project(mut self, project: &Project) -> Result<()> {
        self.run(project).await
    }

    async fn run_for_multiple_projects(mut self, projects: &[Project]) -> Result<()> {
        for project in projects {
            if let Err(error) = self.run(project).await {
                error!("{}", error.red());
            }
        }
        Ok(())
    }
}
