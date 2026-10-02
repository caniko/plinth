use super::FeatureGrid;
use crate::render::{escape_attr, escape_text, id_attr, section_heading};

/// Render a [`FeatureGrid`] into an HTML string.
///
/// Template: `<section class="features">` →
/// `<div class="features-grid">` →
/// `<div class="feature-card">` (with optional `highlight` class) per entry.
/// Each card title is an `<h2>` so a feature grid can follow the page heading
/// without skipping a heading level when it has no section title of its own.
pub fn render_feature_grid(grid: &FeatureGrid) -> String {
    let level = if grid.heading.is_some() { "h3" } else { "h2" };
    let heading = grid.heading.as_ref().map_or_else(String::new, |heading| {
        section_heading(grid.eyebrow.as_deref(), heading, &grid.intro)
    });
    let cards = grid
        .features
        .iter()
        .enumerate()
        .map(|(index, feature)| {
            let class = if feature.highlight {
                "feature-card tartan-surface tartan-flow highlight"
            } else {
                "feature-card tartan-surface tartan-flow"
            };
            let action = feature.action.as_ref().map_or_else(String::new, |action| format!(
                "<div class=\"tartan-action-group\"><a class=\"btn {}\" href=\"{}\"><span>{}</span></a></div>",
                if action.primary { "btn-primary" } else { "btn-secondary" },
                escape_attr(&action.href), escape_text(&action.label),
            ));
            format!(
                "<div class=\"{}\"><span class=\"feature-index\" aria-hidden=\"true\">{:02}</span><{level}>{}</{level}><p>{}</p>{action}</div>",
                class,
                index + 1,
                escape_text(&feature.title),
                escape_text(&feature.description)
            )
        })
        .collect::<String>();
    format!(
        "<section{} class=\"features\">{heading}<div class=\"features-grid tartan-responsive-grid\">{}</div></section>",
        id_attr(grid.id.as_deref()),
        cards
    )
}
