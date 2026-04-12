use anyhow::{Context, Result};
use serde::Deserialize;

use crate::ui;

#[derive(Debug, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
    api_version: u32,
    db: Option<String>,
    vector_search: Option<String>,
    immich: Option<String>,
}

pub async fn check_status(api_url: &str) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .context("Failed to build HTTP client")?;

    let url = format!("{api_url}/api/health");
    let sp = ui::spinner(&format!("Checking {url}..."));

    let response = client.get(&url).send().await;
    sp.finish_and_clear();

    match response {
        Ok(resp) => {
            let status_code = resp.status();
            let health: HealthResponse = resp
                .json()
                .await
                .context("Failed to parse health response")?;

            if health.status == "ok" {
                ui::success(&format!(
                    "{api_url} v{} (API v{}) — HTTP {status_code}",
                    health.version, health.api_version
                ));
            } else {
                ui::warn(&format!(
                    "{api_url} {} v{} (API v{}) — HTTP {status_code}",
                    health.status, health.version, health.api_version
                ));
            }

            if let Some(db) = &health.db {
                let label = if db == "ok" { "  db" } else { " db!" };
                ui::status(label, db);
            }
            if let Some(vs) = &health.vector_search {
                let label = if vs == "available" { "  vs" } else { " vs!" };
                ui::status(label, vs);
            }
            if let Some(immich) = &health.immich {
                let label = if immich == "ok" {
                    "  immich"
                } else {
                    " immich!"
                };
                ui::status(label, immich);
            }

            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Cannot reach {api_url}: {e}");
        }
    }
}
