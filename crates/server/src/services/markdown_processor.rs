use gray_matter::{Matter, engine::YAML};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;

/// Frontmatter metadata for blog posts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostFrontmatter {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
    pub published: Option<bool>,
    pub featured: Option<bool>,
    pub series: Option<String>,
    pub series_title: Option<String>,
    pub series_position: Option<u32>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Parsed markdown content with frontmatter and HTML
#[derive(Debug, Clone)]
pub struct ParsedMarkdown {
    pub frontmatter: Option<PostFrontmatter>,
    pub markdown_content: String,
    pub html_content: String,
    pub reading_time_minutes: u32,
}

/// Parse markdown content with optional frontmatter
pub fn parse_markdown(content: &str) -> Result<ParsedMarkdown, String> {
    // Parse frontmatter with direct deserialization
    let matter = Matter::<YAML>::new();
    let parsed = matter
        .parse::<PostFrontmatter>(content)
        .map_err(|e| format!("Failed to parse content: {}", e))?;

    // Frontmatter is directly deserialized
    let frontmatter = parsed.data;

    // Get markdown content (without frontmatter)
    let markdown_content = parsed.content.clone();

    // Convert markdown to HTML
    let html_content = markdown_to_html(&markdown_content);

    // Calculate reading time (average 200 words per minute)
    let word_count = markdown_content.split_whitespace().count();
    let reading_time_minutes = ((word_count as f32 / 200.0).ceil() as u32).max(1);

    Ok(ParsedMarkdown {
        frontmatter,
        markdown_content,
        html_content,
        reading_time_minutes,
    })
}

/// Convert markdown string to HTML with enhanced image handling.
///
/// Images are rendered with `loading="lazy"`, and for Immich proxy images
/// (`/api/images/{id}?w=X&h=Y`), generates `width`/`height` attributes
/// and `srcset` for responsive loading.
pub fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(markdown, options);

    let mut html_output = String::new();
    let mut in_image = false;
    let mut image_url = String::new();
    let mut image_title = String::new();
    let mut image_alt = String::new();

    // Collect events, handling images specially
    let mut events: Vec<Event> = Vec::new();
    for event in parser {
        match &event {
            Event::Start(Tag::Image {
                dest_url, title, ..
            }) => {
                in_image = true;
                image_url = dest_url.to_string();
                image_title = title.to_string();
                image_alt.clear();
                continue;
            }
            Event::End(TagEnd::Image) => {
                in_image = false;
                let img_html = render_image_tag(&image_url, &image_alt, &image_title);
                events.push(Event::Html(img_html.into()));
                continue;
            }
            Event::Text(text) if in_image => {
                image_alt.push_str(text);
                continue;
            }
            _ => {}
        }
        events.push(event);
    }

    html::push_html(&mut html_output, events.into_iter());
    html_output
}

/// Parse dimension query params (`?w=X&h=Y`) from a URL.
fn parse_image_dimensions(url: &str) -> (String, Option<u32>, Option<u32>) {
    if let Some(idx) = url.find('?') {
        let base = &url[..idx];
        let query = &url[idx + 1..];
        let mut width = None;
        let mut height = None;
        for pair in query.split('&') {
            if let Some(val) = pair.strip_prefix("w=") {
                width = val.parse().ok();
            } else if let Some(val) = pair.strip_prefix("h=") {
                height = val.parse().ok();
            }
        }
        (base.to_string(), width, height)
    } else {
        (url.to_string(), None, None)
    }
}

/// Render an `<img>` tag with lazy loading, optional dimensions, and srcset for proxy images.
fn render_image_tag(url: &str, alt: &str, title: &str) -> String {
    let (base_url, width, height) = parse_image_dimensions(url);
    let is_proxy = base_url.starts_with("/api/images/");
    let escaped_alt = html_escape(alt);

    let mut tag = String::new();

    if is_proxy {
        let width_descriptor = width.map_or("2560w".to_string(), |w| format!("{}w", w));
        let _ = write!(
            tag,
            "<img src=\"{}?size=preview\" \
             srcset=\"{}?size=thumbnail 250w, {}?size=preview 1440w, {}?size=original {}\" \
             sizes=\"(max-width: 768px) 100vw, (max-width: 1200px) 80vw, 1200px\" \
             loading=\"lazy\" alt=\"{}\"",
            base_url, base_url, base_url, base_url, width_descriptor, escaped_alt
        );
    } else {
        let _ = write!(
            tag,
            "<img src=\"{}\" loading=\"lazy\" alt=\"{}\"",
            html_escape(url),
            escaped_alt
        );
    }

    if let Some(w) = width {
        let _ = write!(tag, " width=\"{}\"", w);
    }
    if let Some(h) = height {
        let _ = write!(tag, " height=\"{}\"", h);
    }
    if !title.is_empty() {
        let _ = write!(tag, " title=\"{}\"", html_escape(title));
    }

    tag.push_str(" />");
    tag
}

