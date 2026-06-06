use serde::Deserialize;

use super::{InstallRoute, InstallSection};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallRouteConfig {
    pub label: String,
    pub audience: String,
    #[serde(default)]
    pub command: Option<String>,
    pub href: String,
    #[serde(default)]
    pub recommended: bool,
}

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
