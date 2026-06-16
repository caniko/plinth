//! Ordered workflow project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::{WorkflowStep, WorkflowSteps};

/// Brick that renders an ordered workflow / how-it-works section.
///
/// Displays a `<section class="workflow-steps">` with heading, intro,
/// and a `<div class="workflow-list">` of
/// `<article class="workflow-step">` entries, each numbered with a
/// `<span class="workflow-index">`.
pub struct WorkflowStepsBrick;

impl ProjectBrick for WorkflowStepsBrick {
    fn name(&self) -> &'static str {
        "workflow_steps"
    }
}
