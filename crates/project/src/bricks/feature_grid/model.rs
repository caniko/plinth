/// Model for a features grid section.
///
/// Rendered as `<section class="features">` with a
/// `<div class="features-grid">` of [`Feature`] cards.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FeatureGrid {
    /// Optional short label above the section heading.
    pub eyebrow: Option<String>,
    /// Optional heading; when present the cards use subordinate h3 headings.
    pub heading: Option<String>,
    /// Supporting copy beneath the section heading.
    pub intro: String,
    /// Optional `id` on the wrapping `<section>`.
    pub id: Option<String>,
    /// Feature cards to display.
    pub features: Vec<Feature>,
}

/// A single feature card within a [`FeatureGrid`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Feature {
    /// Optional task-entry link, using the same CTA model as the hero.
    pub action: Option<FeatureAction>,
    /// Card heading (`<h2>`).
    pub title: String,
    /// Card body text (`<p>`).
    pub description: String,
    /// When `true`, the card gets an additional `highlight` CSS class.
    pub highlight: bool,
}

/// An optional task-entry action on a feature card.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureAction {
    /// Visible label for the link.
    pub label: String,
    /// Host-owned destination.
    pub href: String,
    /// Accent treatment for the primary entry point.
    #[serde(default)]
    pub primary: bool,
}

impl Feature {
    /// Create a new feature card with the given title and description.
    ///
    /// `highlight` defaults to `false`; use [`Self::highlight`] to enable it.
    #[must_use]
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            action: None,
            title: title.into(),
            description: description.into(),
            highlight: false,
        }
    }

    /// Mark this feature card as highlighted (builder style).
    #[must_use]
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }
}
