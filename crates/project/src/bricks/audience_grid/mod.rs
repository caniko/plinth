//! Audience and role project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::{Audience, AudienceGrid};

pub struct AudienceGridBrick;

impl ProjectBrick for AudienceGridBrick {
    fn name(&self) -> &'static str {
        "audience_grid"
    }
}
