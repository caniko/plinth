use super::ContentSection;
use crate::render::{escape_attr, escape_text};

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
