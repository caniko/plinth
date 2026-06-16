/// Model for a numbered workflow steps section.
///
/// Rendered as `<section class="workflow-steps">` with an ordered
/// list of [`WorkflowStep`] articles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowSteps {
    /// Optional `id` on the wrapping `<section>`.
    pub id: Option<String>,
    /// Section heading (`<h2>`).
    pub heading: String,
    /// Intro paragraph text.
    pub intro: String,
    /// Ordered workflow steps.
    pub steps: Vec<WorkflowStep>,
}

/// A single step in a [`WorkflowSteps`] section.
///
/// Rendered as `<article class="workflow-step">` with an
/// auto-incremented index (`<span class="workflow-index">`),
/// title (`<h3>`), and body (`<p>`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowStep {
    /// Step heading (`<h3>`).
    pub title: String,
    /// Step body text (`<p>`).
    pub description: String,
}
