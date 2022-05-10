use crate::error::{Error, Result};
use serde::Deserialize;
use tracing::debug;

pub struct Projects;

#[derive(Deserialize)]
pub struct Project {
    pub name: String,
    project_number: String,
}

impl Projects {
    pub async fn load_projects() -> Result<Vec<Project>> {
        let result = tokio::fs::read("projects.json").await;
        match result {
            Ok(buffer) => serde_json::from_slice::<Vec<Project>>(&buffer).map_err(|error| Error {
                message: error.to_string(),
            }),
            Err(error) => {
                debug!("{}", error.to_string());
                Err(Error::new(
                    "Could not locate project.json! Put it in the project root.",
                ))
            }
        }
    }
}

impl Project {
    pub fn url(&self) -> String {
        format!(
            "https://firebaseremoteconfig.googleapis.com/v1/projects/{}/remoteConfig",
            self.project_number
        )
    }
}
