use serde::Deserialize;

use super::{Audience, AudienceGrid};

/// Deserialized config for a single audience entry in a TOML project file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudienceConfig {
    /// Short display label for the audience segment.
    pub label: String,
    /// One-line description of who this audience is.
    pub description: String,
}

/// Build an [`AudienceGrid`] model from deserialized config values.
///
/// Template markers: `<section class="audience-grid">`,
/// `<div class="audience-list">`.
pub fn build_audience_grid(
    id: Option<String>,
    heading: String,
    intro: String,
    audiences: Vec<AudienceConfig>,
) -> AudienceGrid {
    AudienceGrid {
        id,
        heading,
        intro,
        audiences: audiences
            .into_iter()
            .map(|audience| Audience {
                label: audience.label,
                description: audience.description,
            })
            .collect(),
    }
}
