use crate::error::Result;
use crate::network::NetworkService;
use color_eyre::owo_colors::OwoColorize;
use tracing::{info, warn};

pub struct MoveOutCommand {
    parameter_name: String,
    network_service: NetworkService,
}

impl MoveOutCommand {
    pub fn new(parameter_name: String) -> Self {
        Self {
            parameter_name,
            network_service: NetworkService::new(),
        }
    }

    pub async fn start_flow(mut self) -> Result<()> {
        let mut response = self.network_service.get_remote_config().await?;
        let remote_config = &mut response.data;

        if remote_config.parameter_groups.is_empty() {
            warn!("{}", "Parameters group list is empty!".yellow());
            return Ok(());
        }
        let group = remote_config
            .parameter_groups
            .iter_mut()
            .find_map(|(name, group)| {
                if group.parameters.contains_key(&self.parameter_name) {
                    Some((name, &mut group.parameters))
                } else {
                    None
                }
            });
        if group.is_none() {
            let message = format!(
                "Parameter with name {} was not found in any group!",
                self.parameter_name
            );
            warn!("{}", message.yellow());
            return Ok(());
        }
        let (name, params) = group.unwrap();
        info!(
            "Will move parameter {} out of group {}",
            self.parameter_name, name
        );
        let parameter = params.remove(&self.parameter_name).unwrap();
        remote_config
            .parameters
            .insert(std::mem::take(&mut self.parameter_name), parameter);
        self.network_service
            .update_remote_config(response.data, response.etag)
            .await?;
        Ok(())
    }
}
