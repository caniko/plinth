//! Audience and role project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::{Audience, AudienceGrid};

/// Brick that renders an audience-segment grid.
///
/// Displays a heading, intro paragraph, and a list of audience cards
/// (`<article class="audience-card">`) wrapped in `<section class="audience-grid">`.
/// Each card shows a label (`<h3>`) and description (`<p>`).
pub struct AudienceGridBrick;

impl ProjectBrick for AudienceGridBrick {
    fn name(&self) -> &'static str {
        "audience_grid"
    }
}
