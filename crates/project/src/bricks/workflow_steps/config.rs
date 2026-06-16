use serde::Deserialize;

use super::{WorkflowStep, WorkflowSteps};

/// Deserialized config for a single workflow step.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowStepConfig {
    /// Step heading (e.g. "Install", "Configure").
    pub title: String,
    /// Step description body.
    pub description: String,
}

/// Build a [`WorkflowSteps`] model from deserialized config.
///
/// Template markers: `<section class="workflow-steps">`,
/// `<div class="workflow-list">`, `<article class="workflow-step">`.
pub fn build_workflow_steps(
    id: Option<String>,
    heading: String,
    intro: String,
    steps: Vec<WorkflowStepConfig>,
) -> WorkflowSteps {
    WorkflowSteps {
        id,
        heading,
        intro,
        steps: steps
            .into_iter()
            .map(|step| WorkflowStep {
                title: step.title,
                description: step.description,
            })
            .collect(),
    }
}
