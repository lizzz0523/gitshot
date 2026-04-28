use clap::{Parser, Subcommand};

use crate::command::{diff, status};
use crate::config::Config;

mod command;
mod config;
mod inline_diff;
mod renderer;

/// Render git output as a PNG image
#[derive(Parser)]
#[command(name = "gitshot", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Render git diff as a PNG image
    Diff {
        /// Path(s) to diff (file or directory). Defaults to current directory.
        #[arg(default_values_t = vec![".".to_string()])]
        paths: Vec<String>,
        /// Show whitespace changes (ignored by default)
        #[arg(short = 'w', long)]
        whitespace: bool,
    },
    /// Render git status as a PNG image
    Status {
        /// Path(s) to check status (file or directory). Defaults to current directory.
        #[arg(default_values_t = vec![".".to_string()])]
        paths: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let config = Config::load();

    match cli.command {
        Commands::Diff { paths, whitespace } => diff::run(&config, &paths, whitespace),
        Commands::Status { paths } => status::run(&config, &paths),
    }
}
