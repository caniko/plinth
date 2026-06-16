use super::WorkflowSteps;
use crate::render::{escape_text, id_attr};

/// Render a [`WorkflowSteps`] into an HTML string.
///
/// Template: `<section class="workflow-steps">` →
/// `<div class="workflow-list">` →
/// `<article class="workflow-step">` with
/// `<span class="workflow-index">` (1-based), `<h3>`, and `<p>` per step.
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
