//! Feature grid project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::{Feature, FeatureGrid};

/// Brick that renders a features grid section.
///
/// Displays a `<section class="features">` with a CSS grid of
/// `<div class="feature-card">` (optionally `.highlight`) cards,
/// each containing a title (`<h2>`) and description (`<p>`).
pub struct FeatureGridBrick;

impl ProjectBrick for FeatureGridBrick {
    fn name(&self) -> &'static str {
        "feature_grid"
    }
}
