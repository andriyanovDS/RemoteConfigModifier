extern crate core;

pub mod commands;
mod config;
mod error;
mod io;
pub mod network;
mod remote_config;

use clap::{Args, Parser, Subcommand};

/// CLI to add, update and delete Firebase Remote Config parameters
#[derive(Parser)]
#[clap(name = "remote_config_modifier",version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Adds new parameter to config. Can be used to replace existing one
    Add(Add),
    /// Updates existing parameter to config
    Update(Update),
    /// Deletes parameter from config
    Delete(Delete),
    /// Move parameter to group
    MoveTo(MoveTo),
    /// Move parameter out the group
    MoveOut(MoveOut),
    /// Show parameters and conditions
    Show(Project),
    /// Show projects stored in config file
    #[clap(subcommand)]
    Config(Config),
}

#[derive(Debug, Args)]
pub struct Add {
    /// Parameter name
    #[clap(short, long)]
    pub name: Option<String>,
    /// Parameter to add
    #[clap(short, long)]
    pub description: Option<String>,
    #[clap(flatten)]
    pub project: Project,
}

#[derive(Debug, Args)]
pub struct Update {
    /// Parameter to update
    #[clap(short, long)]
    pub name: String,
    #[clap(flatten)]
    pub project: Project,
}

#[derive(Debug, Args)]
pub struct MoveOut {
    /// Parameter to move
    #[clap(long)]
    pub name: String,
    #[clap(flatten)]
    pub project: Project,
}

#[derive(Debug, Args)]
pub struct MoveTo {
    /// Parameter to move
    #[clap(long)]
    pub name: String,
    /// Group where the parameter will be moved
    #[clap(short, long)]
    pub group: Option<String>,
    #[clap(flatten)]
    pub project: Project,
}

#[derive(Debug, Args)]
pub struct Delete {
    /// Parameter to delete
    #[clap(short, long)]
    pub name: String,
    #[clap(flatten)]
    pub project: Project,
}

#[derive(Debug, Subcommand)]
pub enum Config {
    /// Load config from JSON file
    #[clap(parse(from_os_str))]
    Store { path: std::path::PathBuf },
    /// Add project to configuration file
    Add(AddProject),
    /// Remove project from configuration file
    #[clap(name = "rm")]
    Remove { name: String },
    /// Show configuration
    Show(Project),
}

#[derive(Debug, Args)]
pub struct AddProject {
    /// Project name
    #[clap(short, long)]
    pub name: String,
    /// Application IDs
    #[clap(short, long)]
    pub app_ids: Option<Vec<String>>,
    /// Project description
    #[clap(long)]
    pub project_number: String,
}

#[derive(Debug, Args)]
pub struct Project {
    /// Specify single project for command
    #[clap(short, long)]
    pub project: Option<String>,
    #[clap(short, long)]
    pub main: Option<String>,
}
