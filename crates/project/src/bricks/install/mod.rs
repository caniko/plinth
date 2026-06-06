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

pub struct InstallBrick;

impl ProjectBrick for InstallBrick {
    fn name(&self) -> &'static str {
        "install"
    }
}
