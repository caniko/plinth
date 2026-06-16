//! Rich HTML content project brick with prose typography.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::ContentSection;

/// Brick that renders an arbitrary rich-HTML content section.
///
/// Outputs `<section class="content">` with an optional heading
/// (`<h2>`) and pre-rendered HTML wrapped in `<div class="prose">`.
/// Use when markdown or raw HTML should appear as-is in the page.
pub struct ContentBrick;

impl ProjectBrick for ContentBrick {
    fn name(&self) -> &'static str {
        "content"
    }
}
