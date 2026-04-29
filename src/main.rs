use clap::Parser;

use crate::cli::{Cli, Commands};
use crate::config::Config;

mod cli;
mod command;
mod config;
mod model;
mod renderer;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;

    match cli.command {
        Commands::Diff(args) => command::diff::run(&config, &args),
        Commands::Status(args) => command::status::run(&config, &args),
    }
}
