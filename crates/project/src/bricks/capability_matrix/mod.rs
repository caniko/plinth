//! Capability matrix project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::{Capability, CapabilityMatrix};

/// Brick that renders a capability / compatibility matrix table.
///
/// Renders a `<section class="landing-content">` with a heading,
/// intro HTML, and a `<table class="games-matrix">` whose rows
/// show per-item display name, overall status pill, and a list
/// of capability pills.  Data is loaded from an external TOML file.
pub struct CapabilityMatrixBrick;

impl ProjectBrick for CapabilityMatrixBrick {
    fn name(&self) -> &'static str {
        "capability_matrix"
    }
}
