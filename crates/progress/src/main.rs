//! Command-line interface for `plinth-progress`.

use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use plinth_project::ProjectTheme;

#[derive(Debug, Parser)]
#[command(
    name = "plinth-progress",
    version,
    about = "Validate, version-guard and render Plinth project progress sources"
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
    /// Validate the whole source and render one self-contained HTML page.
    Build {
        /// Progress source to validate and render.
        source: PathBuf,
        /// Exact HTML file to write.
        #[arg(long, value_name = "FILE")]
        out: PathBuf,
        /// Project config whose `[theme]` table styles the page.
        #[arg(long, value_name = "FILE")]
        config: Option<PathBuf>,
    },
    /// Print a TOML entry skeleton for a new version. Never edits files.
    Draft {
        /// Exact canonical SemVer of the new entry.
        #[arg(long, value_name = "VERSION")]
        version: String,
        /// Git ref to list commits after; defaults to the newest tag.
        #[arg(long, value_name = "REF")]
        since: Option<String>,
        /// Entry date as YYYY-MM-DD; defaults to today in UTC.
        #[arg(long, value_name = "YYYY-MM-DD")]
        date: Option<String>,
        /// Progress source that receives the new entry.
        source: PathBuf,
    },
}

fn main() -> ExitCode {
    let result = match Cli::parse().command {
        Command::Check { version, source } => check(&source, &version),
        Command::Build {
            source,
            out,
            config,
        } => build(&source, &out, config.as_deref()),
        Command::Draft {
            version,
            since,
            date,
            source,
        } => draft(&source, &version, since.as_deref(), date.as_deref()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn check(source: &Path, version: &str) -> Result<(), String> {
    let progress = load(source)?;
    plinth_progress::check_version(&progress, version).map_err(|report| report.render())
}

fn build(source: &Path, out: &Path, config: Option<&Path>) -> Result<(), String> {
    let progress = load(source)?;

    let theme = match config {
        Some(config) => {
            let (theme, warnings) =
                plinth_progress::load_theme(config).map_err(|report| report.render())?;
            for warning in warnings {
                eprintln!("warning: {}: $: {warning}", config.display());
            }
            theme
        }
        None => ProjectTheme::default(),
    };

    let html = plinth_progress::render(&progress, &theme);
    write_atomically(out, &html)
}

fn draft(
    source: &Path,
    version: &str,
    since: Option<&str>,
    date: Option<&str>,
) -> Result<(), String> {
    let progress = load(source)?;
    let (text, warnings) = plinth_progress::draft(&progress, version, since, date)
        .map_err(|report| report.render())?;
    for warning in warnings {
        eprintln!("warning: {}: $: {warning}", source.display());
    }
    print!("{text}");
    Ok(())
}

fn load(source: &Path) -> Result<plinth_progress::Progress, String> {
    let text = std::fs::read_to_string(source).map_err(|error| io_error(source, &error))?;
    plinth_progress::parse(&text, source).map_err(|report| report.render())
}

/// Writes a temporary sibling first, then renames it over `out`, so a failed
/// write never leaves a partial page and the old file stays until the new one
/// is complete.
fn write_atomically(out: &Path, html: &str) -> Result<(), String> {
    if let Some(parent) = out.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|error| io_error(out, &error))?;
    }

    let name = out
        .file_name()
        .ok_or_else(|| format!("error: {}: $: not a file path", out.display()))?;
    let mut temporary = std::ffi::OsString::from(".");
    temporary.push(name);
    temporary.push(format!(".{}.tmp", std::process::id()));
    let temporary = out.with_file_name(temporary);

    // `create_new` refuses an existing file or symlink, so the temporary can
    // never alias `out` or clobber something else; only a file this run
    // created is ever removed.
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| io_error(&temporary, &error))?;
    let written = std::io::Write::write_all(&mut file, html.as_bytes())
        .and_then(|()| file.sync_all())
        .and_then(|()| std::fs::rename(&temporary, out));
    if let Err(error) = written {
        let _ = std::fs::remove_file(&temporary);
        return Err(io_error(out, &error));
    }
    Ok(())
}

fn io_error(path: &Path, error: &std::io::Error) -> String {
    format!("error: {}: $: {error}", path.display())
}
