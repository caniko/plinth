use serde::Deserialize;

use super::{Cta, Hero};

/// Deserialized config for a single hero call-to-action button.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CtaConfig {
    /// Button label text.
    pub label: String,
    /// Link destination URL.
    pub href: String,
    /// When `true`, renders as `<a class="btn btn-primary">`.
    #[serde(default)]
    pub primary: bool,
}

/// Build a [`Hero`] model from deserialized config values.
///
/// Template markers: `<section class="hero">`, `<img class="hero-logo">`,
/// `<a class="btn btn-primary|secondary">`.
pub fn build_hero(
    logo_src: Option<String>,
    title: String,
    tagline: String,
    subtitle: String,
    person: Option<String>,
    ctas: Vec<CtaConfig>,
) -> Hero {
    Hero {
        logo_src,
        title,
        tagline,
        subtitle,
        person,
        ctas: ctas
            .into_iter()
            .map(|cta| {
                if cta.primary {
                    Cta::primary(cta.label, cta.href)
                } else {
                    Cta::secondary(cta.label, cta.href)
                }
            })
            .collect(),
    }
}
