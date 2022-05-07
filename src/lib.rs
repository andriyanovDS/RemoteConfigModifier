pub mod add_parameter_flow;
pub mod delete_parameter_flow;
mod error;
mod io;
mod network;
mod remote_config;

use clap::{Parser, Subcommand, Args};

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
}

#[derive(Debug, Args)]
pub struct Add {
    #[clap(short, long)]
    pub name: Option<String>,
    #[clap(short, long)]
    pub description: Option<String>
}
