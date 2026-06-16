use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use reqwest::redirect::Policy;

use super::{ProbeKind, ProbeReport, SiteCheckTarget, failed_probe, route_url};

const DEFAULT_TIMEOUT_SECS: u64 = 15;

pub(super) fn client_for_target(target: &SiteCheckTarget) -> Result<reqwest::Client> {
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

pub(super) async fn check_route(
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
