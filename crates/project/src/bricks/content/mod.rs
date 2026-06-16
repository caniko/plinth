//! Rich HTML content project brick with prose typography.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::ContentSection;

pub struct ContentBrick;

impl ProjectBrick for ContentBrick {
    fn name(&self) -> &'static str {
        "content"
    }
}
