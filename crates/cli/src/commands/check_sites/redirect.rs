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

    let mut builder = reqwest::Client::builder()
        .redirect(redirect_policy)
        .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS));

    // Site-check tests and local operators often probe loopback services from
    // an environment that has an outbound HTTP proxy configured.  A proxy is
    // never appropriate for a loopback target, and bypassing it here also
    // keeps the check deterministic inside Nix's network-isolated builders.
    let target_url = reqwest::Url::parse(&target.url).ok();
    let is_loopback = target_url
        .as_ref()
        .and_then(reqwest::Url::host_str)
        .is_some_and(|host| matches!(host, "localhost" | "127.0.0.1" | "::1"));
    if is_loopback {
        builder = builder.no_proxy();

        // reqwest 0.13's default rustls verifier loads the platform CA store
        // while constructing the client.  A loopback HTTP probe does not use
        // TLS, so requiring that store makes local checks fail in hermetic
        // environments (such as Nix sandboxes) before the request is sent.
        // Keep normal certificate verification for HTTPS and non-loopback
        // targets, but use an empty, explicit root set for HTTP loopback
        // probes so client construction remains independent of host CA files.
        if target_url
            .as_ref()
            .is_some_and(|url| url.scheme() == "http")
        {
            builder = builder.tls_certs_only(std::iter::empty());
        }
    }

    builder
        .build()
        .with_context(|| format!("failed to build HTTP client for {}", target.url))
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
