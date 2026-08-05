use super::Hero;
use crate::render::{escape_attr, escape_text, external_attrs, render_inline_text};

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
    let subtitle = if hero.subtitle.trim().is_empty() || hero.subtitle.trim() == hero.tagline.trim()
    {
        String::new()
    } else {
        format!(
            "<p class=\"subtitle\">{}</p>",
            render_inline_text(&hero.subtitle)
        )
    };
    format!(
        "<section class=\"hero\">{}<h1>{}</h1><p class=\"tagline\">{}</p>{}{}<div class=\"hero-actions\">{}</div></section>",
        logo,
        escape_text(&hero.title),
        render_inline_text(&hero.tagline),
        subtitle,
        byline,
        ctas
    )
}

#[cfg(test)]
mod tests {
    use super::render_hero;
    use crate::bricks::hero::Hero;

    #[test]
    fn duplicate_tagline_is_not_rendered_twice() {
        let html = render_hero(
            &Hero {
                logo_src: None,
                title: "Example".into(),
                tagline: "Same copy".into(),
                subtitle: "Same copy".into(),
                person: None,
                ctas: Vec::new(),
            },
            None,
        );

        assert_eq!(html.matches("Same copy").count(), 1);
        assert!(!html.contains("class=\"subtitle\""));
    }

    #[test]
    fn inline_markup_is_rendered_without_allowing_raw_html() {
        let hero = Hero {
            logo_src: None,
            title: "Example".into(),
            tagline: "Use `ignitix` and [flake-parts](https://flake.parts)".into(),
            subtitle: String::new(),
            person: None,
            ctas: Vec::new(),
        };

        let html = render_hero(&hero, None);
        assert!(html.contains("<code>ignitix</code>"));
        assert!(html.contains("href=\"https://flake.parts\""));
        assert!(html.contains(">flake-parts</a>"));
        assert!(!html.contains("[flake-parts](https://flake.parts)"));
    }
}
