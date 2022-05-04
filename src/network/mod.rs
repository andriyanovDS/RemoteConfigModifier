use crate::remote_config::RemoteConfig;
use authenticator::Authenticator;
use reqwest::{
    header::{ACCEPT_ENCODING, AUTHORIZATION, ETAG, IF_MATCH},
    Client, ClientBuilder,
};

mod authenticator;

const REMOTE_CONFIG_URL: &str =
    "https://firebaseremoteconfig.googleapis.com/v1/projects/774774183385/remoteConfig";

pub struct NetworkService {
    client: Client,
    authenticator: Authenticator,
}

pub struct ResponseWithEtag<T> {
    pub etag: String,
    pub data: T,
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

    pub async fn get_remote_config(
        &mut self,
    ) -> Result<ResponseWithEtag<RemoteConfig>, Box<dyn std::error::Error + Send + Sync>> {
        let access_token = self.authenticator.get_access_token().await?;
        let response = self
            .client
            .get(REMOTE_CONFIG_URL)
            .header(AUTHORIZATION, format!("Bearer {}", access_token.as_str()))
            .header(ACCEPT_ENCODING, "gzip, deflate, br")
            .send()
            .await?
            .error_for_status()?;
        let etag = response
            .headers()
            .get(ETAG)
            .expect("ETag header was not found in response headers.")
            .to_str()?
            .to_string();
        let bytes = response.bytes().await?;
        let remote_config = serde_json::from_slice::<RemoteConfig>(&bytes)?;
        Ok(ResponseWithEtag {
            etag,
            data: remote_config,
        })
    }

    pub async fn update_remote_config(
        &mut self,
        config: RemoteConfig,
        etag: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let access_token = self.authenticator.get_access_token().await?;
        let bytes = serde_json::to_string(&config)?.into_bytes();
        self.client
            .put(REMOTE_CONFIG_URL)
            .header(AUTHORIZATION, format!("Bearer {}", access_token.as_str()))
            .header(ACCEPT_ENCODING, "gzip, deflate, br")
            .header(IF_MATCH, etag)
            .body(bytes)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
