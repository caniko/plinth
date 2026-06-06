use serde::Deserialize;

use super::{Screenshot, ScreenshotGrid};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScreenshotConfig {
    pub src: String,
    pub alt: String,
    pub caption: String,
}

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
