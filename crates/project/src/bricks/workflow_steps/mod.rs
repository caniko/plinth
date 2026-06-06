//! Ordered workflow project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::{WorkflowStep, WorkflowSteps};

pub struct WorkflowStepsBrick;

impl ProjectBrick for WorkflowStepsBrick {
    fn name(&self) -> &'static str {
        "workflow_steps"
    }
}
