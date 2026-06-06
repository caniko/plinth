use serde::Deserialize;

use super::{WorkflowStep, WorkflowSteps};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowStepConfig {
    pub title: String,
    pub description: String,
}

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
