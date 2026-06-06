//! Custom rendered section project brick.

pub mod model;

use super::ProjectBrick;

pub use model::CustomSection;

pub struct CustomBrick;

impl ProjectBrick for CustomBrick {
    fn name(&self) -> &'static str {
        "custom"
    }
}