/// Minimal HTML escaping for attribute values.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Generate a URL-safe slug from a title
pub fn generate_slug(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' {
                '-'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_html() {
        let markdown = "# Hello\n\nThis is **bold** text.";
        let html = markdown_to_html(markdown);
        assert!(html.contains("<h1>"));
        assert!(html.contains("<strong>"));
    }

    #[test]
    fn test_markdown_to_html_code_block() {
        let md = "```rust\nfn main() {}\n```";
        let html = markdown_to_html(md);
        assert!(html.contains("<code"));
        assert!(html.contains("fn main()"));
    }

    #[test]
    fn test_markdown_to_html_table() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let html = markdown_to_html(md);
        assert!(html.contains("<table"));
        assert!(html.contains("<td>"));
    }

    #[test]
    fn test_markdown_to_html_strikethrough() {
        let md = "This is ~~deleted~~ text.";
        let html = markdown_to_html(md);
        assert!(html.contains("<del>"));
    }

    #[test]
    fn test_markdown_to_html_empty() {
        let html = markdown_to_html("");
        assert!(html.is_empty());
    }

    #[test]
    fn test_generate_slug() {
        assert_eq!(generate_slug("Hello World"), "hello-world");
        assert_eq!(generate_slug("Rust & WebAssembly"), "rust-_-webassembly");
        assert_eq!(generate_slug("Multiple   Spaces"), "multiple-spaces");
    }

    #[test]
    fn test_generate_slug_empty() {
        assert_eq!(generate_slug(""), "");
    }

    #[test]
    fn test_parse_markdown_with_frontmatter() {
        let content = r#"---
title: Test Post
tags: ["rust", "web"]
---

# Content

Hello world!"#;

        let parsed = parse_markdown(content).unwrap();
        assert!(parsed.frontmatter.is_some());
        assert_eq!(
            parsed.frontmatter.as_ref().unwrap().title.as_deref(),
            Some("Test Post")
        );
        assert!(parsed.html_content.contains("<h1>"));
    }

    #[test]
    fn test_parse_markdown_without_frontmatter() {
        let content = "# Just Content\n\nNo frontmatter here.";
        let parsed = parse_markdown(content).unwrap();
        assert!(parsed.frontmatter.is_none());
        assert!(parsed.html_content.contains("<h1>"));
    }

    #[test]
    fn test_reading_time_calculation() {
        let words = vec!["word"; 500].join(" ");
        let parsed = parse_markdown(&words).unwrap();
        // 500 words / 200 wpm = 2.5 -> rounds to 3 minutes
        assert_eq!(parsed.reading_time_minutes, 3);
    }

    #[test]
    fn test_frontmatter_all_fields() {
        let content = r#"---
title: Full Test
description: A test post
tags: ["a", "b"]
author: Me
published: false
featured: true
---

Some content here."#;
        let parsed = parse_markdown(content).unwrap();
        let fm = parsed.frontmatter.unwrap();
        assert_eq!(fm.title.as_deref(), Some("Full Test"));
        assert_eq!(fm.description.as_deref(), Some("A test post"));
        assert_eq!(fm.author.as_deref(), Some("Me"));
        assert_eq!(fm.published, Some(false));
        assert_eq!(fm.featured, Some(true));
        assert_eq!(
            fm.tags.as_deref(),
            Some(&["a".to_string(), "b".to_string()][..])
        );
    }

    #[test]
    fn test_parse_markdown_only_frontmatter() {
        let content = "---\ntitle: Only FM\n---\n";
        let parsed = parse_markdown(content).unwrap();
        assert!(parsed.frontmatter.is_some());
        assert_eq!(parsed.reading_time_minutes, 1);
    }

    #[test]
    fn test_markdown_image_lazy_loading() {
        let md = "![photo](https://example.com/img.jpg)";
        let html = markdown_to_html(md);
        assert!(html.contains("loading=\"lazy\""));
        assert!(html.contains("alt=\"photo\""));
    }

    #[test]
    fn test_markdown_proxy_image_srcset() {
        let md = "![photo](/api/images/abc-123?w=1920&h=1080)";
        let html = markdown_to_html(md);
        assert!(html.contains("srcset="));
        assert!(html.contains("?size=thumbnail 250w"));
        assert!(html.contains("?size=preview 1440w"));
        assert!(html.contains("?size=original 1920w"));
        assert!(html.contains("width=\"1920\""));
        assert!(html.contains("height=\"1080\""));
        assert!(html.contains("loading=\"lazy\""));
    }

    #[test]
    fn test_markdown_proxy_image_no_dimensions() {
        let md = "![photo](/api/images/abc-123)";
        let html = markdown_to_html(md);
        assert!(html.contains("src=\"/api/images/abc-123?size=preview\""));
        assert!(html.contains("?size=original 2560w"));
        assert!(!html.contains("width="));
        assert!(!html.contains("height="));
    }

    #[test]
    fn test_parse_image_dimensions() {
        let (base, w, h) = parse_image_dimensions("/api/images/abc?w=1920&h=1080");
        assert_eq!(base, "/api/images/abc");
        assert_eq!(w, Some(1920));
        assert_eq!(h, Some(1080));
    }

    #[test]
    fn test_parse_image_dimensions_no_params() {
        let (base, w, h) = parse_image_dimensions("/api/images/abc");
        assert_eq!(base, "/api/images/abc");
        assert_eq!(w, None);
        assert_eq!(h, None);
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("a<b>c&d\"e"), "a&lt;b&gt;c&amp;d&quot;e");
    }

    #[test]
    fn test_html_escape_no_special_chars() {
        assert_eq!(html_escape("hello world"), "hello world");
    }

    #[test]
    fn test_render_image_tag_proxy_with_dimensions() {
        let html = render_image_tag("/api/images/abc-123?w=1920&h=1080", "sunset", "");
        assert!(html.contains("src=\"/api/images/abc-123?size=preview\""));
        assert!(html.contains("srcset="));
        assert!(html.contains("?size=thumbnail 250w"));
        assert!(html.contains("?size=preview 1440w"));
        assert!(html.contains("?size=original 1920w"));
        assert!(html.contains("width=\"1920\""));
        assert!(html.contains("height=\"1080\""));
        assert!(html.contains("loading=\"lazy\""));
        assert!(html.contains("alt=\"sunset\""));
    }

    #[test]
    fn test_render_image_tag_proxy_no_dimensions() {
        let html = render_image_tag("/api/images/abc-123", "photo", "");
        assert!(html.contains("src=\"/api/images/abc-123?size=preview\""));
        assert!(html.contains("?size=original 2560w"));
        assert!(!html.contains("width=\""));
        assert!(!html.contains("height=\""));
    }

    #[test]
    fn test_render_image_tag_external_url() {
        let html = render_image_tag("https://example.com/img.jpg", "photo", "");
        assert!(html.contains("src=\"https://example.com/img.jpg\""));
        assert!(!html.contains("srcset"));
        assert!(html.contains("loading=\"lazy\""));
    }

    #[test]
    fn test_render_image_tag_with_title() {
        let html = render_image_tag("https://example.com/img.jpg", "photo", "A nice photo");
        assert!(html.contains("title=\"A nice photo\""));
    }

    #[test]
    fn test_render_image_tag_empty_title_omitted() {
        let html = render_image_tag("https://example.com/img.jpg", "photo", "");
        assert!(!html.contains("title="));
    }

    #[test]
    fn test_render_image_tag_escapes_alt() {
        let html = render_image_tag("https://example.com/img.jpg", "a <b> & \"c\"", "");
        assert!(html.contains("alt=\"a &lt;b&gt; &amp; &quot;c&quot;\""));
    }

    #[test]
    fn test_render_image_tag_width_only() {
        let html = render_image_tag("/api/images/abc?w=800", "photo", "");
        assert!(html.contains("width=\"800\""));
        assert!(!html.contains("height=\""));
    }

    #[test]
    fn test_render_image_tag_height_only() {
        let html = render_image_tag("/api/images/abc?h=600", "photo", "");
        assert!(!html.contains("width=\""));
        assert!(html.contains("height=\"600\""));
    }

    #[test]
    fn test_parse_image_dimensions_reversed_order() {
        let (base, w, h) = parse_image_dimensions("/api/images/abc?h=1080&w=1920");
        assert_eq!(base, "/api/images/abc");
        assert_eq!(w, Some(1920));
        assert_eq!(h, Some(1080));
    }

    #[test]
    fn test_parse_image_dimensions_extra_params() {
        let (base, w, h) = parse_image_dimensions("/api/images/abc?w=100&foo=bar&h=200");
        assert_eq!(base, "/api/images/abc");
        assert_eq!(w, Some(100));
        assert_eq!(h, Some(200));
    }

    #[test]
    fn test_parse_image_dimensions_non_numeric() {
        let (base, w, h) = parse_image_dimensions("/api/images/abc?w=abc&h=200");
        assert_eq!(base, "/api/images/abc");
        assert_eq!(w, None);
        assert_eq!(h, Some(200));
    }

    #[test]
    fn test_parse_image_dimensions_empty_values() {
        let (base, w, h) = parse_image_dimensions("/api/images/abc?w=&h=100");
        assert_eq!(base, "/api/images/abc");
        assert_eq!(w, None);
        assert_eq!(h, Some(100));
    }

    #[test]
    fn test_markdown_multiple_images_with_text() {
        let md = "Intro text.\n\n![first](/api/images/aaa?w=800&h=600)\n\nMiddle paragraph.\n\n![second](https://example.com/img.jpg)\n\nEnd.";
        let html = markdown_to_html(md);
        // First image: proxy with srcset
        assert!(html.contains("/api/images/aaa?size=preview"));
        assert!(html.contains("alt=\"first\""));
        // Second image: external, no srcset
        assert!(html.contains("src=\"https://example.com/img.jpg\""));
        assert!(html.contains("alt=\"second\""));
        // Text preserved
        assert!(html.contains("Intro text."));
        assert!(html.contains("Middle paragraph."));
        assert!(html.contains("End."));
    }

    #[test]
    fn test_markdown_image_inline_with_text() {
        let md = "Check out this ![photo](pic.jpg) in the text.";
        let html = markdown_to_html(md);
        assert!(html.contains("alt=\"photo\""));
        assert!(html.contains("Check out this"));
        assert!(html.contains("in the text."));
    }

    #[test]
    fn test_markdown_non_image_content_unchanged() {
        let md = "# Title\n\n**Bold** and *italic* with [link](url).";
        let html = markdown_to_html(md);
        assert!(html.contains("<h1>"));
        assert!(html.contains("<strong>"));
        assert!(html.contains("<em>"));
        assert!(html.contains("<a"));
    }

    #[test]
    fn test_frontmatter_series_fields() {
        let content = r#"---
title: Series Post
series: "weekly-rust-tips"
series_title: "Weekly Rust Tips"
series_position: 3
tags: ["rust"]
---

Some content."#;
        let parsed = parse_markdown(content).unwrap();
        let fm = parsed.frontmatter.unwrap();
        assert_eq!(fm.series.as_deref(), Some("weekly-rust-tips"));
        assert_eq!(fm.series_title.as_deref(), Some("Weekly Rust Tips"));
        assert_eq!(fm.series_position, Some(3));
        assert_eq!(fm.title.as_deref(), Some("Series Post"));
    }

    #[test]
    fn test_frontmatter_series_partial() {
        let content = r#"---
title: Partial Series
series: "my-series"
---

Content here."#;
        let parsed = parse_markdown(content).unwrap();
        let fm = parsed.frontmatter.unwrap();
        assert_eq!(fm.series.as_deref(), Some("my-series"));
        assert!(fm.series_title.is_none());
        assert!(fm.series_position.is_none());
    }
}
