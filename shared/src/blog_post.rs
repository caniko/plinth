use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Full blog post with all fields (used when displaying individual post)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogPost {
    /// SurrealDB record ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// URL-friendly slug
    pub slug: String,

    /// Post title
    pub title: String,

    /// Original markdown content
    pub content: String,

    /// Rendered HTML content
    pub html_content: String,

    /// Publication date
    pub published_at: DateTime<Utc>,

    /// Last updated date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,

    /// Author name
    pub author: String,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Whether this post is featured
    #[serde(default)]
    pub featured: bool,

    /// Whether this post is published (vs draft)
    #[serde(default = "default_published")]
    pub published: bool,

    /// Estimated reading time in minutes
    pub reading_time_minutes: u32,

    /// Vector embedding for semantic search (384 dimensions from fastembed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

/// Lightweight version for listing pages (excludes large fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogListItem {
    /// SurrealDB record ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// URL-friendly slug
    pub slug: String,

    /// Post title
    pub title: String,

    /// Short description/excerpt
    pub description: String,

    /// Publication date
    pub published_at: DateTime<Utc>,

    /// Author name
    pub author: String,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Whether this post is featured
    #[serde(default)]
    pub featured: bool,

    /// Estimated reading time in minutes
    pub reading_time_minutes: u32,
}

/// Request payload for publishing a new article via API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishArticleRequest {
    /// Post title (can also be extracted from frontmatter)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Original markdown content (with optional frontmatter)
    pub content: String,

    /// URL-friendly slug (auto-generated from title if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,

    /// Short description/excerpt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Author name (defaults to configured author if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Tags for categorization (can also be extracted from frontmatter)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Whether this post is featured
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured: Option<bool>,

    /// Whether to publish immediately (vs save as draft)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<bool>,

    /// Pre-generated vector embedding (384 dimensions from fastembed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

fn default_published() -> bool {
    true
}

impl BlogPost {
    /// Calculate reading time based on word count (avg 200 words per minute)
    pub fn calculate_reading_time(content: &str) -> u32 {
        let word_count = content.split_whitespace().count();
        ((word_count as f32 / 200.0).ceil() as u32).max(1)
    }

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
        assert_eq!(BlogPost::slugify("Hello World"), "hello-world");
        assert_eq!(BlogPost::slugify("Rust & WebAssembly"), "rust-_-webassembly");
        assert_eq!(BlogPost::slugify("My First Post!"), "my-first-post_");
    }

    #[test]
    fn test_reading_time() {
        let short_content = "Hello world";
        assert_eq!(BlogPost::calculate_reading_time(short_content), 1);

        let long_content = (0..400).map(|_| "word").collect::<Vec<_>>().join(" ");
        assert_eq!(BlogPost::calculate_reading_time(&long_content), 2);
    }
}
