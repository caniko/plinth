//! Install section project brick.

pub mod config;
pub mod diagnostics;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use diagnostics::{
    InstallRouteUxFinding, InstallUxReport, build_install_ux_report, validate_install_section,
};
pub use model::{InstallRoute, InstallSection};
pub use render::render_install_fragment;

/// Brick that renders an install / getting-started section.
///
/// Displays a `<section class="install-section">` with primary
/// and secondary install routes (`<article class="install-route">`),
/// each with an audience label, optional copyable command
/// (`<pre><code>` + copy button), and a "Open guide" link.
pub struct InstallBrick;

impl ProjectBrick for InstallBrick {
    fn name(&self) -> &'static str {
        "install"
    }
}
