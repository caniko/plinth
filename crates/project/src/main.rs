use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
#[cfg(feature = "brick-install")]
use plinth_project::InstallUxReport;
use plinth_project::{Diagnostic, DiagnosticReport, Severity};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

mod cmd;
#[cfg(test)]
mod tests;

const DEFAULT_CONFIG: &str = "website/plinth-project.toml";
const FALLBACK_CONFIG: &str = "plinth-project.toml";
const DEFAULT_BUILD_OUT: &str = "website/public";
const DEFAULT_DEV_OUT: &str = ".plinth-project/public";
const DEFAULT_PUBLISH_OUT: &str = "dist/plinth-project";

#[derive(Parser)]
#[command(name = "plinth-project")]
#[command(about = "Render, serve, audit, and publish static Plinth project sites")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
enum Commands {
    /// Create a minimal project-site definition.
    Init(InitArgs),
    /// Validate a project-site definition.
    Check(CheckArgs),
    /// Render a project site to static files.
    Build(BuildArgs),
    /// Render and serve a project site during local development.
    Dev(DevArgs),
    /// Serve an already-built static site directory.
    Preview(PreviewArgs),
    /// Inspect resolved project-site metadata.
    Inspect(InspectArgs),
    /// Produce a local deployable static bundle.
    Publish(PublishArgs),
    /// Compatibility alias for `build`.
    Render(RenderArgs),
    /// Compatibility alias for `dev --no-watch`.
    Serve(ServeArgs),
    /// Run project-site audits.
    #[cfg(feature = "brick-install")]
    #[command(subcommand)]
    Audit(AuditCommands),
}

#[derive(clap::Args, Clone)]
pub struct ConfigArgs {
    /// Project-site TOML config. If omitted, checks current-directory defaults.
    #[arg(long)]
    pub config: Option<PathBuf>,
}

#[derive(clap::Args, Clone)]
pub struct JsonArgs {
    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(clap::Args, Clone)]
pub struct ServeArgsBase {
    /// Host address to bind.
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    /// TCP port to bind. Port 0 asks the OS for an ephemeral port.
    #[arg(long, default_value_t = 1111)]
    pub port: u16,
    /// Open the served site in the default browser.
    #[arg(long = "open", default_value_t = true)]
    pub open: bool,
    /// Do not open the default browser.
    #[arg(long, overrides_with = "open")]
    pub no_open: bool,
}

impl ServeArgsBase {
    pub fn open_browser(&self) -> bool {
        self.open && !self.no_open
    }
}

#[derive(clap::Args, Clone)]
pub struct InitArgs {
    /// Site title for the generated definition.
    #[arg(long, default_value = "Project")]
    pub title: String,
    /// Site description for the generated definition.
    #[arg(long, default_value = "A Plinth project site.")]
    pub description: String,
    /// Config path to create.
    #[arg(long, default_value = DEFAULT_CONFIG)]
    pub out_config: PathBuf,
    /// Replace an existing config file.
    #[arg(long)]
    pub force: bool,
}

#[derive(clap::Args, Clone)]
pub struct CheckArgs {
    #[command(flatten)]
    pub config: ConfigArgs,
    #[command(flatten)]
    pub json: JsonArgs,
}

#[derive(clap::Args, Clone)]
pub struct BuildArgs {
    #[command(flatten)]
    pub config: ConfigArgs,
    /// Output directory.
    #[arg(long, default_value = DEFAULT_BUILD_OUT)]
    pub out: PathBuf,
    #[command(flatten)]
    pub json: JsonArgs,
}

#[derive(clap::Args, Clone)]
pub struct DevArgs {
    #[command(flatten)]
    pub config: ConfigArgs,
    /// Output directory.
    #[arg(long, default_value = DEFAULT_DEV_OUT)]
    pub out: PathBuf,
    #[command(flatten)]
    pub serve: ServeArgsBase,
    /// Do not watch project-site inputs.
    #[arg(long)]
    pub no_watch: bool,
}

