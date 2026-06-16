//! Custom rendered section project brick.

pub mod model;

use super::ProjectBrick;

pub use model::CustomSection;

/// Brick that wraps a caller-provided render closure as a project section.
///
/// Unlike other bricks, `CustomBrick` does not define a fixed config or
/// template.  The caller supplies an `Arc<dyn Fn() -> String>` via
/// [`CustomSection::new()`], and the render output is placed directly into
/// the page.  Useful for embeddable widgets, external content, or
/// experiments that don't have a dedicated brick yet.
pub struct CustomBrick;

impl ProjectBrick for CustomBrick {
    fn name(&self) -> &'static str {
        "custom"
    }
}
