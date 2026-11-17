//! Command-line interface for `plinth-progress`.

use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "plinth-progress",
    version,
    about = "Validate and version-guard Plinth project progress sources"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse and validate the whole source, then require an exact version.
    Check {
        /// Exact canonical SemVer that must appear among the entries.
        #[arg(long, value_name = "VERSION")]
        version: String,
        /// Progress source to validate.
        source: PathBuf,
    },
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Check { version, source } => match check(&source, &version) {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                eprintln!("{message}");
                ExitCode::FAILURE
            }
        },
    }
}

fn check(source: &Path, version: &str) -> Result<(), String> {
    let text = std::fs::read_to_string(source)
        .map_err(|error| format!("error: {}: $: {error}", source.display()))?;
    let progress = plinth_progress::parse(&text, source).map_err(|report| report.render())?;
    plinth_progress::check_version(&progress, version).map_err(|report| report.render())
}
