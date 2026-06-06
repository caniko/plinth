use super::FeatureGrid;
use crate::render::{escape_text, id_attr};

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
                "<div class=\"{}\"><h3>{}</h3><p>{}</p></div>",
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
