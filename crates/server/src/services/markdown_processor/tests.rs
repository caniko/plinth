use super::images::{html_escape, parse_image_dimensions, render_image_tag};
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
