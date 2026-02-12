use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Portfolio item representing a project or work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioItem {
    /// SurrealDB record ID
    #[serde(skip_serializing_if = "Option::is_none")]
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
        assert_eq!(PortfolioItem::slugify("My Awesome Project"), "my-awesome-project");
        assert_eq!(PortfolioItem::slugify("Rust & WASM"), "rust-_-wasm");
    }
}
