/// A single call-to-action button in the hero section.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cta {
    /// Button label text.
    pub label: String,
    /// Link destination URL.
    pub href: String,
    /// When `true`, renders with the `btn-primary` CSS class.
    pub primary: bool,
}

impl Cta {
    /// Create a primary (accent) CTA button.
    #[must_use]
    pub fn primary(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: href.into(),
            primary: true,
        }
    }

    /// Create a secondary (outline) CTA button.
    #[must_use]
    pub fn secondary(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: href.into(),
            primary: false,
        }
    }
}

/// Model for the page hero section.
///
/// Rendered as `<section class="hero">` with optional logo,
/// title (`<h1>`), tagline, subtitle, byline, and action buttons.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hero {
    /// Optional URL for the hero logo image (`<img class="hero-logo">`).
    pub logo_src: Option<String>,
    /// Hero title (`<h1>`).
    pub title: String,
    /// Tagline text (`<p class="tagline">`).
    pub tagline: String,
    /// Subtitle text (`<p class="subtitle">`).
    pub subtitle: String,
    /// Optional person identifier for the byline (`<p class="hero-byline">`).
    pub person: Option<String>,
    /// Call-to-action buttons displayed in `<div class="hero-actions">`.
    pub ctas: Vec<Cta>,
}
