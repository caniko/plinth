use serde::Deserialize;

use super::{Feature, FeatureGrid};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureConfig {
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub highlight: bool,
}

pub fn build_feature_grid(id: Option<String>, features: Vec<FeatureConfig>) -> FeatureGrid {
    FeatureGrid {
        id,
        features: features
            .into_iter()
            .map(|feature| {
                let built = Feature::new(feature.title, feature.description);
                if feature.highlight {
                    built.highlight()
                } else {
                    built
                }
            })
            .collect(),
    }
}
