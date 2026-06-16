/// Model for a rich-HTML content section.
///
/// Rendered as `<section class="content">` with an optional
/// heading and a `<div class="prose">` body that may contain
/// arbitrary pre-rendered HTML.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentSection {
    /// `id` attribute on the wrapping `<section>`.
    pub id: String,
    /// Optional section heading (`<h2>`). Omitted when `None` or empty.
    pub heading: Option<String>,
    /// Pre-rendered HTML injected into `<div class="prose">`.
    pub html: String,
}
