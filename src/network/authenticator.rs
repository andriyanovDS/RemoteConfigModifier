use std::future::Future;
use std::pin::Pin;
use tracing::info;
use yup_oauth2::authenticator_delegate::{DefaultInstalledFlowDelegate, InstalledFlowDelegate};
use yup_oauth2::{AccessToken, InstalledFlowAuthenticator, InstalledFlowReturnMethod};

pub struct Authenticator {
    token: Option<AccessToken>,
}

impl Authenticator {
    pub fn new() -> Self {
        Self { token: None }
    }

    pub async fn get_access_token(&mut self) -> Result<&AccessToken, yup_oauth2::error::Error> {
        if self.token.is_some() {
            Ok(self.token.as_ref().unwrap())
        } else {
            let token = self.auth().await?;
            self.token = Some(token);
            Ok(self.token.as_ref().unwrap())
        }
    }

    async fn auth(&self) -> Result<AccessToken, yup_oauth2::error::Error> {
        let secret_bytes = include_bytes!("../../clientsecret.json");
        let secret = yup_oauth2::parse_application_secret(secret_bytes)?;
        let auth =
            InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
                .persist_tokens_to_disk("token_cache.json")
                .flow_delegate(Box::new(FlowDelegate))
                .build()
                .await?;
        let scopes = ["https://www.googleapis.com/auth/cloud-platform"];
        auth.token(&scopes).await
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
