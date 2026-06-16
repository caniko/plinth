/// Model for a comparison / coverage table section.
///
/// Rendered as `<section class="comparison-section">` with a
/// `<table class="coverage-table">`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComparisonSection {
    /// Optional `id` on the wrapping `<section>`.
    pub id: Option<String>,
    /// Section heading (`<h2>`).
    pub heading: String,
    /// Subtitle text (`<p class="section-subtitle">`).
    pub intro: String,
    /// Comparison rows for the table body.
    pub rows: Vec<ComparisonRow>,
}

/// A single comparison row within a [`ComparisonSection`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComparisonRow {
    /// The area or feature being compared.
    pub area: String,
    /// Status string (mapped to `badge low|mid|high` CSS class).
    pub status: String,
    /// Notes explaining the status.
    pub notes: String,
}
