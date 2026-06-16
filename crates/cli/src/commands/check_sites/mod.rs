use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use reqwest::{StatusCode, Url};
use serde::{Deserialize, Serialize};

use crate::ui;

mod plinth;
mod redirect;
mod static_site;
#[cfg(test)]
mod tests;

#[derive(Debug, Deserialize)]
pub struct SiteCheckConfig {
    #[serde(default)]
    pub targets: Vec<SiteCheckTarget>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SiteCheckTarget {
    pub id: String,
    pub title: String,
    pub url: String,
    pub kind: SiteCheckKind,
    #[serde(default)]
    pub routes: Vec<String>,
    #[serde(default)]
    pub markers: Vec<String>,
    #[serde(default = "default_expected_status")]
    pub expected_status: u16,
    #[serde(default = "default_follow_redirects")]
    pub follow_redirects: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteCheckKind {
    Plinth,
    Static,
}

#[derive(Debug, Serialize)]
pub struct SiteCheckReport {
    pub config_path: PathBuf,
    pub ok: bool,
    pub targets: Vec<TargetReport>,
}

#[derive(Debug, Serialize)]
pub struct TargetReport {
    pub id: String,
    pub title: String,
    pub url: String,
    pub kind: SiteCheckKind,
    pub ok: bool,
    pub probes: Vec<ProbeReport>,
}

#[derive(Debug, Serialize)]
pub struct ProbeReport {
    pub url: String,
    pub kind: ProbeKind,
    pub ok: bool,
    pub status: Option<u16>,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeKind {
    Health,
    Route,
}

fn default_expected_status() -> u16 {
    200
}

fn default_follow_redirects() -> bool {
    true
}

pub async fn check_sites(config_path: Option<&str>, json: bool) -> Result<()> {
    let config_path = resolve_config_path(config_path)?;
    let config = load_config(&config_path)?;
    let report = run_checks(config_path, config).await;
    let ok = report.ok;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_human_report(&report);
    }

    if ok {
        Ok(())
    } else {
        Err(anyhow!("one or more site checks failed"))
    }
}

fn resolve_config_path(config_path: Option<&str>) -> Result<PathBuf> {
    if let Some(path) = config_path {
        return Ok(PathBuf::from(path));
    }
    if let Ok(path) = env::var("PLINTH_SITE_CHECK_CONFIG")
        && !path.trim().is_empty()
    {
        return Ok(PathBuf::from(path));
    }

    let config_home = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .ok_or_else(|| {
            anyhow!(
                "no site check config path provided; pass --config, set PLINTH_SITE_CHECK_CONFIG, or set XDG_CONFIG_HOME/HOME"
            )
        })?;

    Ok(config_home.join("plinth").join("site-checks.toml"))
}

fn load_config(path: &Path) -> Result<SiteCheckConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read site check config {}", path.display()))?;
    let config: SiteCheckConfig = toml::from_str(&content)
        .with_context(|| format!("failed to parse site check config {}", path.display()))?;

    if config.targets.is_empty() {
        anyhow::bail!("site check config {} has no targets", path.display());
    }

    for target in &config.targets {
        validate_target(target)?;
    }

    Ok(config)
}

fn validate_target(target: &SiteCheckTarget) -> Result<()> {
    if target.id.trim().is_empty() {
        anyhow::bail!("site check target id must not be empty");
    }
    if target.title.trim().is_empty() {
        anyhow::bail!("site check target {} title must not be empty", target.id);
    }
    Url::parse(&target.url)
        .with_context(|| format!("site check target {} has invalid url", target.id))?;
    StatusCode::from_u16(target.expected_status).with_context(|| {
        format!(
            "site check target {} has invalid expected_status {}",
            target.id, target.expected_status
        )
    })?;
    Ok(())
}

async fn run_checks(config_path: PathBuf, config: SiteCheckConfig) -> SiteCheckReport {
    let mut targets = Vec::with_capacity(config.targets.len());

    for target in config.targets {
        targets.push(check_target(target).await);
    }

    let ok = targets.iter().all(|target| target.ok);
    SiteCheckReport {
        config_path,
        ok,
        targets,
    }
}

async fn check_target(target: SiteCheckTarget) -> TargetReport {
    let mut probes = Vec::new();
    let client = match redirect::client_for_target(&target) {
        Ok(client) => client,
        Err(err) => {
            return TargetReport {
                id: target.id,
                title: target.title,
                url: target.url,
                kind: target.kind,
                ok: false,
                probes: vec![ProbeReport {
                    url: String::new(),
                    kind: ProbeKind::Route,
                    ok: false,
                    status: None,
                    message: err.to_string(),
                }],
            };
        }
    };

    if target.kind == SiteCheckKind::Plinth {
        probes.push(plinth::check_health(&client, &target).await);
    }

    for route in routes_for_target(&target) {
        probes.push(redirect::check_route(&client, &target, &route).await);
    }

    let ok = probes.iter().all(|probe| probe.ok);
    TargetReport {
        id: target.id,
        title: target.title,
        url: target.url,
        kind: target.kind,
        ok,
        probes,
    }
}

fn routes_for_target(target: &SiteCheckTarget) -> Vec<String> {
    match target.kind {
        SiteCheckKind::Plinth => {
            if target.routes.is_empty() {
                plinth::default_routes()
            } else {
                target.routes.clone()
            }
        }
        SiteCheckKind::Static => static_site::routes(&target.routes),
    }
}

fn route_url(target: &SiteCheckTarget, route: &str) -> Result<Url> {
    let base = if target.url.ends_with('/') {
        target.url.clone()
    } else {
        format!("{}/", target.url)
    };
    let base =
        Url::parse(&base).with_context(|| format!("target {} has invalid url", target.id))?;
    base.join(route.trim_start_matches('/'))
        .with_context(|| format!("target {} has invalid route {route}", target.id))
}

fn failed_probe(
    url: &str,
    kind: ProbeKind,
    status: Option<u16>,
    err: anyhow::Error,
) -> ProbeReport {
    ProbeReport {
        url: url.to_string(),
        kind,
        ok: false,
        status,
        message: err.to_string(),
    }
}

fn print_human_report(report: &SiteCheckReport) {
    ui::status("Config", &report.config_path.display().to_string());
    for target in &report.targets {
        if target.ok {
            ui::success(&format!("{} ({})", target.title, target.url));
        } else {
            ui::warn(&format!("{} ({})", target.title, target.url));
        }

        for probe in &target.probes {
            let label = if probe.ok { "ok" } else { "fail" };
            let status = probe
                .status
                .map(|status| format!(" HTTP {status}"))
                .unwrap_or_default();
            ui::detail(&format!(
                "{label} {}{status} - {}",
                probe.url, probe.message
            ));
        }
    }
}
