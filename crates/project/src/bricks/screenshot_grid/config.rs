use serde::Deserialize;

use super::{Screenshot, ScreenshotGrid};

/// Deserialized config for a single screenshot entry.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScreenshotConfig {
    /// Image source URL.
    pub src: String,
    /// Alt text for the image.
    pub alt: String,
    /// Caption displayed below the image.
    pub caption: String,
}

/// Build a [`ScreenshotGrid`] model from deserialized config.
///
/// Template markers: `<section class="landing-content">`,
/// `<div class="screenshots-grid">`,
/// `<button class="lightbox-trigger">`.
pub fn build_screenshot_grid(
    id: String,
    heading: String,
    intro: String,
    screenshots: Vec<ScreenshotConfig>,
) -> ScreenshotGrid {
    ScreenshotGrid {
        id,
        heading,
        intro,
        screenshots: screenshots
            .into_iter()
            .map(|screenshot| Screenshot {
                src: screenshot.src,
                alt: screenshot.alt,
                caption: screenshot.caption,
            })
            .collect(),
    }
}
