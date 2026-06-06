//! Feature grid project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::{Feature, FeatureGrid};

pub struct FeatureGridBrick;

impl ProjectBrick for FeatureGridBrick {
    fn name(&self) -> &'static str {
        "feature_grid"
    }
}
