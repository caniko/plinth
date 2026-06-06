use serde::Deserialize;

use super::{Cta, Hero};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CtaConfig {
    pub label: String,
    pub href: String,
    #[serde(default)]
    pub primary: bool,
}

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
