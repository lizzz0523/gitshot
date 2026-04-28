use clap::{Parser, Subcommand};

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
    Diff {
        /// Path(s) to diff (file or directory). Defaults to current directory.
        #[arg(default_value = ".")]
        paths: Vec<String>,
        /// Show whitespace changes (ignored by default)
        #[arg(short = 'w', long)]
        whitespace: bool,
    },
    /// Render git status as a PNG image
    Status {
        /// Path(s) to check status (file or directory). Defaults to current directory.
        #[arg(default_value = ".")]
        paths: Vec<String>,
    },
}
