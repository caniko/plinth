use serde::Deserialize;

use super::{Audience, AudienceGrid};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudienceConfig {
    pub label: String,
    pub description: String,
}

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
