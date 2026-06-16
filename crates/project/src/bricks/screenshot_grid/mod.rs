//! Screenshot grid project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::{Screenshot, ScreenshotGrid};

/// Brick that renders a screenshot gallery grid.
///
/// Displays a `<section class="landing-content">` with heading, intro,
/// and a `<div class="screenshots-grid">` of `<figure class="screenshot-slot">`
/// elements.  Each screenshot has a lightbox trigger button
/// (`<button class="lightbox-trigger">`) for full-size viewing.
pub struct ScreenshotGridBrick;

impl ProjectBrick for ScreenshotGridBrick {
    fn name(&self) -> &'static str {
        "screenshot_grid"
    }
}
