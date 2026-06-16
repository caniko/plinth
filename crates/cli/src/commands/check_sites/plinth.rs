use anyhow::anyhow;
use reqwest::StatusCode;
use serde::Deserialize;

use super::{ProbeKind, ProbeReport, SiteCheckTarget, failed_probe, route_url};

#[derive(Debug, Deserialize)]
pub(super) struct HealthResponse {
    pub(super) status: String,
    pub(super) db: Option<String>,
}

pub(super) fn default_routes() -> Vec<String> {
    vec![
        "/".to_string(),
        "/about".to_string(),
        "/posts".to_string(),
        "/projects".to_string(),
        "/feeds/blog.xml".to_string(),
        "/feeds/projects.xml".to_string(),
    ]
}

pub(super) async fn check_health(
    client: &reqwest::Client,
    target: &SiteCheckTarget,
) -> ProbeReport {
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
