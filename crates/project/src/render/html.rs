#[cfg(feature = "brick-person-mention")]
use plinth_person::{ExternalLink, LinkKind};

#[allow(dead_code)]
pub(crate) fn id_attr(id: Option<&str>) -> String {
    id.map_or_else(String::new, |id| format!(" id=\"{}\"", escape_attr(id)))
}

pub(crate) fn escape_text(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(crate) fn escape_attr(input: &str) -> String {
    escape_text(input).replace('"', "&quot;")
}

pub(crate) fn external_attrs(href: &str) -> &'static str {
    if href.starts_with("http://") || href.starts_with("https://") {
        " target=\"_blank\" rel=\"noopener noreferrer\""
    } else {
        ""
    }
}

#[cfg(feature = "brick-person-mention")]
pub(crate) fn render_external_link(link: &ExternalLink) -> String {
    format!(
        "<a class=\"person-link link-{}\" href=\"{}\"{}>{}</a>",
        link_kind_class(&link.kind),
        escape_attr(&link.href),
        external_attrs(&link.href),
        escape_text(&link.label),
    )
}

#[cfg(feature = "brick-person-mention")]
fn link_kind_class(kind: &LinkKind) -> &'static str {
    match kind {
        LinkKind::Person => "person",
        LinkKind::ProjectSite => "project-site",
        LinkKind::Source => "source",
        LinkKind::Demo => "demo",
        LinkKind::Docs => "docs",
        LinkKind::Contact => "contact",
        LinkKind::Other => "other",
    }
}
