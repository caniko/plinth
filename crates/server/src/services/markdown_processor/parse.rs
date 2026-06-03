use gray_matter::{Matter, engine::YAML};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::images::markdown_to_html;

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
