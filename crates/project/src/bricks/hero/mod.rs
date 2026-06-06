//! Hero project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::{Cta, Hero};

pub struct HeroBrick;

impl ProjectBrick for HeroBrick {
    fn name(&self) -> &'static str {
        "hero"
    }
}
