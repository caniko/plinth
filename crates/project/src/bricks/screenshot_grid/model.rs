/// Model for a screenshot gallery grid section.
///
/// Rendered as `<section class="landing-content">` with a
/// `<div class="screenshots-grid">` of [`Screenshot`] figures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScreenshotGrid {
    /// `id` attribute on the wrapping `<section>`.
    pub id: String,
    /// Section heading (`<h2>`).
    pub heading: String,
    /// Intro paragraph text.
    pub intro: String,
    /// Screenshot entries for the grid.
    pub screenshots: Vec<Screenshot>,
}

/// A single screenshot entry in a [`ScreenshotGrid`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Screenshot {
    /// Image source URL.
    pub src: String,
    /// Alt text for accessibility.
    pub alt: String,
    /// Caption shown below the image in `<figcaption>`.
    pub caption: String,
}
