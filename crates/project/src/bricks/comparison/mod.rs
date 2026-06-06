//! Comparison table project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::{ComparisonRow, ComparisonSection};

pub struct ComparisonBrick;

impl ProjectBrick for ComparisonBrick {
    fn name(&self) -> &'static str {
        "comparison"
    }
}
