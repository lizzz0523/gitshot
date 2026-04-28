use clap::{Parser, Subcommand};

pub mod diff;
pub mod status;

/// Render git output as a PNG image
#[derive(Parser)]
#[command(name = "gitshot", version, about)]
pub struct Cli {
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

pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Diff { paths, whitespace } => diff::run(&paths, whitespace),
        Commands::Status { paths } => status::run(&paths),
    }
}
