//! Capability matrix project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::{Capability, CapabilityMatrix};

pub struct CapabilityMatrixBrick;

impl ProjectBrick for CapabilityMatrixBrick {
    fn name(&self) -> &'static str {
        "capability_matrix"
    }
}
