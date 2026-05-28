use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::content_format::ContentFormat;
use crate::serde_helpers::deserialize_flexible_id;

/// Portfolio item representing a project or work
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortfolioItem {
    /// Database record ID
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_flexible_id"
    )]
    pub id: Option<String>,

    /// URL-friendly slug
    pub slug: String,

    /// Project title
    pub title: String,

    /// Short description
    pub description: String,

    /// Detailed description or content (markdown)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Rendered HTML content from markdown
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_content: Option<String>,

    /// Technologies used (e.g., ["Rust", "WebAssembly", "Leptos"])
    pub tech_stack: Vec<String>,

    /// Primary project link (GitHub, live demo, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,

    /// Demo/preview URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demo: Option<String>,

    /// Image URL for preview/thumbnail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,

    /// Project completion/publication date
    pub date: DateTime<Utc>,

    /// Whether this project is featured on the main portfolio page
    #[serde(default)]
    pub featured: bool,

    /// Display order (lower numbers appear first)
    #[serde(default)]
    pub order: i32,
}

/// Request payload for publishing or updating a portfolio item via admin API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishPortfolioRequest {
    /// Optional database record ID. Ignored for writes; slug is the upsert key.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_flexible_id"
    )]
    pub id: Option<String>,

    /// URL-friendly slug. Generated from title by the CLI/server if absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,

    /// Project title.
    pub title: String,

    /// Short description.
    pub description: String,

    /// Optional long-form markdown content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Optional rendered HTML content. Markdown content is rendered server-side.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_content: Option<String>,

    /// Technologies used.
    pub tech_stack: Vec<String>,

    /// Primary project link.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,

    /// Demo/preview URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub demo: Option<String>,

    /// Hosted preview/thumbnail image URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,

    /// Project completion/publication date.
    pub date: DateTime<Utc>,

    /// Whether this project is featured on the main portfolio page.
    #[serde(default)]
    pub featured: bool,

    /// Display order (lower numbers appear first).
    #[serde(default)]
    pub order: i32,

    /// Source content format. Only markdown is currently accepted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_format: Option<ContentFormat>,
}

impl PortfolioItem {
    /// Generate a URL-friendly slug from title
    pub fn slugify(title: &str) -> String {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(
            PortfolioItem::slugify("My Awesome Project"),
            "my-awesome-project"
        );
        assert_eq!(PortfolioItem::slugify("Rust & WASM"), "rust-_-wasm");
    }

    #[test]
    fn test_slugify_empty() {
        assert_eq!(PortfolioItem::slugify(""), "");
    }

    #[test]
    fn test_slugify_numbers_only() {
        assert_eq!(PortfolioItem::slugify("123 456"), "123-456");
    }

    #[test]
    fn test_slugify_consecutive_spaces() {
        assert_eq!(
            PortfolioItem::slugify("Multiple   Spaces   Here"),
            "multiple-spaces-here"
        );
    }
}
