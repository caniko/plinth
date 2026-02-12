use pulldown_cmark::{html, Options, Parser};
use gray_matter::{engine::YAML, Matter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Frontmatter metadata for blog posts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostFrontmatter {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
    pub published: Option<bool>,
    pub featured: Option<bool>,
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
    // Parse frontmatter
    let matter = Matter::<YAML>::new();
    let parsed = matter.parse(content);

    // Extract frontmatter if present
    let frontmatter: Option<PostFrontmatter> = parsed
        .data
        .as_ref()
        .and_then(|data| serde_yaml::from_str(&data.to_string()).ok());

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

/// Convert markdown string to HTML
pub fn markdown_to_html(markdown: &str) -> String {
    // Set up options for extended markdown features
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

    let parser = Parser::new_ext(markdown, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
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
    fn test_generate_slug() {
        assert_eq!(generate_slug("Hello World"), "hello-world");
        assert_eq!(generate_slug("Rust & WebAssembly"), "rust-webassembly");
        assert_eq!(generate_slug("Multiple   Spaces"), "multiple-spaces");
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
}
