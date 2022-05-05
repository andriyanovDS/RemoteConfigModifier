use crate::error::Result;
use crate::io::InputReader;
use crate::network::NetworkService;
use colored::Colorize;
use tracing::{error, info, warn};

pub struct DeleteParameterFlow<'a> {
    name: &'a str,
    network_service: NetworkService,
}

impl<'a> DeleteParameterFlow<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            network_service: NetworkService::new(),
        }
    }

    pub async fn start_flow(mut self) {
        if let Err(error) = self.delete_parameter().await {
            error!("{}", error.message.red());
        }
    }

    async fn delete_parameter(&mut self) -> Result<()> {
        let mut response = self.network_service.get_remote_config().await?;
        let remote_config = &mut response.data;
        let parameter = remote_config.parameters.remove(self.name);
        if parameter.is_none() {
            let message = format!("Parameter with name {} does not exists!", &self.name);
            warn!("{}", message.yellow());
            return Ok(());
        }
        let parameter = parameter.unwrap();
        let message = format!("{} parameter will be removed", &self.name);
        warn!("{}", message.yellow());
        warn!("{:#}", &parameter);
        if !InputReader::ask_confirmation("Confirm: y/n").await? {
            info!("Operation was canceled.");
            return Ok(());
        }
        self.network_service
            .update_remote_config(response.data, response.etag)
            .await?;
        Ok(())
    }
}
