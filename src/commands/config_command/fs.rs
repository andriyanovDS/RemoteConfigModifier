use crate::config::{Config, Project};
use crate::error::{Error, Result};
use directories_next::ProjectDirs;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::ErrorKind::NotFound;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tracing::debug;

const CONFIG_FILE_NAME: &str = "config.json";

pub struct ConfigFile {
    app_name: String,
}

impl ConfigFile {
    pub fn new(app_name: String) -> Self {
        Self { app_name }
    }

    pub fn add_project(&self, project: Project) -> Result<Config> {
        let mut config = self.load()?;
        config.projects.push(project);
        let config_path = self.configuration_file_path()?;
        config.store(config_path.as_path())?;
        Ok(config)
    }

    pub fn remove_project(&self, project_name: &str) -> Result<Config> {
        let mut config = self.load()?;
        config
            .projects
            .retain(|project| project.name.as_str() != project_name);
        let config_path = self.configuration_file_path()?;
        config.store(config_path.as_path())?;
        Ok(config)
    }

    pub fn store(&self, path: PathBuf) -> Result<()> {
        if !path.exists() {
            return Err(Error {
                message: format!("File does not exist at path {:?}", path),
            });
        }
        let config = ConfigFile::load_at_path(&path)?;
        let config_path = self.configuration_file_path()?;
        config.store(config_path.as_path())?;
        Ok(())
    }

    pub fn load(&self) -> Result<Config> {
        let file_path = self.configuration_file_path()?;
        ConfigFile::load_at_path(&file_path)
    }

    pub fn config_path(&self) -> Result<String> {
        self.configuration_file_path().and_then(|path_buf| {
            path_buf
                .to_str()
                .map(|str| str.to_string())
                .ok_or_else(|| Error::new("Failed to construct config path"))
        })
    }

    fn load_at_path(file_path: &PathBuf) -> Result<Config> {
        match File::open(&file_path) {
            Ok(mut config_file) => {
                let mut content = String::new();
                config_file.read_to_string(&mut content).map_err(|error| {
                    debug!("Error: {:?}", error);
                    Error::new("Failed to read config file")
                })?;
                serde_json::from_slice::<Config>(content.as_bytes()).map_err(|error| {
                    debug!("Error: {:?}", error);
                    Error::new("Failed to read config file")
                })
            }
            Err(error) if error.kind() == NotFound => {
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent).map_err(|error| {
                        debug!("Error: {:?}", error);
                        Error::new("Failed to create config directory")
                    })?;
                    let config = Config {
                        projects: Vec::new(),
                    };
                    config.store(file_path.as_path())?;
                    Ok(config)
                } else {
                    debug!("Error: {:?}", error);
                    Err(Error::new("Failed to open configuration file"))
                }
            }
            Err(error) => Err(Error {
                message: error.to_string(),
            }),
        }
    }

    fn configuration_file_path(&self) -> Result<PathBuf> {
        let directories = ProjectDirs::from("com", "", &self.app_name)
            .ok_or_else(|| Error::new("Could not determine project directories path"))?;
        let path = directories
            .config_dir()
            .to_str()
            .ok_or_else(|| Error::new("Could not determine configuration file path"))?;
        let path = [path, CONFIG_FILE_NAME].iter().collect();
        Ok(path)
    }
}

impl Config {
    fn store(&self, path: &Path) -> Result<()> {
        let parent_path = path.parent().ok_or_else(|| Error::new("Invalid path"))?;
        fs::create_dir_all(parent_path).map_err(|error| {
            debug!("Error: {:?}", error);
            Error::new("Failed to create config directory.")
        })?;
        let string = serde_json::to_string_pretty(&self).map_err(|error| {
            debug!("Error: {:?}", error);
            Error::new("Failed to serialize config.")
        })?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .map_err(|error| {
                debug!("Error: {:?}", error);
                Error::new("Failed to open configuration file.")
            })?;

        file.write_all(string.as_bytes()).map_err(|error| {
            debug!("Error: {:?}", error);
            Error::new("Failed to write configuration file.")
        })
    }
}
