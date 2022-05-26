use serde::{Deserialize, Serialize};
use term_table::row::Row;
use term_table::table_cell::TableCell;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub projects: Vec<Project>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Project {
    pub name: String,
    pub app_ids: Vec<String>,
    project_number: String,
}

impl Project {
    pub fn new(name: String, project_number: String, app_ids: Vec<String>) -> Self {
        Self {
            name,
            project_number,
            app_ids,
        }
    }
    pub fn url(&self) -> String {
        format!(
            "https://firebaseremoteconfig.googleapis.com/v1/projects/{}/remoteConfig",
            self.project_number
        )
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            projects: Vec::new(),
        }
    }
}

impl<'a> From<&'a Project> for Row<'a> {
    fn from(project: &'a Project) -> Self {
        Row::new(vec![
            TableCell::new(&project.name),
            TableCell::new(&project.project_number),
            TableCell::new(format!("{:#?}", project.app_ids))
        ])
    }
}
