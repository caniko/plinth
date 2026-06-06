//! Screenshot grid project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::{Screenshot, ScreenshotGrid};

pub struct ScreenshotGridBrick;

impl ProjectBrick for ScreenshotGridBrick {
    fn name(&self) -> &'static str {
        "screenshot_grid"
    }
}
