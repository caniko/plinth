use serde::Deserialize;

use super::{InstallRoute, InstallSection, Platform};

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
    /// Link target for this route.
    pub href: String,
    /// Optional text under the route title.
    #[serde(default)]
    pub note: Option<String>,
    /// Optional link text; defaults to "Open guide".
    #[serde(default)]
    pub link_label: Option<String>,
    /// When `true`, the route card gets a "Recommended" badge.
    #[serde(default)]
    pub recommended: bool,
    /// Platforms this route installs on (`linux`, `macos`, `windows`).
    #[serde(default)]
    pub platforms: Vec<Platform>,
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
    platforms: Vec<Platform>,
    primary_routes: Vec<InstallRouteConfig>,
    secondary_routes: Vec<InstallRouteConfig>,
) -> InstallSection {
    InstallSection {
        id,
        heading,
        intro,
        guide_href,
        guide_label: None,
        platforms,
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
    InstallRoute {
        label: config.label,
        audience: config.audience,
        command: config.command,
        href: config.href,
        note: config.note,
        link_label: config.link_label,
        recommended: config.recommended,
        platforms: config.platforms,
    }
}
