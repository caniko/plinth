use anyhow::{Context, Result};
use plinth_shared::toml_config::PlinthConfig;

use crate::ui;

pub fn validate(path: Option<&str>) -> Result<()> {
    let config_path = path
        .map(String::from)
        .unwrap_or_else(|| std::env::var("PLINTH_CONFIG").unwrap_or_else(|_| "plinth.toml".into()));

    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Cannot read '{config_path}'"))?;

    let config = PlinthConfig::parse(&content)
        .with_context(|| format!("Invalid TOML in '{config_path}'"))?;

    ui::success(&format!("'{config_path}' is valid"));
    ui::status("Site", &config.site.name);
    ui::status(
        "Server",
        &format!("{}:{}", config.server.host, config.server.port),
    );
    ui::status("DB", &config.database.path);

    if !config.immich.api_url.is_empty() {
        ui::status("Immich", &config.immich.api_url);
    }
    if !config.observability.otlp_endpoint.is_empty() {
        ui::status("OTLP", &config.observability.otlp_endpoint);
    }
    if config.donation.enabled {
        ui::status(
            "Donate",
            &format!("{} link(s)", config.donation.links.len()),
        );
    }

    Ok(())
}
