#[cfg(feature = "brick-install")]
use std::fs;
#[cfg(feature = "brick-install")]
use std::path::Path;
use std::path::PathBuf;
#[cfg(feature = "brick-install")]
use std::process::Command;

#[cfg(feature = "brick-install")]
use anyhow::bail;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
#[cfg(feature = "brick-install")]
use plinth_project::{
    InstallUxReport, ProjectSection, dev::StaticServer, install_ux_report, render_install_fragment,
};
use plinth_project::{
    ProjectSite, RenderOptions,
    dev::{ServeOptions, serve_development, start_development_server},
    load_project_site, project_watch_paths, render_static,
};
#[cfg(feature = "brick-install")]
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "plinth-project")]
#[command(about = "Render, serve, and audit static Plinth project sites")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Render a project site to static files.
    Render(RenderArgs),
    /// Render and serve a project site during development.
    Serve(ServeArgs),
    /// Run project-site audits.
    #[cfg(feature = "brick-install")]
    #[command(subcommand)]
    Audit(AuditCommands),
}

#[derive(clap::Args, Clone)]
struct RenderArgs {
    /// Project-site TOML config.
    #[arg(long, default_value = "website/plinth-project.toml")]
    config: PathBuf,
    /// Output directory.
    #[arg(long, default_value = "website/public")]
    out: PathBuf,
}

#[derive(clap::Args, Clone)]
struct ServeArgs {
    /// Project-site TOML config.
    #[arg(long, default_value = "website/plinth-project.toml")]
    config: PathBuf,
    /// Output directory.
    #[arg(long, default_value = "website/public")]
    out: PathBuf,
    /// Host address to bind.
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    /// TCP port to bind. Port 0 asks the OS for an ephemeral port.
    #[arg(long, default_value_t = 1111)]
    port: u16,
    /// Do not open the default browser.
    #[arg(long)]
    no_open: bool,
    /// Watch the config directory and reload browser tabs after rerender.
    #[arg(long)]
    watch: bool,
}

#[cfg(feature = "brick-install")]
#[derive(Subcommand)]
enum AuditCommands {
    /// Audit the install section with deterministic checks and screenshots.
    Install(AuditInstallArgs),
}

#[cfg(feature = "brick-install")]
#[derive(clap::Args)]
struct AuditInstallArgs {
    #[command(flatten)]
    render: RenderArgs,
    /// Screenshot output directory.
    #[arg(long, default_value = "target/site-audit")]
    screenshots: PathBuf,
    /// JSON report path.
    #[arg(long, default_value = "target/site-audit/report.json")]
    report: PathBuf,
    /// Browser executable for headless screenshots.
    #[arg(long, env = "PLINTH_PROJECT_BROWSER", default_value = "chromium")]
    browser: PathBuf,
    /// visual-rubric executable.
    #[arg(long, env = "VISUAL_RUBRIC_BIN", default_value = "visual-rubric")]
    rubric_bin: PathBuf,
    /// Use a passing fake AI verdict.
    #[arg(long)]
    fake_ai: bool,
    /// Skip the AI rubric after deterministic checks and screenshots.
    #[arg(long)]
    skip_ai: bool,
}

#[cfg(feature = "brick-install")]
#[derive(Serialize)]
struct SiteAuditReport {
    install: InstallUxReport,
    screenshots: Vec<ScreenshotAudit>,
}

#[cfg(feature = "brick-install")]
#[derive(Serialize)]
struct ScreenshotAudit {
    name: String,
    width: u32,
    height: u32,
    path: PathBuf,
    rubric: RubricAudit,
}

#[cfg(feature = "brick-install")]
#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum RubricAudit {
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
struct Viewport {
    name: &'static str,
    width: u32,
    height: u32,
}

#[cfg(feature = "brick-install")]
#[derive(Deserialize)]
struct VisualRubricCliVerdict {
    verdict: String,
    reason: String,
    #[serde(default)]
    anomalies: Vec<String>,
}

#[cfg(feature = "brick-install")]
const VIEWPORTS: &[Viewport] = &[
    Viewport {
        name: "desktop",
        width: 1440,
        height: 1100,
    },
    Viewport {
        name: "mobile",
        width: 390,
        height: 1800,
    },
];

#[cfg(feature = "brick-install")]
const WEBSITE_RUBRIC_PROMPT: &str = "\
You are auditing a software project website install section. Focus on install flow clarity, \
scanability, heading hierarchy, call-to-action placement, prerequisite visibility, command-copy \
ergonomics, responsive layout, text clipping, overlapping UI, and whether the next step is obvious. \
Reply with strict JSON matching this schema and nothing else:
{ \"verdict\": \"pass\" | \"fail\", \"reason\": string, \"anomalies\": string[] }";

#[cfg(feature = "brick-install")]
const WEBSITE_RUBRIC_QUESTION: &str = "\
Does this page make the install section easy to find, choose from, and act on without layout or \
responsive UX defects?";

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Render(args) => render_site(&args).map(|_| ()),
        Commands::Serve(args) => serve_site(args),
        #[cfg(feature = "brick-install")]
        Commands::Audit(AuditCommands::Install(args)) => audit_install(&args),
    }
}

fn render_site(args: &RenderArgs) -> Result<ProjectSite> {
    let site = load_project_site(&args.config).context("load project site")?;
    render_static(&site, &RenderOptions::new(&args.out)).context("render project site")?;
    Ok(site)
}

