pub mod add_parameter_flow;
mod network;
mod remote_config;
mod error;
mod io;

use clap::{Parser, Subcommand};

/// CLI to add, update and delete Firebase Remote Config parameters
#[derive(Parser)]
#[clap(version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command
}

#[derive(Subcommand)]
pub enum Command {
    /// Adds new parameter to config. Can be used to replace existing one
    Add { name: Option<String> },
    /// Updates existing parameter to config
    Update { name: String, },
    /// Deletes parameter from config
    Delete { name: String },
}
