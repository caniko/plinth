use super::ScreenshotGrid;
use crate::render::{escape_attr, escape_text};

/// Render a [`ScreenshotGrid`] into an HTML string.
///
/// Template: `<section class="landing-content">` →
/// `<div class="screenshots-grid">` →
/// `<figure class="screenshot-slot">` with `<button class="lightbox-trigger">`
/// containing `<img>` and `<figcaption>`.
pub fn render_screenshots(grid: &ScreenshotGrid) -> String {
    let screenshots = grid
        .screenshots
        .iter()
        .map(|screenshot| {
            format!(
                "<figure class=\"screenshot-slot\"><button type=\"button\" class=\"lightbox-trigger\" data-lightbox-image=\"{}\" data-lightbox-alt=\"{}\" data-lightbox-caption=\"{}\"><img src=\"{}\" alt=\"{}\"></button><figcaption>{}</figcaption></figure>",
                escape_attr(&screenshot.src),
                escape_attr(&screenshot.alt),
                escape_attr(&screenshot.caption),
                escape_attr(&screenshot.src),
                escape_attr(&screenshot.alt),
                escape_text(&screenshot.caption)
            )
        })
        .collect::<String>();
    format!(
        "<section id=\"{}\" class=\"landing-content\"><h2>{}</h2><p>{}</p><div class=\"screenshots-grid\">{}</div></section>",
        escape_attr(&grid.id),
        escape_text(&grid.heading),
        escape_text(&grid.intro),
        screenshots
    )
}
