pub mod add_parameter_flow;
pub mod delete_parameter_flow;
mod error;
mod io;
pub mod move_out_group;
pub mod move_to_group_flow;
mod network;
mod remote_config;
pub mod show_config_flow;
pub mod update_parameter_flow;

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
    /// Show config parameters and conditions
    Show,
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