#[derive(clap::Args, Clone)]
pub struct PreviewArgs {
    /// Built static site directory to serve.
    #[arg(long, default_value = DEFAULT_BUILD_OUT)]
    pub dir: PathBuf,
    #[command(flatten)]
    pub serve: ServeArgsBase,
}

#[derive(clap::Args, Clone)]
pub struct InspectArgs {
    #[command(flatten)]
    pub config: ConfigArgs,
    #[command(flatten)]
    pub json: JsonArgs,
}

#[derive(clap::Args, Clone)]
pub struct PublishArgs {
    #[command(flatten)]
    pub config: ConfigArgs,
    /// Output directory.
    #[arg(long, default_value = DEFAULT_PUBLISH_OUT)]
    pub out: PathBuf,
    #[command(flatten)]
    pub json: JsonArgs,
}

#[derive(clap::Args, Clone)]
pub struct RenderArgs {
    /// Project-site TOML config.
    #[arg(long)]
    pub config: Option<PathBuf>,
    /// Output directory.
    #[arg(long, default_value = DEFAULT_BUILD_OUT)]
    pub out: PathBuf,
}

#[derive(clap::Args, Clone)]
pub struct ServeArgs {
    /// Project-site TOML config.
    #[arg(long)]
    pub config: Option<PathBuf>,
    /// Output directory.
    #[arg(long, default_value = DEFAULT_BUILD_OUT)]
    pub out: PathBuf,
    /// Host address to bind.
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    /// TCP port to bind. Port 0 asks the OS for an ephemeral port.
    #[arg(long, default_value_t = 1111)]
    pub port: u16,
    /// Do not open the default browser.
    #[arg(long)]
    pub no_open: bool,
    /// Watch the config directory and reload browser tabs after rerender.
    #[arg(long)]
    pub watch: bool,
}

#[cfg(feature = "brick-install")]
#[derive(Subcommand)]
enum AuditCommands {
    /// Audit the install section with deterministic checks and screenshots.
    Install(AuditInstallArgs),
    /// Audit every rendered page with visual-rubric.
    Site(AuditSiteArgs),
}

#[cfg(feature = "brick-install")]
#[derive(clap::Args)]
pub struct AuditInstallArgs {
    #[command(flatten)]
    pub config: ConfigArgs,
    /// Render output directory used for the audit page.
    #[arg(long, default_value = DEFAULT_BUILD_OUT)]
    pub out: PathBuf,
    /// Screenshot output directory.
    #[arg(long, default_value = "target/site-audit")]
    pub screenshots: PathBuf,
    /// JSON report path.
    #[arg(long, default_value = "target/site-audit/report.json")]
    pub report: PathBuf,
    /// Browser executable for headless screenshots.
    #[arg(long, env = "PLINTH_PROJECT_BROWSER", default_value = "chromium")]
    pub browser: PathBuf,
    /// visual-rubric executable.
    #[arg(long, env = "VISUAL_RUBRIC_BIN", default_value = "visual-rubric")]
    pub rubric_bin: PathBuf,
    /// Use a passing fake AI verdict.
    #[arg(long)]
    pub fake_ai: bool,
    /// Skip the AI rubric after deterministic checks and screenshots.
    #[arg(long)]
    pub skip_ai: bool,
}

