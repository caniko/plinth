use serde::Deserialize;

use super::{InstallRoute, InstallSection};

/// Deserialized config for a single install route card.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallRouteConfig {
    /// Display name for this route (e.g. "macOS (Homebrew)").
    pub label: String,
    /// User segment name (e.g. "Linux", "Docker").
    pub audience: String,
    /// Optional shell command shown in a copyable `<pre><code>` block.
    #[serde(default)]
    pub command: Option<String>,
    /// Link to the full install guide (should include an anchor fragment).
    pub href: String,
    /// When `true`, the route card gets a "Recommended" badge.
    #[serde(default)]
    pub recommended: bool,
}

/// Build an [`InstallSection`] model from deserialized config.
///
/// Template markers: `<section class="install-section">`,
/// `<article class="install-route">`, `<div class="command-row">`,
/// `<button class="copy-command">`.
pub fn build_install_section(
    id: String,
    heading: String,
    intro: String,
    guide_href: String,
    primary_routes: Vec<InstallRouteConfig>,
    secondary_routes: Vec<InstallRouteConfig>,
) -> InstallSection {
    InstallSection {
        id,
        heading,
        intro,
        guide_href,
        primary_routes: primary_routes
            .into_iter()
            .map(build_install_route)
            .collect(),
        secondary_routes: secondary_routes
            .into_iter()
            .map(build_install_route)
            .collect(),
    }
}

fn build_install_route(config: InstallRouteConfig) -> InstallRoute {
    let route = InstallRoute::new(config.label, config.audience, config.href);
    let route = if let Some(command) = config.command {
        route.command(command)
    } else {
        route
    };
    if config.recommended {
        route.recommended()
    } else {
        route
    }
}
