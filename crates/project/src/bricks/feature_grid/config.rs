use serde::Deserialize;

use super::{Feature, FeatureGrid};

/// Deserialized config for a single feature card.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureConfig {
    /// Optional link turning the card into a task entry point.
    #[serde(default)]
    pub action: Option<super::FeatureAction>,
    /// Card heading.
    pub title: String,
    /// Card body text.
    pub description: String,
    /// When `true`, adds CSS class `highlight` to the card.
    #[serde(default)]
    pub highlight: bool,
}

/// Build a [`FeatureGrid`] model from deserialized config.
///
/// Template markers: `<section class="features">`,
/// `<div class="features-grid">`.
pub fn build_feature_grid(id: Option<String>, features: Vec<FeatureConfig>) -> FeatureGrid {
    FeatureGrid {
        eyebrow: None,
        heading: None,
        intro: String::new(),
        id,
        features: features
            .into_iter()
            .map(|feature| {
                let mut built = Feature::new(feature.title, feature.description);
                built.action = feature.action;
                if feature.highlight {
                    built.highlight()
                } else {
                    built
                }
            })
            .collect(),
    }
}
