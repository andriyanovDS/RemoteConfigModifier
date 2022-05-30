use super::config_command::ConfigFile;
use crate::config::Project;
use crate::error::{Error, Result};
use async_trait::async_trait;

#[async_trait]
pub trait Command {
    async fn run_for_single_project(self, project: &Project) -> Result<()>;
    async fn run_for_multiple_projects(self, projects: &[Project]) -> Result<()>;
}

pub struct CommandRunner {
    config_file: ConfigFile,
}

impl CommandRunner {
    pub fn new(app_name: String) -> Self {
        Self {
            config_file: ConfigFile::new(app_name),
        }
    }

    pub async fn run<C: Command>(self, command: C, arguments: crate::cli::Project) -> Result<()> {
        let mut projects = self.config_file.load()?.projects;

        if projects.is_empty() {
            return Err(Error::new(
                "Projects are empty! Add projects to projects.json file.",
            ));
        }

        if let Some(project_name) = arguments.project {
            let project_name = project_name.to_lowercase();
            let requested_project = projects
                .iter()
                .find(|project| project.name.to_lowercase() == project_name);
            return match requested_project {
                None => {
                    let project_names: Vec<_> = projects.iter().map(|proj| &proj.name).collect();
                    let error = Error {
                        message: format!(
                            "Project {} was not found. Available projects: {:?}",
                            project_name, project_names
                        ),
                    };
                    Err(error)
                }
                Some(project) => command.run_for_single_project(project).await,
            };
        }
        if let Some(main_project_name) = arguments.main {
            let main_project_name = main_project_name.to_lowercase();
            let project_index = projects.iter().enumerate().find_map(|(index, project)| {
                if project.name.to_lowercase() == main_project_name {
                    Some(index)
                } else {
                    None
                }
            });
            return match project_index {
                None => {
                    let project_names: Vec<_> = projects.iter().map(|proj| &proj.name).collect();
                    let error = Error {
                        message: format!(
                            "Project {} was not found. Available projects: {:?}",
                            main_project_name, project_names
                        ),
                    };
                    Err(error)
                }
                Some(index) => {
                    projects.swap(0, index);
                    command.run_for_multiple_projects(&projects).await
                }
            };
        } else {
            command.run_for_multiple_projects(&projects).await
        }
    }
}
