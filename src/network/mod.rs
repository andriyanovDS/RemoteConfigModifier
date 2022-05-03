use authenticator::Authenticator;
use reqwest::{Client, ClientBuilder, header::{AUTHORIZATION, ETAG, ACCEPT_ENCODING}};
use tracing::info;
use crate::error::Error;
use crate::remote_config::RemoteConfig;

mod authenticator;

pub struct NetworkService {
    client: Client,
    authenticator: Authenticator,
}

impl Default for NetworkService {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkService {
    pub fn new() -> Self {
        Self {
            client: ClientBuilder::new().gzip(true).build().unwrap(),
            authenticator: Authenticator::new(),
        }
    }

    pub async fn get_remote_config(&mut self) -> Result<RemoteConfig, Box<dyn std::error::Error + Send + Sync>> {
        let access_token = self.authenticator.get_access_token().await?;
        let response = self.client.get("https://firebaseremoteconfig.googleapis.com/v1/projects/774774183385/remoteConfig")
            .header(AUTHORIZATION, format!("Bearer {}", access_token.as_str()))
            .header(ACCEPT_ENCODING, "gzip, deflate, br")
            .send()
            .await?
            .error_for_status()?;
        let etag = response.headers().get(ETAG).expect("ETag header was not found in response.");
        let bytes = response.bytes().await?;
        let remote_config: RemoteConfig = serde_json::from_slice(&bytes)?;
        Ok(remote_config)
    }
}
