use crate::remote_config::RemoteConfig;
use authenticator::Authenticator;
use reqwest::{
    header::{ACCEPT_ENCODING, AUTHORIZATION, ETAG, IF_MATCH},
    Client, ClientBuilder,
};
use spinners::{Spinner, Spinners};
use std::future::Future;
use tracing::debug;
use crate::projects::Project;

mod authenticator;

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
        project: &Project,
    ) -> Result<ResponseWithEtag<RemoteConfig>, Box<dyn std::error::Error + Send + Sync>> {
        NetworkService::perform_with_spinner(
            "Downloading remote config...",
            "Downloading completed successfully",
            async move {
                let access_token = self.authenticator.get_access_token().await?;
                let response = self
                    .client
                    .get(project.url())
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
                debug!("Received remote config: {:?}", &remote_config);
                Ok(ResponseWithEtag {
                    etag,
                    data: remote_config,
                })
            },
        )
        .await
    }

    pub async fn update_remote_config(
        &mut self,
        project: &Project,
        config: RemoteConfig,
        etag: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        debug!("Remote config to upload: {:#?}", &config);
        NetworkService::perform_with_spinner(
            "Uploading remote config...",
            "Uploading completed successfully",
            async move {
                let access_token = self.authenticator.get_access_token().await?;
                let bytes = serde_json::to_string(&config)?.into_bytes();
                self.client
                    .put(project.url())
                    .header(AUTHORIZATION, format!("Bearer {}", access_token.as_str()))
                    .header(ACCEPT_ENCODING, "gzip, deflate, br")
                    .header(IF_MATCH, etag)
                    .body(bytes)
                    .send()
                    .await?
                    .error_for_status()?;
                Ok(())
            },
        )
        .await
    }

    async fn perform_with_spinner<F, R>(
        start_message: &str,
        completion_message: &str,
        future: F,
    ) -> Result<R, Box<dyn std::error::Error + Send + Sync>>
    where
        F: Future<Output = Result<R, Box<dyn std::error::Error + Send + Sync>>>,
    {
        let mut spinner = Spinner::new(Spinners::Dots12, start_message.into());
        let result = future.await;
        if result.is_ok() {
            print!("\r");
            spinner.stop_with_message(completion_message.into());
            println!();
        } else {
            println!();
        }
        result
    }
}
