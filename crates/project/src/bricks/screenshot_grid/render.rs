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
            let img = format!(
                "<img src=\"{}\" alt=\"{}\" loading=\"lazy\">",
                escape_attr(&screenshot.src),
                escape_attr(&screenshot.alt)
            );
            let img = match &screenshot.still {
                Some(still) => format!(
                    "<picture><source media=\"(prefers-reduced-motion: reduce)\" srcset=\"{}\">{img}</picture>",
                    escape_attr(still)
                ),
                None => img,
            };
            format!(
                "<figure class=\"screenshot-slot\"><button type=\"button\" class=\"lightbox-trigger\" data-lightbox-image=\"{}\" data-lightbox-alt=\"{}\" data-lightbox-caption=\"{}\">{img}</button><figcaption>{}</figcaption></figure>",
                escape_attr(&screenshot.src),
                escape_attr(&screenshot.alt),
                escape_attr(&screenshot.caption),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Screenshot;

    fn grid(still: Option<&str>) -> ScreenshotGrid {
        ScreenshotGrid {
            id: "shots".into(),
            heading: "Shots".into(),
            intro: "Intro".into(),
            screenshots: vec![Screenshot {
                src: "/a.webp".into(),
                alt: "A".into(),
                caption: "Cap".into(),
                still: still.map(str::to_owned),
            }],
        }
    }

    #[test]
    fn still_wraps_img_in_reduced_motion_picture() {
        let html = render_screenshots(&grid(Some("/a-still.png")));
        assert!(html.contains(
            "<picture><source media=\"(prefers-reduced-motion: reduce)\" srcset=\"/a-still.png\"><img src=\"/a.webp\" alt=\"A\" loading=\"lazy\"></picture>"
        ));
    }

    #[test]
    fn no_still_keeps_plain_img() {
        let html = render_screenshots(&grid(None));
        assert!(!html.contains("<picture"));
        assert!(html.contains("<img src=\"/a.webp\" alt=\"A\" loading=\"lazy\">"));
    }
}
