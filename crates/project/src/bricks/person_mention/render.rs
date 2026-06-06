use super::PersonMention;
use crate::render::{escape_attr, escape_text, external_attrs, id_attr, render_external_link};

pub fn render_person_mention(mention: &PersonMention) -> String {
    let role = mention
        .person
        .role
        .as_ref()
        .map_or_else(String::new, |role| {
            format!("<p class=\"person-role\">{}</p>", escape_text(role))
        });
    let avatar = mention
        .person
        .avatar_url
        .as_ref()
        .map_or_else(String::new, |url| {
            format!(
                "<img class=\"person-avatar\" src=\"{}\" alt=\"{}\">",
                escape_attr(url),
                escape_attr(&mention.person.name)
            )
        });
    let links = mention
        .person
        .links
        .iter()
        .map(render_external_link)
        .collect::<String>();
    format!(
        "<section{} class=\"person-mention\"><div class=\"section-heading\"><h2>{}</h2><p>{}</p></div><article class=\"person-card\">{}<div><h3><a href=\"{}\"{}>{}</a></h3>{}<div class=\"person-links\">{}</div></div></article></section>",
        id_attr(mention.id.as_deref()),
        escape_text(&mention.heading),
        escape_text(&mention.intro),
        avatar,
        escape_attr(&mention.person.url),
        external_attrs(&mention.person.url),
        escape_text(&mention.person.name),
        role,
        links
    )
}
