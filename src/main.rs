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
        Commands::Diff { paths, whitespace } => command::diff::run(&config, &paths, whitespace),
        Commands::Status { paths } => command::status::run(&config, &paths),
    }
}
