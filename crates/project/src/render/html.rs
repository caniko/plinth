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

/// Renders the small inline-markup subset accepted in generated copy.
///
/// Backtick-delimited code spans and Markdown links are supported. Everything
/// else remains escaped text, so generated copy cannot inject raw HTML.
pub(crate) fn render_inline_text(input: &str) -> String {
    let mut rendered = String::new();
    let mut cursor = 0;
    let mut plain_start = 0;

    while cursor < input.len() {
        let Some(character) = input[cursor..].chars().next() else {
            break;
        };

        if character == '`' {
            if let Some(end) = input[cursor + character.len_utf8()..].find('`') {
                let content_start = cursor + character.len_utf8();
                let content_end = content_start + end;
                rendered.push_str(&escape_text(&input[plain_start..cursor]));
                rendered.push_str("<code>");
                rendered.push_str(&escape_text(&input[content_start..content_end]));
                rendered.push_str("</code>");
                cursor = content_end + 1;
                plain_start = cursor;
                continue;
            }
        } else if character == '[' {
            let label_start = cursor + character.len_utf8();
            if let Some(label_end) = input[label_start..].find("](") {
                let label_end = label_start + label_end;
                let href_start = label_end + 2;
                if let Some(href_end) = input[href_start..].find(')') {
                    let href_end = href_start + href_end;
                    let href = &input[href_start..href_end];
                    if safe_inline_href(href) {
                        rendered.push_str(&escape_text(&input[plain_start..cursor]));
                        rendered.push_str("<a href=\"");
                        rendered.push_str(&escape_attr(href));
                        rendered.push('"');
                        rendered.push_str(external_attrs(href));
                        rendered.push('>');
                        rendered.push_str(&escape_text(&input[label_start..label_end]));
                        rendered.push_str("</a>");
                        cursor = href_end + 1;
                        plain_start = cursor;
                        continue;
                    }
                }
            }
        }

        cursor += character.len_utf8();
    }

    rendered.push_str(&escape_text(&input[plain_start..]));
    rendered
}

fn safe_inline_href(href: &str) -> bool {
    href.starts_with("https://")
        || href.starts_with("http://")
        || href.starts_with('/')
        || href.starts_with('#')
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
