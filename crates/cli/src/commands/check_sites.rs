use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result, anyhow};
use reqwest::{StatusCode, Url, redirect::Policy};
use serde::{Deserialize, Serialize};

use crate::ui;

const DEFAULT_TIMEOUT_SECS: u64 = 15;

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

#[derive(Debug, Deserialize)]
struct HealthResponse {
    status: String,
    db: Option<String>,
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
    if let Ok(path) = env::var("PLINTH_SITE_CHECK_CONFIG") {
        if !path.trim().is_empty() {
            return Ok(PathBuf::from(path));
        }
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
    let client = match client_for_target(&target) {
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
        probes.push(check_health(&client, &target).await);
    }

    for route in routes_for_target(&target) {
        probes.push(check_route(&client, &target, &route).await);
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

fn client_for_target(target: &SiteCheckTarget) -> Result<reqwest::Client> {
    let redirect_policy = if target.follow_redirects {
        Policy::limited(10)
    } else {
        Policy::none()
    };

    reqwest::Client::builder()
        .redirect(redirect_policy)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
        .build()
        .context("failed to build HTTP client")
}

fn routes_for_target(target: &SiteCheckTarget) -> Vec<String> {
    match target.kind {
        SiteCheckKind::Plinth => {
            if target.routes.is_empty() {
                vec![
                    "/".to_string(),
                    "/about".to_string(),
                    "/posts".to_string(),
                    "/projects".to_string(),
                    "/feeds/blog.xml".to_string(),
                    "/feeds/projects.xml".to_string(),
                ]
            } else {
                target.routes.clone()
            }
        }
        SiteCheckKind::Static => {
            let mut routes = vec!["/".to_string()];
            for route in &target.routes {
                if !routes.iter().any(|existing| existing == route) {
                    routes.push(route.clone());
                }
            }
            routes
        }
    }
}

async fn check_health(client: &reqwest::Client, target: &SiteCheckTarget) -> ProbeReport {
    let url = match route_url(target, "/api/health") {
        Ok(url) => url,
        Err(err) => return failed_probe("", ProbeKind::Health, None, err),
    };

    match client.get(url.clone()).send().await {
        Ok(response) => {
            let status = response.status();
            if status != StatusCode::OK {
                return failed_probe(
                    url.as_str(),
                    ProbeKind::Health,
                    Some(status.as_u16()),
                    anyhow!("expected HTTP 200, got {status}"),
                );
            }

            match response.json::<HealthResponse>().await {
                Ok(health) if health.status == "ok" && health.db.as_deref() == Some("ok") => {
                    ProbeReport {
                        url: url.to_string(),
                        kind: ProbeKind::Health,
                        ok: true,
                        status: Some(status.as_u16()),
                        message: "health ok".to_string(),
                    }
                }
                Ok(health) => failed_probe(
                    url.as_str(),
                    ProbeKind::Health,
                    Some(status.as_u16()),
                    anyhow!(
                        "health is not ok: status={}, db={}",
                        health.status,
                        health.db.unwrap_or_else(|| "missing".to_string())
                    ),
                ),
                Err(err) => failed_probe(
                    url.as_str(),
                    ProbeKind::Health,
                    Some(status.as_u16()),
                    anyhow!("failed to parse health JSON: {err}"),
                ),
            }
        }
        Err(err) => failed_probe(url.as_str(), ProbeKind::Health, None, anyhow!(err)),
    }
}

async fn check_route(
    client: &reqwest::Client,
    target: &SiteCheckTarget,
    route: &str,
) -> ProbeReport {
    let url = match route_url(target, route) {
        Ok(url) => url,
        Err(err) => return failed_probe("", ProbeKind::Route, None, err),
    };

    match client.get(url.clone()).send().await {
        Ok(response) => {
            let status = response.status();
            if status.as_u16() != target.expected_status {
                return failed_probe(
                    url.as_str(),
                    ProbeKind::Route,
                    Some(status.as_u16()),
                    anyhow!("expected HTTP {}, got {status}", target.expected_status),
                );
            }

            let content_type = response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .unwrap_or("")
                .to_string();
            let body = match response.text().await {
                Ok(body) => body,
                Err(err) => {
                    return failed_probe(
                        url.as_str(),
                        ProbeKind::Route,
                        Some(status.as_u16()),
                        anyhow!("failed to read response body: {err}"),
                    );
                }
            };

            let missing_markers: Vec<&str> = target
                .markers
                .iter()
                .filter_map(|marker| (!body.contains(marker)).then_some(marker.as_str()))
                .collect();

            if !missing_markers.is_empty() {
                return failed_probe(
                    url.as_str(),
                    ProbeKind::Route,
                    Some(status.as_u16()),
                    anyhow!("missing marker(s): {}", missing_markers.join(", ")),
                );
            }

            ProbeReport {
                url: url.to_string(),
                kind: ProbeKind::Route,
                ok: true,
                status: Some(status.as_u16()),
                message: if content_type.is_empty() {
                    "route ok".to_string()
                } else {
                    format!("route ok ({content_type})")
                },
            }
        }
        Err(err) => failed_probe(url.as_str(), ProbeKind::Route, None, anyhow!(err)),
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

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    fn target(url: String, kind: SiteCheckKind) -> SiteCheckTarget {
        SiteCheckTarget {
            id: "test".to_string(),
            title: "Test".to_string(),
            url,
            kind,
            routes: Vec::new(),
            markers: Vec::new(),
            expected_status: 200,
            follow_redirects: true,
        }
    }

    #[test]
    fn parses_defaults() {
        let config: SiteCheckConfig = toml::from_str(
            r#"
            [[targets]]
            id = "site"
            title = "Site"
            url = "https://example.com"
            kind = "static"
            "#,
        )
        .unwrap();

        let target = &config.targets[0];
        assert_eq!(target.expected_status, 200);
        assert!(target.follow_redirects);
    }

    #[test]
    fn expands_plinth_default_routes() {
        let routes = routes_for_target(&target(
            "https://example.com".to_string(),
            SiteCheckKind::Plinth,
        ));
        assert_eq!(
            routes,
            vec![
                "/",
                "/about",
                "/posts",
                "/projects",
                "/feeds/blog.xml",
                "/feeds/projects.xml"
            ]
        );
    }

    #[test]
    fn expands_static_root_once() {
        let mut target = target("https://example.com".to_string(), SiteCheckKind::Static);
        target.routes = vec!["/".to_string(), "/docs".to_string()];
        assert_eq!(routes_for_target(&target), vec!["/", "/docs"]);
    }

    #[tokio::test]
    async fn static_target_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_string("hello marker"))
            .mount(&server)
            .await;

        let mut target = target(server.uri(), SiteCheckKind::Static);
        target.markers = vec!["marker".to_string()];

        let report = check_target(target).await;
        assert!(report.ok);
    }

    #[tokio::test]
    async fn static_target_missing_marker_fails() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_string("hello"))
            .mount(&server)
            .await;

        let mut target = target(server.uri(), SiteCheckKind::Static);
        target.markers = vec!["marker".to_string()];

        let report = check_target(target).await;
        assert!(!report.ok);
        assert!(report.probes[0].message.contains("missing marker"));
    }

    #[tokio::test]
    async fn plinth_bad_health_json_fails() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/health"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&server)
            .await;
        for route in routes_for_target(&target(server.uri(), SiteCheckKind::Plinth)) {
            Mock::given(method("GET"))
                .and(path(route))
                .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
                .mount(&server)
                .await;
        }

        let report = check_target(target(server.uri(), SiteCheckKind::Plinth)).await;
        assert!(!report.ok);
        assert!(
            report.probes[0]
                .message
                .contains("failed to parse health JSON")
        );
    }

    #[tokio::test]
    async fn redirect_handling_can_be_disabled() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(302).insert_header("location", "/final"))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/final"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&server)
            .await;

        let mut follows = target(server.uri(), SiteCheckKind::Static);
        assert!(check_target(follows.clone()).await.ok);

        follows.follow_redirects = false;
        let report = check_target(follows).await;
        assert!(!report.ok);
        assert_eq!(report.probes[0].status, Some(302));
    }
}
