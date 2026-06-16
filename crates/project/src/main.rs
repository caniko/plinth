#[cfg(feature = "brick-install")]
use std::process::Command;
use std::{
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

#[cfg(feature = "brick-install")]
use anyhow::bail;
use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use plinth_project::{
    Asset, Diagnostic, DiagnosticReport, ProjectSection, ProjectSite, RenderOptions, Severity,
    dev::{ServeOptions, StaticServer, serve_development, start_development_server},
    load_project_site, project_watch_paths, render_static, validate_site,
};
#[cfg(feature = "brick-install")]
use plinth_project::{InstallUxReport, install_ux_report, render_install_fragment};
use serde::Serialize;

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
struct ConfigArgs {
    /// Project-site TOML config. If omitted, checks current-directory defaults.
    #[arg(long)]
    config: Option<PathBuf>,
}

#[derive(clap::Args, Clone)]
struct JsonArgs {
    /// Emit machine-readable JSON.
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args, Clone)]
struct ServeArgsBase {
    /// Host address to bind.
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    /// TCP port to bind. Port 0 asks the OS for an ephemeral port.
    #[arg(long, default_value_t = 1111)]
    port: u16,
    /// Open the served site in the default browser.
    #[arg(long = "open", default_value_t = true)]
    open: bool,
    /// Do not open the default browser.
    #[arg(long, overrides_with = "open")]
    no_open: bool,
}

impl ServeArgsBase {
    fn open_browser(&self) -> bool {
        self.open && !self.no_open
    }
}

#[derive(clap::Args, Clone)]
struct InitArgs {
    /// Site title for the generated definition.
    #[arg(long, default_value = "Project")]
    title: String,
    /// Site description for the generated definition.
    #[arg(long, default_value = "A Plinth project site.")]
    description: String,
    /// Config path to create.
    #[arg(long, default_value = DEFAULT_CONFIG)]
    out_config: PathBuf,
    /// Replace an existing config file.
    #[arg(long)]
    force: bool,
}

#[derive(clap::Args, Clone)]
struct CheckArgs {
    #[command(flatten)]
    config: ConfigArgs,
    #[command(flatten)]
    json: JsonArgs,
}

#[derive(clap::Args, Clone)]
struct BuildArgs {
    #[command(flatten)]
    config: ConfigArgs,
    /// Output directory.
    #[arg(long, default_value = DEFAULT_BUILD_OUT)]
    out: PathBuf,
    #[command(flatten)]
    json: JsonArgs,
}

#[derive(clap::Args, Clone)]
struct DevArgs {
    #[command(flatten)]
    config: ConfigArgs,
    /// Output directory.
    #[arg(long, default_value = DEFAULT_DEV_OUT)]
    out: PathBuf,
    #[command(flatten)]
    serve: ServeArgsBase,
    /// Do not watch project-site inputs.
    #[arg(long)]
    no_watch: bool,
}

#[derive(clap::Args, Clone)]
struct PreviewArgs {
    /// Built static site directory to serve.
    #[arg(long, default_value = DEFAULT_BUILD_OUT)]
    dir: PathBuf,
    #[command(flatten)]
    serve: ServeArgsBase,
}

#[derive(clap::Args, Clone)]
struct InspectArgs {
    #[command(flatten)]
    config: ConfigArgs,
    #[command(flatten)]
    json: JsonArgs,
}

#[derive(clap::Args, Clone)]
struct PublishArgs {
    #[command(flatten)]
    config: ConfigArgs,
    /// Output directory.
    #[arg(long, default_value = DEFAULT_PUBLISH_OUT)]
    out: PathBuf,
    #[command(flatten)]
    json: JsonArgs,
}

#[derive(clap::Args, Clone)]
struct RenderArgs {
    /// Project-site TOML config.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Output directory.
    #[arg(long, default_value = DEFAULT_BUILD_OUT)]
    out: PathBuf,
}

#[derive(clap::Args, Clone)]
struct ServeArgs {
    /// Project-site TOML config.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Output directory.
    #[arg(long, default_value = DEFAULT_BUILD_OUT)]
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
    config: ConfigArgs,
    /// Render output directory used for the audit page.
    #[arg(long, default_value = DEFAULT_BUILD_OUT)]
    out: PathBuf,
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
#[derive(serde::Deserialize)]
struct VisualRubricCliVerdict {
    verdict: String,
    reason: String,
    #[serde(default)]
    anomalies: Vec<String>,
}

#[derive(Serialize)]
struct CheckJson<'a> {
    config: &'a Path,
    diagnostics: &'a [Diagnostic],
    ok: bool,
}

