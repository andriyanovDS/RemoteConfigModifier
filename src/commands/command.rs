use crate::error::{Error, Result};
use crate::projects::{Project, Projects};
use async_trait::async_trait;

#[async_trait]
pub trait Command {
    async fn run_for_single_project(self, project: &Project) -> Result<()>;
    async fn run_for_multiple_projects(self, projects: &[Project]) -> Result<()>;
}

pub struct CommandRunner<C: Command> {
    command: C,
}

impl<C> CommandRunner<C>
where
    C: Command,
{
    pub fn new(command: C) -> Self {
        Self { command }
    }

    pub async fn run(self, arguments: crate::Project) -> Result<()> {
        let mut projects = Projects::load_projects().await?;
        if let Some(project_name) = arguments.project {
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
                Some(project) => self.command.run_for_single_project(project).await,
            };
        }
        if let Some(main_project_name) = arguments.main {
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
                    self.command.run_for_multiple_projects(&projects).await
                }
            };
        } else {
            self.command.run_for_multiple_projects(&projects).await
        }
    }
}
