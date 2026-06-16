//! Comparison table project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::{ComparisonRow, ComparisonSection};

/// Brick that renders a comparison / coverage table section.
///
/// Displays a `<section class="comparison-section">` with a heading,
/// subtitle, and `<table class="coverage-table">` whose rows show
/// area, status badge (`<span class="badge low/mid/high">`), and notes.
pub struct ComparisonBrick;

impl ProjectBrick for ComparisonBrick {
    fn name(&self) -> &'static str {
        "comparison"
    }
}
