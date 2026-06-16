use crate::{BuildArgs, BuildJson, emit_diagnostics, emit_json, resolve_config_path};
use anyhow::{Context, Result, anyhow};
use plinth_project::{ProjectSite, RenderOptions, load_project_site, render_static, validate_site};

pub fn build_site(args: &BuildArgs) -> Result<ProjectSite> {
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
