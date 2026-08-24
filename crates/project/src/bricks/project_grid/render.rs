use plinth_person::ProjectReference;

use super::ProjectGrid;
use crate::render::{escape_attr, escape_text, external_attrs, id_attr, render_external_link};

/// Render a [`ProjectGrid`] into an HTML string.
///
/// Template: `<section class="project-grid">` →
/// `<div class="section-heading">` (h2, p) →
/// `<div class="project-list">` →
/// `<article class="project-card">` per project.
pub fn render_project_grid(grid: &ProjectGrid, projects: &[ProjectReference]) -> String {
    let cards = projects.iter().map(render_project_card).collect::<String>();
    format!(
        "<section{} class=\"project-grid\"><div class=\"section-heading\"><h2>{}</h2><p>{}</p></div><div class=\"project-list\">{}</div></section>",
        id_attr(grid.id.as_deref()),
        escape_text(&grid.heading),
        escape_text(&grid.intro),
        cards
    )
}

fn render_project_card(project: &ProjectReference) -> String {
    let description = project
        .description
        .as_deref()
        .filter(|text| !text.is_empty());
    let description_html =
        description.map_or_else(String::new, |text| format!("<p>{}</p>", escape_text(text)));
    let links = project
        .links()
        .iter()
        .map(render_external_link)
        .collect::<String>();
    format!(
        "<article class=\"project-card\"><h3><a href=\"{}\"{}>{}</a></h3>{}<div class=\"project-links\">{}</div></article>",
        escape_attr(&project.url),
        external_attrs(&project.url),
        escape_text(&project.title),
        description_html,
        links
    )
}
