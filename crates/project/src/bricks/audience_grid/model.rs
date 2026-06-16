/// Model for the audience-selection grid section.
///
/// Rendered as `<section class="audience-grid">` with a heading,
/// intro paragraph, and a list of [`Audience`] cards.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudienceGrid {
    /// Optional `id` attribute on the wrapping `<section>`.
    pub id: Option<String>,
    /// Section heading (`<h2>`).
    pub heading: String,
    /// Introductory paragraph below the heading.
    pub intro: String,
    /// Audience cards to display.
    pub audiences: Vec<Audience>,
}

/// A single audience-segment card within an [`AudienceGrid`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Audience {
    /// Short display label shown in `<h3>`.
    pub label: String,
    /// One-line description shown in `<p>`.
    pub description: String,
}
