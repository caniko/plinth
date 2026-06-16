use crate::{CheckArgs, emit_check, resolve_config_path};
use anyhow::{Context, Result, anyhow};
use plinth_project::{load_project_site, validate_site};

pub fn check_site(args: &CheckArgs) -> Result<()> {
    let config = resolve_config_path(args.config.config.clone())?;
    let site = load_project_site(&config).context("load project site")?;
    let report = validate_site(&site);
    emit_check(&config, &report, args.json.json)?;
    if report.has_errors() {
        return Err(anyhow!("project-site diagnostics failed"));
    }
    Ok(())
}