#[derive(Serialize)]
struct BuildJson<'a> {
    config: &'a Path,
    out: &'a Path,
    diagnostics: &'a [Diagnostic],
    ok: bool,
}

#[derive(Serialize)]
struct InspectJson<'a> {
    config: &'a Path,
    title: &'a str,
    description: &'a str,
    base_url: &'a str,
    pages: Vec<PageJson<'a>>,
    assets: Vec<AssetJson<'a>>,
    people: Vec<PersonJson<'a>>,
    projects: Vec<ProjectJson<'a>>,
    watch_paths: Vec<PathBuf>,
}

#[derive(Serialize)]
struct PageJson<'a> {
    slug: &'a str,
    title: &'a str,
    description: &'a str,
    sections: Vec<&'static str>,
}

#[derive(Serialize)]
struct AssetJson<'a> {
    source: &'a Path,
    target: &'a str,
}

#[derive(Serialize)]
struct PersonJson<'a> {
    id: &'a str,
    name: &'a str,
    url: &'a str,
}

#[derive(Serialize)]
struct ProjectJson<'a> {
    title: &'a str,
    url: &'a str,
    source_url: Option<&'a str>,
    demo_url: Option<&'a str>,
}

#[derive(Serialize)]
struct PublishManifest<'a> {
    generator: &'static str,
    version: &'static str,
    title: &'a str,
    pages: Vec<PageManifest<'a>>,
    files: Vec<String>,
}

#[derive(Serialize)]
struct PageManifest<'a> {
    slug: &'a str,
    title: &'a str,
    path: String,
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

/// visual-rubric question preset carrying the install-section question and
/// system prompt; see `presets` in the visual-rubric crate.
#[cfg(feature = "brick-install")]
const WEBSITE_RUBRIC_PRESET: &str = "website-install";

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Init(args) => init_site(&args),
        Commands::Check(args) => check_site(&args),
        Commands::Build(args) => build_site(&args).map(|_| ()),
        Commands::Dev(args) => dev_site(args),
        Commands::Preview(args) => preview_site(args),
        Commands::Inspect(args) => inspect_site(&args),
        Commands::Publish(args) => publish_site(&args),
        Commands::Render(args) => build_site(&BuildArgs {
            config: ConfigArgs {
                config: args.config,
            },
            out: args.out,
            json: JsonArgs { json: false },
        })
        .map(|_| ()),
        Commands::Serve(args) => serve_site(args),
        #[cfg(feature = "brick-install")]
        Commands::Audit(AuditCommands::Install(args)) => audit_install(&args),
    }
}

fn init_site(args: &InitArgs) -> Result<()> {
    if args.out_config.exists() && !args.force {
        return Err(anyhow!(
            "{} already exists; pass --force to replace it",
            args.out_config.display()
        ));
    }
    if let Some(parent) = args.out_config.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let template = initial_config_template(&args.title, &args.description);
    fs::write(&args.out_config, template)
        .with_context(|| format!("failed to write {}", args.out_config.display()))?;
    println!("Created {}", args.out_config.display());
    Ok(())
}

fn check_site(args: &CheckArgs) -> Result<()> {
    let config = resolve_config_path(args.config.config.clone())?;
    let site = load_project_site(&config).context("load project site")?;
    let report = validate_site(&site);
    emit_check(&config, &report, args.json.json)?;
    if report.has_errors() {
        return Err(anyhow!("project-site diagnostics failed"));
    }
    Ok(())
}

fn build_site(args: &BuildArgs) -> Result<ProjectSite> {
    let config = resolve_config_path(args.config.config.clone())?;
    let site = load_project_site(&config).context("load project site")?;
    let report = validate_site(&site);
    if args.json.json {
        emit_json(&BuildJson {
            config: &config,
            out: &args.out,
            diagnostics: &report.diagnostics,
            ok: !report.has_errors(),
        })?;
    } else {
        emit_diagnostics(&report);
    }
    if report.has_errors() {
        return Err(anyhow!("project-site diagnostics failed"));
    }
    render_static(&site, &RenderOptions::new(&args.out)).context("render project site")?;
    if !args.json.json {
        println!("Rendered {} -> {}", config.display(), args.out.display());
    }
    Ok(site)
}

