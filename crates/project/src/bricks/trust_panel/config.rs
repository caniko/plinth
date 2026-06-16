use serde::Deserialize;

use super::{TrustItem, TrustPanel};

/// Deserialized config for a single trust / policy item.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustItemConfig {
    /// Item title (e.g. "Data Privacy", "Open Source").
    pub title: String,
    /// Item description body.
    pub description: String,
}

/// Build a [`TrustPanel`] model from deserialized config.
///
/// Template markers: `<section class="trust-panel">`,
/// `<div class="trust-list">`, `<article class="trust-item">`.
pub fn build_trust_panel(
    id: Option<String>,
    heading: String,
    intro: String,
    items: Vec<TrustItemConfig>,
) -> TrustPanel {
    TrustPanel {
        id,
        heading,
        intro,
        items: items
            .into_iter()
            .map(|item| TrustItem {
                title: item.title,
                description: item.description,
            })
            .collect(),
    }
}
