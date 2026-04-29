use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Render git output as a PNG image
#[derive(Parser)]
#[command(name = "gitshot", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Render git diff as a PNG image
    Diff(DiffArgs),
    /// Render git status as a PNG image
    Status(StatusArgs),
}

/// Output file path. Defaults to a file under the system temp dir when omitted.
#[derive(Args)]
pub struct OutputArgs {
    #[arg(short = 'o', long, value_name = "FILE")]
    pub output: Option<PathBuf>,
}

#[derive(Args)]
pub struct DiffArgs {
    /// Path(s) to diff (file or directory). Defaults to current directory.
    #[arg(default_value = ".")]
    pub paths: Vec<String>,
    /// Show whitespace changes (ignored by default)
    #[arg(short = 'w', long)]
    pub whitespace: bool,
    #[command(flatten)]
    pub output: OutputArgs,
}

#[derive(Args)]
pub struct StatusArgs {
    /// Path(s) to check status (file or directory). Defaults to current directory.
    #[arg(default_value = ".")]
    pub paths: Vec<String>,
    #[command(flatten)]
    pub output: OutputArgs,
}