fn dev_site(args: DevArgs) -> Result<()> {
    let config = resolve_config_path(args.config.config)?;
    let open_browser = args.serve.open_browser();
    serve_rendered_site(ServeRenderedArgs {
        config,
        out: args.out,
        host: args.serve.host,
        port: args.serve.port,
        open_browser,
        watch: !args.no_watch,
    })
}

fn serve_site(args: ServeArgs) -> Result<()> {
    let config = resolve_config_path(args.config)?;
    serve_rendered_site(ServeRenderedArgs {
        config,
        out: args.out,
        host: args.host,
        port: args.port,
        open_browser: !args.no_open,
        watch: args.watch,
    })
}

fn preview_site(args: PreviewArgs) -> Result<()> {
    if !args.dir.exists() {
        return Err(anyhow!(
            "{} does not exist; run `plinth-project build --out {}` first",
            args.dir.display(),
            args.dir.display()
        ));
    }
    let open_browser = args.serve.open_browser();
    let server = StaticServer::start(args.dir.clone(), args.serve.host, args.serve.port)
        .context("failed to start preview server")?;
    let url = server.base_url();
    println!("Serving built project site at {url}");
    if open_browser {
        open::that(&url).with_context(|| format!("failed to open {url}"))?;
    }
    if std::env::var_os("PLINTH_PROJECT_SERVE_ONCE").is_some() {
        return Ok(());
    }
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

fn inspect_site(args: &InspectArgs) -> Result<()> {
    let config = resolve_config_path(args.config.config.clone())?;
    let site = load_project_site(&config).context("load project site")?;
    let watch_paths = project_watch_paths(&config).context("load project watch paths")?;

    if args.json.json {
        emit_json(&inspect_json(&config, &site, watch_paths))?;
    } else {
        print_inspection(&config, &site, &watch_paths);
    }
    Ok(())
}

fn publish_site(args: &PublishArgs) -> Result<()> {
    let config = resolve_config_path(args.config.config.clone())?;
    let site = load_project_site(&config).context("load project site")?;
    let report = validate_site(&site);
    if report.has_errors() {
        emit_check(&config, &report, args.json.json)?;
        return Err(anyhow!("project-site diagnostics failed"));
    }
    render_static(&site, &RenderOptions::new(&args.out)).context("render project site")?;
    let manifest = publish_manifest(&site, &args.out)?;
    let manifest_path = args.out.join("plinth-project-manifest.json");
    write_json_report(&manifest_path, &manifest)?;
    if args.json.json {
        emit_json(&manifest)?;
    } else {
        println!("Published {} -> {}", config.display(), args.out.display());
        println!("Wrote {}", manifest_path.display());
    }
    Ok(())
}

struct ServeRenderedArgs {
    config: PathBuf,
    out: PathBuf,
    host: String,
    port: u16,
    open_browser: bool,
    watch: bool,
}

fn serve_rendered_site(args: ServeRenderedArgs) -> Result<()> {
    let mut options = ServeOptions::new(&args.out);
    options.host = args.host;
    options.port = args.port;
    options.open_browser = args.open_browser;
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

fn resolve_config_path(explicit: Option<PathBuf>) -> Result<PathBuf> {
    resolve_config_path_in(
        &std::env::current_dir().context("failed to read current directory")?,
        explicit,
    )
}

fn resolve_config_path_in(cwd: &Path, explicit: Option<PathBuf>) -> Result<PathBuf> {
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

fn emit_check(config: &Path, report: &DiagnosticReport, json: bool) -> Result<()> {
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

fn emit_diagnostics(report: &DiagnosticReport) {
    for diagnostic in &report.diagnostics {
        println!("{}", format_diagnostic(diagnostic));
    }
}

fn format_diagnostic(diagnostic: &Diagnostic) -> String {
    let severity = match diagnostic.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    };
    format!("{severity}[{}]: {}", diagnostic.code, diagnostic.message)
}

fn emit_json<T: Serialize>(value: &T) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).context("failed to encode JSON")?
    );
    Ok(())
}

