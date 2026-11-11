//! Markdown to HTML for the preview pane.
//!
//! Author HTML is escaped rather than passed through: the preview is rendered
//! with `dangerous_inner_html`, so the only HTML in the output is the markup
//! pulldown-cmark generated.

use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html};

pub(crate) fn render(body: &str) -> String {
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let events = Parser::new_ext(body, options).map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) if !is_safe_url(&dest_url) => Event::Start(Tag::Link {
            link_type,
            dest_url: CowStr::Borrowed("#"),
            title,
            id,
        }),
        other => other,
    });
    let mut output = String::new();
    html::push_html(&mut output, events);
    output
}

/// Allow ordinary web and relative links; drop script-bearing schemes.
fn is_safe_url(url: &str) -> bool {
    let trimmed = url.trim_start();
    // A scheme only counts before a path, query or fragment separator.
    let scheme_end = trimmed
        .find([':', '/', '?', '#'])
        .filter(|index| trimmed.as_bytes()[*index] == b':');
    let Some(end) = scheme_end else {
        return true;
    };
    let scheme = trimmed[..end].to_ascii_lowercase();
    !matches!(scheme.as_str(), "javascript" | "data" | "vbscript")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_markdown_and_escapes_author_html() {
        let output = render("# Title\n\nA <script>alert('x')</script> paragraph.");
        assert!(output.contains("<h1>Title</h1>"));
        assert!(!output.contains("<script>"));
        assert!(output.contains("&lt;script&gt;"));
    }

    #[test]
    fn drops_javascript_links_but_keeps_relative_ones() {
        let output = render("[bad](javascript:alert(1)) [good](/posts/hello)");
        assert!(!output.contains("javascript:"));
        assert!(output.contains("href=\"#\""));
        assert!(output.contains("href=\"/posts/hello\""));
    }
}
