use super::WorkflowSteps;
use crate::render::{escape_text, id_attr, section_heading};

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
                "<article class=\"workflow-step tartan-surface tartan-flow\"><span class=\"workflow-index\">{}</span><h3>{}</h3><p>{}</p></article>",
                idx + 1,
                escape_text(&step.title),
                escape_text(&step.description),
            )
        })
        .collect::<String>();
    format!(
        "<section{} class=\"workflow-steps\">{}<div class=\"workflow-list tartan-responsive-grid\">{}</div></section>",
        id_attr(workflow.id.as_deref()),
        section_heading(
            workflow.eyebrow.as_deref(),
            &workflow.heading,
            &workflow.intro
        ),
        steps
    )
}
