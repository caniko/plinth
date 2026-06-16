#[cfg(feature = "brick-person-mention")]
use plinth_person::{ExternalLink, LinkKind};

/// Returns an HTML `id` attribute fragment, or the empty string when `id` is
/// `None`.
///
/// ```ignore
/// id_attr(Some("intro")) // => " id=\"intro\""
/// id_attr(None)          // => ""
/// ```
#[allow(dead_code)]
pub(crate) fn id_attr(id: Option<&str>) -> String {
    id.map_or_else(String::new, |id| format!(" id=\"{}\"", escape_attr(id)))
}

/// Escapes `&`, `<`, and `>` for safe insertion into HTML text content.
pub(crate) fn escape_text(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escapes text for safe insertion into an HTML attribute value (also
/// handles `&`, `<`, `>` via [`escape_text`]).
pub(crate) fn escape_attr(input: &str) -> String {
    escape_text(input).replace('"', "&quot;")
}

/// Returns `target`/`rel` attributes for an external link.
///
/// Returns ` target=\"_blank\" rel=\"noopener noreferrer\"` when `href`
/// starts with `http://` or `https://`, or the empty string for relative
/// links.
pub(crate) fn external_attrs(href: &str) -> &'static str {
    if href.starts_with("http://") || href.starts_with("https://") {
        " target=\"_blank\" rel=\"noopener noreferrer\""
    } else {
        ""
    }
}

/// Renders an `<a>` element for an external link, using a CSS class derived
/// from the link's kind.
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

/// Maps a [`LinkKind`] to its corresponding CSS class name.
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
