use super::Hero;
use crate::render::{escape_attr, escape_text, external_attrs};

/// Render a [`Hero`] into an HTML string.
///
/// Template: `<section class="hero">` → optional `<img class="hero-logo">` →
/// `<h1>` → `<p class="tagline">` → `<p class="subtitle">` →
/// optional `<p class="hero-byline">` → `<div class="hero-actions">`
/// with `<a class="btn btn-primary|secondary">` buttons.
pub fn render_hero(hero: &Hero, person: Option<&plinth_person::PersonReference>) -> String {
    let logo = hero.logo_src.as_ref().map_or_else(String::new, |src| {
        format!(
            "<img src=\"{}\" alt=\"{} logo\" class=\"hero-logo\">",
            escape_attr(src),
            escape_attr(&hero.title)
        )
    });
    let ctas = hero
        .ctas
        .iter()
        .map(|cta| {
            let class = if cta.primary {
                "btn btn-primary"
            } else {
                "btn btn-secondary"
            };
            format!(
                "<a href=\"{}\" class=\"{}\">{}</a>",
                escape_attr(&cta.href),
                class,
                escape_text(&cta.label)
            )
        })
        .collect::<String>();
    let byline = person.map_or_else(String::new, |person| {
        format!(
            "<p class=\"hero-byline\">By <a href=\"{}\"{}>{}</a></p>",
            escape_attr(&person.url),
            external_attrs(&person.url),
            escape_text(&person.name)
        )
    });
    format!(
        "<section class=\"hero\">{}<h1>{}</h1><p class=\"tagline\">{}</p><p class=\"subtitle\">{}</p>{}<div class=\"hero-actions\">{}</div></section>",
        logo,
        escape_text(&hero.title),
        escape_text(&hero.tagline),
        escape_text(&hero.subtitle),
        byline,
        ctas
    )
}
