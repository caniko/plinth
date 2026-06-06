use super::WorkflowSteps;
use crate::render::{escape_text, id_attr};

pub fn render_workflow_steps(workflow: &WorkflowSteps) -> String {
    let steps = workflow
        .steps
        .iter()
        .enumerate()
        .map(|(idx, step)| {
            format!(
                "<article class=\"workflow-step\"><span class=\"workflow-index\">{}</span><h3>{}</h3><p>{}</p></article>",
                idx + 1,
                escape_text(&step.title),
                escape_text(&step.description),
            )
        })
        .collect::<String>();
    format!(
        "<section{} class=\"workflow-steps\"><div class=\"section-heading\"><h2>{}</h2><p>{}</p></div><div class=\"workflow-list\">{}</div></section>",
        id_attr(workflow.id.as_deref()),
        escape_text(&workflow.heading),
        escape_text(&workflow.intro),
        steps
    )
}
