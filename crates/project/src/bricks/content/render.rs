use super::ContentSection;
use crate::render::{escape_attr, escape_text};

/// Render a [`ContentSection`] into an HTML string.
///
/// Template: `<section class="content">` → optional `<h2>` →
/// `<div class="prose">` with raw HTML body.
pub fn render_content(section: &ContentSection) -> String {
    let heading = section
        .heading
        .as_deref()
        .filter(|h| !h.is_empty())
        .map(|h| format!("<h2>{}</h2>", escape_text(h)))
        .unwrap_or_default();
    format!(
        r#"<section class="content"{}><div class="prose">{}{}</div></section>"#,
        if section.id.is_empty() {
            String::new()
        } else {
            format!(" id=\"{}\"", escape_attr(&section.id))
        },
        heading,
        section.html,
    )
}
