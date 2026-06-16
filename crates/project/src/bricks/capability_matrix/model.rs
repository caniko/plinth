/// Model for a capability / compatibility matrix section.
///
/// Rendered as `<section class="landing-content">` with an HTML
/// intro and a `<table class="games-matrix">`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityMatrix {
    /// `id` attribute on the wrapping `<section>`.
    pub id: String,
    /// Section heading (`<h2>`).
    pub heading: String,
    /// Pre-rendered HTML intro paragraph (may contain links).
    pub intro_html: String,
    /// Table rows, one per game / item.
    pub capabilities: Vec<Capability>,
}

/// A single row in the capability matrix.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capability {
    /// Unique slug used as the TOML key.
    pub slug: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Overall rating / support-tier label.
    pub overall: String,
    /// Per-capability detail pills as `(label, value)` pairs.
    pub details: Vec<(String, String)>,
}
