/// Model for a features grid section.
///
/// Rendered as `<section class="features">` with a
/// `<div class="features-grid">` of [`Feature`] cards.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeatureGrid {
    /// Optional `id` on the wrapping `<section>`.
    pub id: Option<String>,
    /// Feature cards to display.
    pub features: Vec<Feature>,
}

/// A single feature card within a [`FeatureGrid`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Feature {
    /// Card heading (`<h3>`).
    pub title: String,
    /// Card body text (`<p>`).
    pub description: String,
    /// When `true`, the card gets an additional `highlight` CSS class.
    pub highlight: bool,
}

impl Feature {
    /// Create a new feature card with the given title and description.
    ///
    /// `highlight` defaults to `false`; use [`Self::highlight`] to enable it.
    #[must_use]
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
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
