use serde::{Deserialize, Serialize};

use crate::content_format::ContentFormat;

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

    /// Source content format (defaults to Markdown for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_format: Option<ContentFormat>,

    /// Pre-rendered HTML content (used for Typst posts where CLI compiles to HTML)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_content: Option<String>,

    /// Series slug (assigns this post to a series)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,

    /// Series display title (optional, humanized from slug if absent)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_title: Option<String>,

    /// Position within the series (auto-assigned if absent)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_position: Option<u32>,
}
