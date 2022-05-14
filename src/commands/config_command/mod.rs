mod fs;

use crate::config::Config;
use crate::config::Project;
use crate::error::Result;
use crate::Config as Subcommand;
pub use fs::ConfigFile;
use term_table::row::Row;
use term_table::table_cell::TableCell;
use term_table::{Table, TableStyle};
use tracing::info;

pub struct ConfigCommand {
    subcommand: Subcommand,
    config_file: ConfigFile,
}

impl ConfigCommand {
    pub fn new(app_name: String, subcommand: Subcommand) -> Self {
        Self {
            subcommand,
            config_file: ConfigFile::new(app_name),
        }
    }

    pub fn run(self) -> Result<()> {
        match self.subcommand {
            Subcommand::Add(data) => {
                let project = Project::new(data.name, data.project_number);
                let config = self.config_file.add_project(project)?;
                config.render(None);
                Ok(())
            }
            Subcommand::Remove { name } => {
                let config = self.config_file.remove_project(name.as_str())?;
                config.render(None);
                Ok(())
            }
            Subcommand::Store { path } => {
                self.config_file.store(path)?;
                let file_path = self.config_file.config_path()?;
                info!(
                    "Configuration file was successfully stored at {}",
                    file_path
                );
                Ok(())
            }
            Subcommand::Show(arguments) => {
                let config = self.config_file.load()?;
                let config_path = self.config_file.config_path()?;
                info!("Config was loaded from {}", config_path);
                config.render(arguments.project.as_ref());
                Ok(())
            }
        }
    }
}

impl Config {
    fn render(&self, project_name: Option<&String>) {
        let mut table = Table::new();
        table.style = TableStyle::simple();
        let header_row = Row::new(vec![
            TableCell::new("Project name"),
            TableCell::new("Project number"),
        ]);
        table.add_row(header_row);

        self.projects
            .iter()
            .filter(|project| {
                project_name
                    .map(|name| name == &project.name)
                    .unwrap_or(true)
            })
            .map(From::from)
            .for_each(|row| table.add_row(row));

        println!("{}", table.render());
    }
}