fn inspect_json<'a>(
    config: &'a Path,
    site: &'a ProjectSite,
    watch_paths: Vec<PathBuf>,
) -> InspectJson<'a> {
    InspectJson {
        config,
        title: &site.title,
        description: &site.description,
        base_url: &site.base_url,
        pages: site
            .pages
            .iter()
            .map(|page| PageJson {
                slug: &page.slug,
                title: &page.title,
                description: &page.description,
                sections: page.sections.iter().map(section_name).collect(),
            })
            .collect(),
        assets: site.assets.iter().map(asset_json).collect(),
        people: site
            .people
            .iter()
            .map(|person| PersonJson {
                id: &person.id,
                name: &person.name,
                url: &person.url,
            })
            .collect(),
        projects: site
            .projects
            .iter()
            .map(|project| ProjectJson {
                title: &project.title,
                url: &project.url,
                source_url: project.source_url.as_deref(),
                demo_url: project.demo_url.as_deref(),
            })
            .collect(),
        watch_paths,
    }
}

fn print_inspection(config: &Path, site: &ProjectSite, watch_paths: &[PathBuf]) {
    println!("Config: {}", config.display());
    println!("Title: {}", site.title);
    println!("Description: {}", site.description);
    println!("Base URL: {}", site.base_url);
    println!("Pages:");
    for page in &site.pages {
        let sections = page
            .sections
            .iter()
            .map(section_name)
            .collect::<Vec<_>>()
            .join(", ");
        println!("  - {} ({}) [{}]", page.title, page.slug, sections);
    }
    println!("Assets:");
    if site.assets.is_empty() {
        println!("  - none");
    } else {
        for asset in &site.assets {
            println!("  - {} -> {}", asset.source.display(), asset.target);
        }
    }
    println!("People:");
    if site.people.is_empty() {
        println!("  - none");
    } else {
        for person in &site.people {
            println!("  - {} ({})", person.name, person.id);
        }
    }
    println!("Projects:");
    if site.projects.is_empty() {
        println!("  - none");
    } else {
        for project in &site.projects {
            println!("  - {} ({})", project.title, project.url);
        }
    }
    println!("Watch paths:");
    for path in watch_paths {
        println!("  - {}", path.display());
    }
}

fn asset_json(asset: &Asset) -> AssetJson<'_> {
    AssetJson {
        source: &asset.source,
        target: &asset.target,
    }
}

fn section_name(section: &ProjectSection) -> &'static str {
    match section {
        #[cfg(feature = "brick-hero")]
        ProjectSection::Hero(_) => "hero",
        #[cfg(feature = "brick-feature-grid")]
        ProjectSection::FeatureGrid(_) => "feature_grid",
        #[cfg(feature = "brick-install")]
        ProjectSection::Install(_) => "install",
        #[cfg(feature = "brick-person-mention")]
        ProjectSection::PersonMention(_) => "person_mention",
        #[cfg(feature = "brick-workflow-steps")]
        ProjectSection::WorkflowSteps(_) => "workflow_steps",
        #[cfg(feature = "brick-audience-grid")]
        ProjectSection::AudienceGrid(_) => "audience_grid",
        #[cfg(feature = "brick-trust-panel")]
        ProjectSection::TrustPanel(_) => "trust_panel",
        #[cfg(feature = "brick-screenshot-grid")]
        ProjectSection::ScreenshotGrid(_) => "screenshot_grid",
        #[cfg(feature = "brick-capability-matrix")]
        ProjectSection::CapabilityMatrix(_) => "capability_matrix",
        #[cfg(feature = "brick-comparison")]
        ProjectSection::Comparison(_) => "comparison",
        #[cfg(feature = "brick-content")]
        ProjectSection::Content(_) => "content",
        #[cfg(feature = "brick-custom")]
        ProjectSection::Custom(_) => "custom",
    }
}

fn publish_manifest<'a>(site: &'a ProjectSite, out: &Path) -> Result<PublishManifest<'a>> {
    let mut files = collect_files(out)?;
    files.retain(|file| file != "plinth-project-manifest.json");
    Ok(PublishManifest {
        generator: "plinth-project",
        version: env!("CARGO_PKG_VERSION"),
        title: &site.title,
        pages: site
            .pages
            .iter()
            .map(|page| PageManifest {
                slug: &page.slug,
                title: &page.title,
                path: if page.slug == "index" {
                    "index.html".into()
                } else {
                    format!("{}/index.html", page.slug)
                },
            })
            .collect(),
        files,
    })
}

