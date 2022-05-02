use crate::remote_config::RemoteConfig;
use authenticator::Authenticator;
use hyper::{client::connect::HttpConnector, Body, Client, Request};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use std::error::Error;

mod authenticator;

pub struct NetworkService {
    client: Client<HttpsConnector<HttpConnector>>,
    authenticator: Authenticator,
}

impl Default for NetworkService {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkService {
    pub fn new() -> Self {
        let https = HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .build();
        Self {
            client: Client::builder().build::<_, Body>(https),
            authenticator: Authenticator::new(),
        }
    }

    pub async fn get_remote_config(&mut self) -> Result<RemoteConfig, Box<dyn Error + Send + Sync>> {
        let access_token = self.authenticator.get_access_token().await?;
        let request = Request::get(
            "https://firebaseremoteconfig.googleapis.com/v1/projects/774774183385/remoteConfig",
        )
        .header("Authorization", format!("Bearer {}", access_token.as_str()))
        .body(Body::empty())
        .unwrap();
        let response = self.client.request(request).await?;
        let body = response.into_body();
        let bytes = hyper::body::to_bytes(body).await?;
        let remote_config: RemoteConfig = serde_json::from_slice(&bytes)?;
        Ok(remote_config)
    }
}
