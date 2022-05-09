pub mod commands;
mod error;
mod io;
mod network;
mod projects;
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
    Update { name: String },
    /// Deletes parameter from config
    Delete { name: String },
    /// Move parameter to group
    MoveTo(MoveTo),
    /// Move parameter out the group
    MoveOut { parameter: String },
    /// Show parameters and conditions
    Show(Project),
}

#[derive(Debug, Args)]
pub struct Add {
    #[clap(short, long)]
    pub name: Option<String>,
    #[clap(short, long)]
    pub description: Option<String>,
}

#[derive(Debug, Args)]
pub struct MoveTo {
    #[clap(short, long)]
    pub parameter: String,
    #[clap(short, long)]
    pub group: Option<String>,
}

#[derive(Debug, Args)]
pub struct Project {
    #[clap(short, long)]
    pub project: Option<String>,
    #[clap(short, long)]
    pub main: Option<String>,
}
