use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::content_format::ContentFormat;
use crate::serde_helpers::deserialize_flexible_id;

/// Full blog post with all fields (used when displaying individual post)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlogPost {
    /// Database record ID
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_flexible_id"
    )]
    pub id: Option<String>,

    /// URL-friendly slug
    pub slug: String,

    /// Post title
    pub title: String,

    /// Short description/excerpt
    #[serde(default)]
    pub description: String,

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

    /// Source content format (markdown or typst)
    #[serde(default)]
    pub content_format: ContentFormat,

    /// Content source: "api" (CLI/API published) or "declarative" (Nix-managed)
    #[serde(default = "default_source")]
    pub source: String,

    /// SHA-256 hash of source file content (for declarative change detection)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,

    /// Series slug (if this post belongs to a series)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series_slug: Option<String>,

    /// Series display title
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series_title: Option<String>,

    /// Position within the series (1-based)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series_position: Option<u32>,
}

/// Lightweight version for listing pages (excludes large fields)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlogListItem {
    /// Database record ID
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_flexible_id"
    )]
    pub id: Option<String>,

    /// URL-friendly slug
    pub slug: String,

    /// Post title
    pub title: String,

    /// Short description/excerpt
    #[serde(default)]
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

    /// Series slug (if this post belongs to a series)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series_slug: Option<String>,

    /// Series display title
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series_title: Option<String>,

    /// Position within the series (1-based)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series_position: Option<u32>,
}

fn default_published() -> bool {
    true
}

pub(crate) fn default_source() -> String {
    "api".to_string()
}

impl From<&BlogPost> for BlogListItem {
    fn from(p: &BlogPost) -> Self {
        BlogListItem {
            id: p.id.clone(),
            slug: p.slug.clone(),
            title: p.title.clone(),
            description: if p.description.is_empty() {
                p.content.chars().take(200).collect::<String>() + "..."
            } else {
                p.description.clone()
            },
            published_at: p.published_at,
            author: p.author.clone(),
            tags: p.tags.clone(),
            featured: p.featured,
            reading_time_minutes: p.reading_time_minutes,
            series_slug: p.series_slug.clone(),
            series_title: p.series_title.clone(),
            series_position: p.series_position,
        }
    }
}

impl From<BlogPost> for BlogListItem {
    fn from(p: BlogPost) -> Self {
        BlogListItem {
            id: p.id,
            slug: p.slug,
            title: p.title,
            description: if p.description.is_empty() {
                p.content.chars().take(200).collect::<String>() + "..."
            } else {
                p.description
            },
            published_at: p.published_at,
            author: p.author,
            tags: p.tags,
            featured: p.featured,
            reading_time_minutes: p.reading_time_minutes,
            series_slug: p.series_slug,
            series_title: p.series_title,
            series_position: p.series_position,
        }
    }
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
