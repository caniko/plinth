use super::TrustPanel;
use crate::render::{escape_text, id_attr, section_heading};

/// Render a [`TrustPanel`] into an HTML string.
///
/// Template: `<section class="trust-panel">` →
/// `<div class="trust-list">` →
/// `<article class="trust-item">` with `<h3>` and `<p>` per entry.
pub fn render_trust_panel(panel: &TrustPanel) -> String {
    let items = panel
        .items
        .iter()
        .map(|item| {
            format!(
                "<article class=\"trust-item\"><h3>{}</h3><p>{}</p></article>",
                escape_text(&item.title),
                escape_text(&item.description),
            )
        })
        .collect::<String>();
    format!(
        "<section{} class=\"trust-panel\">{}<div class=\"trust-list tartan-responsive-grid\">{}</div></section>",
        id_attr(panel.id.as_deref()),
        section_heading(panel.eyebrow.as_deref(), &panel.heading, &panel.intro),
        items
    )
}
