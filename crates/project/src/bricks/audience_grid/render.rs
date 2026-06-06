use super::AudienceGrid;
use crate::render::{escape_text, id_attr};

pub fn render_audience_grid(grid: &AudienceGrid) -> String {
    let audiences = grid
        .audiences
        .iter()
        .map(|audience| {
            format!(
                "<article class=\"audience-card\"><h3>{}</h3><p>{}</p></article>",
                escape_text(&audience.label),
                escape_text(&audience.description),
            )
        })
        .collect::<String>();
    format!(
        "<section{} class=\"audience-grid\"><div class=\"section-heading\"><h2>{}</h2><p>{}</p></div><div class=\"audience-list\">{}</div></section>",
        id_attr(grid.id.as_deref()),
        escape_text(&grid.heading),
        escape_text(&grid.intro),
        audiences
    )
}
