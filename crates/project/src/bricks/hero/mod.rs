//! Hero project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::{Cta, Hero};

/// Brick that renders the page hero section.
///
/// Displays a `<section class="hero">` with an optional logo image,
/// title, tagline, subtitle, optional byline from a person reference,
/// and action buttons (`<a class="btn btn-primary|secondary">`).
pub struct HeroBrick;

impl ProjectBrick for HeroBrick {
    fn name(&self) -> &'static str {
        "hero"
    }
}
