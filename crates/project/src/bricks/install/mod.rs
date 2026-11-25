//! Install section project brick.

pub mod config;
pub mod diagnostics;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use diagnostics::{
    InstallRouteUxFinding, InstallUxReport, build_install_ux_report, validate_install_section,
};
pub use model::{InstallRoute, InstallSection, Platform};
pub use render::render_install_fragment;

/// Brick that renders an install / getting-started section.
///
/// Displays a `<section class="install-section">` with primary
/// and secondary install routes (`<article class="install-route">`),
/// each with an audience label, optional note and copyable command
/// (`<pre><code>` + copy button), and a link with customizable text.
pub struct InstallBrick;

impl ProjectBrick for InstallBrick {
    fn name(&self) -> &'static str {
        "install"
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn install_copy_uses_configured_text_or_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("plinth-project.toml");
        for (guide_label, route_copy, label) in [
            ("", "", "Open full install guide"),
            (
                "guide_label = '   '",
                "note = '   '\nlink_label = '   '",
                "Open full install guide",
            ),
            ("guide_label = 'All & sundry'", "", "All &amp; sundry"),
        ] {
            std::fs::write(
                &config,
                format!(
                    r#"
[site]
title = "Example"
description = "Example site"
[[pages]]
slug = "index"
title = "Example"
[[pages.sections]]
type = "install"
id = "install"
heading = "Install"
intro = "Pick a route."
guide_href = "/docs/install.html"
{guide_label}
[[pages.sections.primary_routes]]
label = "Package"
audience = "Linux"
href = "/package"
recommended = true
platforms = ["linux", "macos"]
{route_copy}
[[pages.sections.primary_routes]]
label = "Archive"
audience = "Linux"
href = "/archive"
note = "Use <stable>"
link_label = "Download & save"
[[pages.sections.secondary_routes]]
label = "Source"
audience = "Developers"
href = "/source"
[[pages.sections.secondary_routes]]
label = "Manual"
audience = "Developers"
href = "/manual"
note = "Advanced setup"
link_label = "Read instructions"
"#
                ),
            )
            .unwrap();
            let site = crate::load_project_site(&config).unwrap();
            let crate::ProjectSection::Install(install) = &site.pages[0].sections[0] else {
                panic!("expected an install section");
            };
            let html = super::render_install_fragment(install);
            assert!(html.contains(&format!("<a href=\"/docs/install.html\">{label}</a>")));
            assert!(html.contains("data-platforms=\"linux macos\""));
            assert!(html.contains(">Recommended</span>"));
            assert!(html.contains(
                "<strong>Package</strong><a class=\"route-link\" href=\"/package\">Open guide</a>"
            ));
            assert!(html.contains("<p>Use &lt;stable&gt;</p><a class=\"route-link\" href=\"/archive\">Download &amp; save</a>"));
            assert!(html.contains(
                "<strong>Source</strong><a class=\"route-link\" href=\"/source\">Open guide</a>"
            ));
            assert!(html.contains("<p>Advanced setup</p><a class=\"route-link\" href=\"/manual\">Read instructions</a>"));
        }
    }
}
