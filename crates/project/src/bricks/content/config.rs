use super::ContentSection;

/// Build a [`ContentSection`] model from raw parts.
///
/// The `html` argument is treated as pre-rendered / trusted HTML
/// and injected directly into `<div class="prose">`.
pub fn build_content(id: String, heading: Option<String>, html: String) -> ContentSection {
    ContentSection { id, heading, html }
}