fn collect_files(root: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();
    collect_files_inner(root, root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files_inner(root: &Path, dir: &Path, files: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry.with_context(|| format!("failed to read {}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_inner(root, &path, files)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(root)
                .context("generated file was outside output directory")?;
            files.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

fn initial_config_template(title: &str, description: &str) -> String {
    format!(
        r#"[site]
title = "{}"
description = "{}"
base_url = "/"
footer_note = "{}"

[[nav]]
label = "Home"
href = "/"

[[pages]]
slug = "index"
title = "{}"
description = "{}"

[[pages.sections]]
type = "hero"
title = "{}"
tagline = "{}"
subtitle = "{}"
"#,
        toml_escape(title),
        toml_escape(description),
        toml_escape(description),
        toml_escape(title),
        toml_escape(description),
        toml_escape(title),
        toml_escape(description),
        toml_escape(description)
    )
}

fn toml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(feature = "brick-install")]
fn audit_install(args: &AuditInstallArgs) -> Result<()> {
    let config = resolve_config_path(args.config.config.clone())?;
    let site = load_project_site(&config).context("load project site")?;
    render_static(&site, &RenderOptions::new(&args.out)).context("render project site")?;
    let install = install_ux_report(&site).context("site does not define an install section")?;
    let audit_path = write_install_audit_page(&args.out, &site)?;
    create_clean_dir(&args.screenshots)?;

    let server = StaticServer::start(args.out.clone(), "127.0.0.1", 0)
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
    // `configured` runs the machine's active rubric backend
    // (~/.config/visual-rubric/config.toml) instead of hardcoding one.
    let output = Command::new(rubric_bin)
        .arg("configured")
        .arg("--json")
        .arg("--image")
        .arg(image)
        .arg("--preset")
        .arg(WEBSITE_RUBRIC_PRESET)
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

fn write_json_report<T: Serialize>(path: &Path, report: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(report).context("failed to encode JSON")?;
    fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_resolution_uses_current_directory_only_order() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("website")).unwrap();
        fs::write(dir.path().join("plinth-project.toml"), "").unwrap();
        fs::write(dir.path().join(DEFAULT_CONFIG), "").unwrap();

        let resolved = resolve_config_path_in(dir.path(), None).unwrap();
        assert_eq!(resolved, dir.path().join(DEFAULT_CONFIG));
    }

    #[test]
    fn config_resolution_falls_back_to_root_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(FALLBACK_CONFIG), "").unwrap();

        let resolved = resolve_config_path_in(dir.path(), None).unwrap();
        assert_eq!(resolved, dir.path().join(FALLBACK_CONFIG));
    }

    #[test]
    fn missing_config_error_names_init_and_config_flag() {
        let dir = tempfile::tempdir().unwrap();
        let error = resolve_config_path_in(dir.path(), None)
            .unwrap_err()
            .to_string();
        assert!(error.contains("plinth-project init"));
        assert!(error.contains("--config"));
        assert!(error.contains(DEFAULT_CONFIG));
    }

    #[test]
    fn diagnostic_human_format_is_stable() {
        let diagnostic = Diagnostic {
            severity: Severity::Error,
            code: "install.no_recommended_route",
            message: "install section must mark one route as recommended".into(),
        };

        assert_eq!(
            format_diagnostic(&diagnostic),
            "error[install.no_recommended_route]: install section must mark one route as recommended"
        );
    }

    #[test]
    fn diagnostic_json_format_is_stable() {
        let report = DiagnosticReport {
            diagnostics: vec![Diagnostic {
                severity: Severity::Warning,
                code: "install.unanchored_guide_link",
                message: "prefer a route-specific anchor".into(),
            }],
        };
        let json = serde_json::to_string(&report).unwrap();

        assert_eq!(
            json,
            r#"{"diagnostics":[{"severity":"warning","code":"install.unanchored_guide_link","message":"prefer a route-specific anchor"}]}"#
        );
    }

    #[test]
    fn publish_manifest_includes_generated_files_and_metadata() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("index.html"), "home").unwrap();
        fs::write(dir.path().join("style.css"), "body{}").unwrap();
        let site = ProjectSite::new("Example", "Example site")
            .page(plinth_project::Page::new("index", "Example"));

        let manifest = publish_manifest(&site, dir.path()).unwrap();

        assert_eq!(manifest.generator, "plinth-project");
        assert_eq!(manifest.title, "Example");
        assert_eq!(manifest.pages[0].path, "index.html");
        assert_eq!(manifest.files, vec!["index.html", "style.css"]);
    }

    #[test]
    fn init_template_escapes_toml_strings() {
        let template = initial_config_template("A \"quoted\" site", "Path \\\\ ready");
        toml::from_str::<plinth_project::ProjectConfig>(&template).unwrap();
        assert!(template.contains("A \\\"quoted\\\" site"));
    }
}
