use serde::Deserialize;

use super::{TrustItem, TrustPanel};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustItemConfig {
    pub title: String,
    pub description: String,
}

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