#[cfg(feature = "brick-install")]
#[derive(clap::Args)]
pub struct AuditSiteArgs {
    #[command(flatten)]
    pub config: ConfigArgs,
    /// Render output directory used for the audited site.
    #[arg(long, default_value = DEFAULT_BUILD_OUT)]
    pub out: PathBuf,
    /// Screenshot output directory.
    #[arg(long, default_value = "target/site-audit")]
    pub screenshots: PathBuf,
    /// JSON report path.
    #[arg(long, default_value = "target/site-audit/site-report.json")]
    pub report: PathBuf,
    /// Browser executable for headless screenshots.
    #[arg(long, env = "PLINTH_PROJECT_BROWSER", default_value = "chromium")]
    pub browser: PathBuf,
    /// visual-rubric executable.
    #[arg(long, env = "VISUAL_RUBRIC_BIN", default_value = "visual-rubric")]
    pub rubric_bin: PathBuf,
    /// Local visual-rubric checkout to run through its Nix dev shell.
    #[arg(long, env = "VISUAL_RUBRIC_PROJECT")]
    pub rubric_project: Option<PathBuf>,
    /// Use a passing fake AI verdict.
    #[arg(long)]
    pub fake_ai: bool,
    /// Skip the AI rubric after screenshots.
    #[arg(long)]
    pub skip_ai: bool,
    /// Explicit route to audit. Repeat to override rendered-page discovery.
    #[arg(long = "route")]
    pub routes: Vec<String>,
    /// Do not fail the command when visual-rubric reports fail/error.
    #[arg(long)]
    pub no_fail_on_rubric: bool,
    /// Use the shared persistent-browser and rubric-batch producer contract.
    #[arg(long)]
    pub shared_capture: bool,
    /// Root directory for shared capture artifacts.
    #[arg(long, default_value = "target/visual/captures")]
    pub capture_output: PathBuf,
    /// Versioned shared capture manifest path.
    #[arg(long, default_value = "target/visual/capture_manifest.json")]
    pub capture_manifest: PathBuf,
    /// Versioned shared producer report path.
    #[arg(long, default_value = "target/visual/run_report.json")]
    pub visual_report: PathBuf,
    /// Number of persistent rubric workers.
    #[arg(long, default_value_t = 4)]
    pub rubric_workers: usize,
    /// Content-addressed rubric cache directory.
    #[arg(long)]
    pub rubric_cache: Option<PathBuf>,
}

#[cfg(feature = "brick-install")]
#[derive(Serialize)]
pub struct SiteAuditReport {
    pub install: InstallUxReport,
    pub screenshots: Vec<ScreenshotAudit>,
}

#[cfg(feature = "brick-install")]
#[derive(Serialize)]
pub struct FullSiteAuditReport {
    pub config: PathBuf,
    pub out: PathBuf,
    pub served_url: String,
    pub preset: &'static str,
    pub pages: Vec<PageAuditReport>,
}

#[cfg(feature = "brick-install")]
#[derive(Serialize)]
pub struct PageAuditReport {
    pub route: String,
    pub screenshots: Vec<ScreenshotAudit>,
}

#[cfg(feature = "brick-install")]
#[derive(Serialize)]
pub struct ScreenshotAudit {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub path: PathBuf,
    pub rubric: RubricAudit,
}

#[cfg(feature = "brick-install")]
#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RubricAudit {
    Pass {
        reason: String,
        anomalies: Vec<String>,
    },
    Fail {
        reason: String,
        anomalies: Vec<String>,
    },
    Error {
        message: String,
    },
    Skipped {
        reason: String,
    },
}

#[cfg(feature = "brick-install")]
pub struct Viewport {
    pub name: &'static str,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize)]
pub struct CheckJson<'a> {
    pub config: &'a Path,
    pub diagnostics: &'a [Diagnostic],
    pub ok: bool,
}

#[derive(Serialize)]
pub struct BuildJson<'a> {
    pub config: &'a Path,
    pub out: &'a Path,
    pub diagnostics: &'a [Diagnostic],
    pub ok: bool,
}

#[derive(Serialize)]
pub struct InspectJson<'a> {
    pub config: &'a Path,
    pub title: &'a str,
    pub description: &'a str,
    pub base_url: &'a str,
    pub pages: Vec<PageJson<'a>>,
    pub assets: Vec<AssetJson<'a>>,
    pub people: Vec<PersonJson<'a>>,
    pub projects: Vec<ProjectJson<'a>>,
    pub watch_paths: Vec<PathBuf>,
}

#[derive(Serialize)]
pub struct PageJson<'a> {
    pub slug: &'a str,
    pub title: &'a str,
    pub description: &'a str,
    pub sections: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct AssetJson<'a> {
    pub source: &'a Path,
    pub target: &'a str,
}

#[derive(Serialize)]
pub struct PersonJson<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub url: &'a str,
}

