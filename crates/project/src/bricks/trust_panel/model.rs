/// Model for a trust / rights / posture panel section.
///
/// Rendered as `<section class="trust-panel">` with heading, intro,
/// and a list of [`TrustItem`] cards.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPanel {
    /// Optional `id` on the wrapping `<section>`.
    pub id: Option<String>,
    /// Section heading (`<h2>`).
    pub heading: String,
    /// Intro paragraph text.
    pub intro: String,
    /// Trust item cards.
    pub items: Vec<TrustItem>,
}

/// A single trust item card within a [`TrustPanel`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustItem {
    /// Item heading (`<h3>`).
    pub title: String,
    /// Item body text (`<p>`).
    pub description: String,
}