fn serve_site(args: ServeArgs) -> Result<()> {
    let mut options = ServeOptions::new(&args.out);
    options.host = args.host;
    options.port = args.port;
    options.open_browser = !args.no_open;
    options.watch = args.watch;
    options.reload = args.watch;
    options.watch_paths = project_watch_paths(&args.config).context("load project watch paths")?;

    if std::env::var_os("PLINTH_PROJECT_SERVE_ONCE").is_some() {
        let (_server, _reload) =
            start_development_server(&options, &|| load_project_site(&args.config))
                .context("start project-site server")?;
        return Ok(());
    }

    serve_development(options, || load_project_site(&args.config)).context("serve project site")
}

#[cfg(feature = "brick-install")]
fn audit_install(args: &AuditInstallArgs) -> Result<()> {
    let site = render_site(&args.render)?;
    let install = install_ux_report(&site).context("site does not define an install section")?;
    let audit_path = write_install_audit_page(&args.render.out, &site)?;
    create_clean_dir(&args.screenshots)?;

    let server = StaticServer::start(args.render.out.clone(), "127.0.0.1", 0)
        .context("failed to start static server")?;
    let audit_url = format!("{}{}", server.base_url(), audit_path);
    let mut screenshots = Vec::new();

    for viewport in VIEWPORTS {
        let path = args.screenshots.join(format!("{}.png", viewport.name));
        capture_screenshot(&args.browser, &audit_url, viewport, &path)?;
        let rubric = if args.fake_ai {
            RubricAudit::Pass {
                reason: "fake rubric enabled".into(),
                anomalies: Vec::new(),
            }
        } else if args.skip_ai {
            RubricAudit::Skipped {
                reason: "AI rubric skipped by flag".into(),
            }
        } else {
            run_visual_rubric(&args.rubric_bin, &path)
        };

        screenshots.push(ScreenshotAudit {
            name: viewport.name.into(),
            width: viewport.width,
            height: viewport.height,
            path,
            rubric,
        });
    }

    drop(server);

    write_json_report(
        &args.report,
        &SiteAuditReport {
            install,
            screenshots,
        },
    )
}

#[cfg(feature = "brick-install")]
fn write_install_audit_page(out: &Path, site: &ProjectSite) -> Result<String> {
    let install = site
        .pages
        .iter()
        .flat_map(|page| &page.sections)
        .find_map(|section| match section {
            ProjectSection::Install(install) => Some(install),
            _ => None,
        })
        .context("site does not define an install section")?;
    let audit_path = Path::new("__audit").join("install.html");
    let body = render_install_fragment(install);
    let html = format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Install Audit</title><link rel=\"stylesheet\" href=\"/style.css\"></head><body><main>{body}</main></body></html>"
    );
    let target = out.join(&audit_path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&target, html).with_context(|| format!("failed to write {}", target.display()))?;
    Ok(audit_path.to_string_lossy().replace('\\', "/"))
}

#[cfg(feature = "brick-install")]
fn capture_screenshot(browser: &Path, url: &str, viewport: &Viewport, output: &Path) -> Result<()> {
    let status = Command::new(browser)
        .arg("--headless")
        .arg("--disable-gpu")
        .arg("--hide-scrollbars")
        .arg("--no-sandbox")
        .arg(format!(
            "--window-size={},{}",
            viewport.width, viewport.height
        ))
        .arg(format!("--screenshot={}", output.display()))
        .arg(url)
        .status()
        .with_context(|| {
            format!(
                "failed to run browser {}; set PLINTH_PROJECT_BROWSER or pass --browser",
                browser.display()
            )
        })?;
    if !status.success() {
        bail!(
            "browser {} failed for {} screenshot with status {status}",
            browser.display(),
            viewport.name
        );
    }
    if !output.exists() {
        bail!(
            "browser {} reported success but did not write {}",
            browser.display(),
            output.display()
        );
    }
    Ok(())
}

#[cfg(feature = "brick-install")]
fn run_visual_rubric(rubric_bin: &Path, image: &Path) -> RubricAudit {
    let output = Command::new(rubric_bin)
        .arg("--json")
        .arg("--image")
        .arg(image)
        .arg("--question")
        .arg(WEBSITE_RUBRIC_QUESTION)
        .arg("--system-prompt")
        .arg(WEBSITE_RUBRIC_PROMPT)
        .output();
    let output = match output {
        Ok(output) => output,
        Err(error) => {
            return RubricAudit::Error {
                message: format!(
                    "failed to run visual-rubric binary {}; set VISUAL_RUBRIC_BIN or pass --rubric-bin: {error}",
                    rubric_bin.display()
                ),
            };
        }
    };

    if !output.status.success() {
        return RubricAudit::Error {
            message: format!(
                "visual-rubric exited with status {}: {}{}",
                output.status,
                String::from_utf8_lossy(&output.stderr),
                String::from_utf8_lossy(&output.stdout)
            ),
        };
    }

    match serde_json::from_slice::<VisualRubricCliVerdict>(&output.stdout) {
        Ok(verdict) if verdict.verdict == "pass" => RubricAudit::Pass {
            reason: verdict.reason,
            anomalies: verdict.anomalies,
        },
        Ok(verdict) => RubricAudit::Fail {
            reason: verdict.reason,
            anomalies: verdict.anomalies,
        },
        Err(error) => RubricAudit::Error {
            message: format!(
                "failed to parse visual-rubric JSON from {:?}: {error}",
                String::from_utf8_lossy(&output.stdout)
            ),
        },
    }
}

#[cfg(feature = "brick-install")]
fn create_clean_dir(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).with_context(|| format!("failed to clean {}", path.display()))?;
    }
    fs::create_dir_all(path).with_context(|| format!("failed to create {}", path.display()))
}

#[cfg(feature = "brick-install")]
fn write_json_report(path: &Path, report: &SiteAuditReport) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(report).context("failed to encode audit report")?;
    fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))
}
