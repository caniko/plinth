use super::FeatureGrid;
use crate::render::{escape_text, id_attr};

/// Render a [`FeatureGrid`] into an HTML string.
///
/// Template: `<section class="features">` →
/// `<div class="features-grid">` →
/// `<div class="feature-card">` (with optional `highlight` class) per entry.
/// Each card title is an `<h2>` so a feature grid can follow the page heading
/// without skipping a heading level when it has no section title of its own.
pub fn render_feature_grid(grid: &FeatureGrid) -> String {
    let cards = grid
        .features
        .iter()
        .map(|feature| {
            let class = if feature.highlight {
                "feature-card highlight"
            } else {
                "feature-card"
            };
            format!(
                "<div class=\"{}\"><h2>{}</h2><p>{}</p></div>",
                class,
                escape_text(&feature.title),
                escape_text(&feature.description)
            )
        })
        .collect::<String>();
    format!(
        "<section{} class=\"features\"><div class=\"features-grid\">{}</div></section>",
        id_attr(grid.id.as_deref()),
        cards
    )
}
