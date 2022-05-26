use directories_next::ProjectDirs;
use std::fs;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use tracing::{debug, info, warn};
use yup_oauth2::authenticator_delegate::{DefaultInstalledFlowDelegate, InstalledFlowDelegate};
use yup_oauth2::{AccessToken, InstalledFlowAuthenticator, InstalledFlowReturnMethod};

const TOKEN_CACHE_FILE_NAME: &str = "token_cache.json";

pub struct Authenticator {
    token: Option<AccessToken>,
    app_name: String,
}

impl Authenticator {
    pub fn new(app_name: String) -> Self {
        Self {
            token: None,
            app_name,
        }
    }

    pub async fn get_access_token(
        &mut self,
    ) -> Result<&AccessToken, Box<dyn std::error::Error + Send + Sync>> {
        if self.token.is_some() {
            Ok(self.token.as_ref().unwrap())
        } else {
            let token = self.auth().await?;
            self.token = Some(token);
            Ok(self.token.as_ref().unwrap())
        }
    }

    async fn auth(&self) -> Result<AccessToken, Box<dyn std::error::Error + Send + Sync>> {
        let secret_bytes = include_bytes!("../../clientsecret.json");
        let secret = yup_oauth2::parse_application_secret(secret_bytes)?;
        let token_file_path = self
            .token_file_path()
            .ok_or_else(|| crate::error::Error::new("Failed to store auth token."))?;
        debug!("Auth token will be saved to {token_file_path:?}");
        let auth =
            InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
                .persist_tokens_to_disk(token_file_path)
                .flow_delegate(Box::new(FlowDelegate))
                .build()
                .await?;
        let scopes = ["https://www.googleapis.com/auth/cloud-platform"];
        auth.token(&scopes).await.map_err(Into::into)
    }

    fn token_file_path(&self) -> Option<PathBuf> {
        let directories = ProjectDirs::from("com", "", &self.app_name)?;
        let cache_dir = directories.cache_dir();
        if !cache_dir.exists() {
            if let Err(error) = fs::create_dir_all(cache_dir) {
                warn!("Failed to create cache directory: {:?}", error);
                return None;
            }
        }
        let path = [cache_dir.to_str()?, TOKEN_CACHE_FILE_NAME]
            .iter()
            .collect();
        Some(path)
    }
}

struct FlowDelegate;

async fn open_url_in_browser(url: &str, need_code: bool) -> Result<String, String> {
    if webbrowser::open(url).is_ok() {
        info!("Url was opened in the browser");
    }
    let delegate = DefaultInstalledFlowDelegate;
    delegate.present_user_url(url, need_code).await
}

impl InstalledFlowDelegate for FlowDelegate {
    fn present_user_url<'a>(
        &'a self,
        url: &'a str,
        need_code: bool,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + 'a>> {
        Box::pin(open_url_in_browser(url, need_code))
    }
}