#[derive(Serialize)]
pub struct ProjectJson<'a> {
    pub title: &'a str,
    pub url: &'a str,
    pub source_url: Option<&'a str>,
    pub demo_url: Option<&'a str>,
}

#[derive(Serialize)]
pub struct PublishManifest<'a> {
    pub generator: &'static str,
    pub version: &'static str,
    pub title: &'a str,
    pub pages: Vec<PageManifest<'a>>,
    pub files: Vec<String>,
}

#[derive(Serialize)]
pub struct PageManifest<'a> {
    pub slug: &'a str,
    pub title: &'a str,
    pub path: String,
}

#[cfg(feature = "brick-install")]
const VIEWPORTS: &[Viewport] = &[
    Viewport {
        name: "desktop",
        width: 1440,
        height: 900,
    },
    Viewport {
        name: "tablet-landscape",
        width: 1024,
        height: 768,
    },
    Viewport {
        name: "tablet-portrait",
        width: 768,
        height: 1024,
    },
    Viewport {
        name: "mobile",
        width: 390,
        height: 844,
    },
];

#[cfg(feature = "brick-install")]
const WEBSITE_RUBRIC_PRESET: &str = "website-install";

#[cfg(feature = "brick-install")]
const PLINTH_SITE_BEAUTY_PRESET: &str = "plinth-site-beauty";

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Init(args) => cmd::init_site(&args),
        Commands::Check(args) => cmd::check_site(&args),
        Commands::Build(args) => cmd::build_site(&args).map(|_| ()),
        Commands::Dev(args) => cmd::dev_site(args),
        Commands::Preview(args) => cmd::preview_site(args),
        Commands::Inspect(args) => cmd::inspect_site(&args),
        Commands::Publish(args) => cmd::publish_site(&args),
        Commands::Render(args) => cmd::build_site(&BuildArgs {
            config: ConfigArgs {
                config: args.config,
            },
            out: args.out,
            json: JsonArgs { json: false },
        })
        .map(|_| ()),
        Commands::Serve(args) => cmd::serve_site(args),
        #[cfg(feature = "brick-install")]
        Commands::Audit(AuditCommands::Install(args)) => cmd::audit_install(&args),
        #[cfg(feature = "brick-install")]
        Commands::Audit(AuditCommands::Site(args)) => cmd::audit_site(&args),
    }
}

pub fn resolve_config_path(explicit: Option<PathBuf>) -> Result<PathBuf> {
    resolve_config_path_in(
        &std::env::current_dir().context("failed to read current directory")?,
        explicit,
    )
}

pub fn resolve_config_path_in(cwd: &Path, explicit: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }

    for candidate in [DEFAULT_CONFIG, FALLBACK_CONFIG] {
        let path = cwd.join(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    Err(anyhow!(
        "could not find a project-site config in {}; expected `{}` or `{}`. Run `plinth-project init` to create one, or pass `--config path/to/plinth-project.toml`.",
        cwd.display(),
        DEFAULT_CONFIG,
        FALLBACK_CONFIG
    ))
}

pub fn emit_check(config: &Path, report: &DiagnosticReport, json: bool) -> Result<()> {
    if json {
        emit_json(&CheckJson {
            config,
            diagnostics: &report.diagnostics,
            ok: !report.has_errors(),
        })
    } else {
        emit_diagnostics(report);
        if report.diagnostics.is_empty() {
            println!("OK {}", config.display());
        }
        Ok(())
    }
}

pub fn emit_diagnostics(report: &DiagnosticReport) {
    for diagnostic in &report.diagnostics {
        println!("{}", format_diagnostic(diagnostic));
    }
}

pub fn format_diagnostic(diagnostic: &Diagnostic) -> String {
    let severity = match diagnostic.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    };
    format!("{severity}[{}]: {}", diagnostic.code, diagnostic.message)
}

pub fn emit_json<T: Serialize>(value: &T) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).context("failed to encode JSON")?
    );
    Ok(())
}

pub fn write_json_report<T: Serialize>(path: &Path, report: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(report).context("failed to encode JSON")?;
    fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))
}
