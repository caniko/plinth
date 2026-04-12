use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::content_format::ContentFormat;
use crate::serde_helpers::deserialize_flexible_id;

/// Full blog post with all fields (used when displaying individual post)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlogPost {
    /// SurrealDB record ID
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
    /// SurrealDB record ID
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

fn default_published() -> bool {
    true
}

fn default_source() -> String {
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

/// Navigation context for a post within a series
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeriesNav {
    pub series_slug: String,
    pub series_title: String,
    pub current_position: u32,
    pub total_published: u32,
    pub prev: Option<SeriesEntry>,
    pub next: Option<SeriesEntry>,
    pub entries: Vec<SeriesEntry>,
}

/// A single entry in a series table of contents
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeriesEntry {
    pub slug: String,
    pub title: String,
    #[serde(alias = "series_position", default)]
    pub position: u32,
}

/// Lightweight series info for listing pages
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeriesListItem {
    pub slug: String,
    pub title: String,
    pub post_count: u32,
    pub total_reading_time: u32,
    pub latest_published_at: Option<DateTime<Utc>>,
}

/// Convert a slug like "weekly-rust-tips" to "Weekly Rust Tips"
pub fn humanize_slug(slug: &str) -> String {
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    upper + chars.as_str()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(BlogPost::slugify("Hello World"), "hello-world");
        assert_eq!(
            BlogPost::slugify("Rust & WebAssembly"),
            "rust-_-webassembly"
        );
        assert_eq!(BlogPost::slugify("My First Post!"), "my-first-post_");
    }

    #[test]
    fn test_slugify_empty_string() {
        assert_eq!(BlogPost::slugify(""), "");
    }

    #[test]
    fn test_slugify_consecutive_dashes() {
        assert_eq!(BlogPost::slugify("hello---world"), "hello-world");
    }

    #[test]
    fn test_slugify_leading_trailing_whitespace() {
        assert_eq!(BlogPost::slugify("  Hello World  "), "hello-world");
    }

    #[test]
    fn test_slugify_numbers() {
        assert_eq!(BlogPost::slugify("Part 1 of 3"), "part-1-of-3");
    }

    #[test]
    fn test_reading_time() {
        let short_content = "Hello world";
        assert_eq!(BlogPost::calculate_reading_time(short_content), 1);

        let long_content = (0..400).map(|_| "word").collect::<Vec<_>>().join(" ");
        assert_eq!(BlogPost::calculate_reading_time(&long_content), 2);
    }

    #[test]
    fn test_reading_time_empty() {
        assert_eq!(BlogPost::calculate_reading_time(""), 1);
    }

    #[test]
    fn test_reading_time_boundary_200_words() {
        let content = (0..200).map(|_| "word").collect::<Vec<_>>().join(" ");
        assert_eq!(BlogPost::calculate_reading_time(&content), 1);
    }

    #[test]
    fn test_reading_time_201_words() {
        let content = (0..201).map(|_| "word").collect::<Vec<_>>().join(" ");
        assert_eq!(BlogPost::calculate_reading_time(&content), 2);
    }

    #[test]
    fn test_blog_post_serialization_roundtrip() {
        let post = BlogPost {
            id: None,
            slug: "test-post".to_string(),
            title: "Test Post".to_string(),
            description: String::new(),
            content: "Hello world".to_string(),
            html_content: "<p>Hello world</p>".to_string(),
            published_at: chrono::Utc::now(),
            updated_at: None,
            author: "Author".to_string(),
            tags: vec!["rust".to_string(), "web".to_string()],
            featured: false,
            published: true,
            reading_time_minutes: 1,
            embedding: None,
            content_format: ContentFormat::default(),
            source: default_source(),
            content_hash: None,
            series_slug: None,
            series_title: None,
            series_position: None,
        };
        let json = serde_json::to_string(&post).unwrap();
        let deserialized: BlogPost = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.slug, "test-post");
        assert_eq!(deserialized.title, "Test Post");
        assert_eq!(deserialized.tags, vec!["rust", "web"]);
        assert!(!deserialized.featured);
        assert!(deserialized.published);
    }

    #[test]
    fn test_publish_request_skip_none_fields() {
        let req = PublishArticleRequest {
            title: None,
            content: "body".to_string(),
            slug: None,
            description: None,
            author: None,
            tags: None,
            featured: None,
            published: None,
            embedding: None,
            content_format: None,
            html_content: None,
            series: None,
            series_title: None,
            series_position: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("title"));
        assert!(!json.contains("slug"));
        assert!(!json.contains("embedding"));
        assert!(json.contains("content"));
    }

    #[test]
    fn test_blog_list_item_serialization_roundtrip() {
        let item = BlogListItem {
            id: Some("blog_posts:abc".to_string()),
            slug: "my-post".to_string(),
            title: "My Post".to_string(),
            description: "A short description".to_string(),
            published_at: chrono::Utc::now(),
            author: "Me".to_string(),
            tags: vec![],
            featured: true,
            reading_time_minutes: 5,
            series_slug: None,
            series_title: None,
            series_position: None,
        };
        let json = serde_json::to_string(&item).unwrap();
        let deserialized: BlogListItem = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.slug, "my-post");
        assert!(deserialized.featured);
        assert_eq!(deserialized.reading_time_minutes, 5);
    }

    #[test]
    fn test_humanize_slug() {
        assert_eq!(humanize_slug("weekly-rust-tips"), "Weekly Rust Tips");
        assert_eq!(humanize_slug("my-series"), "My Series");
        assert_eq!(humanize_slug("single"), "Single");
        assert_eq!(humanize_slug(""), "");
    }

    #[test]
    fn test_series_nav_serialization_roundtrip() {
        let nav = SeriesNav {
            series_slug: "my-series".to_string(),
            series_title: "My Series".to_string(),
            current_position: 2,
            total_published: 3,
            prev: Some(SeriesEntry {
                slug: "part-1".to_string(),
                title: "Part 1".to_string(),
                position: 1,
            }),
            next: Some(SeriesEntry {
                slug: "part-3".to_string(),
                title: "Part 3".to_string(),
                position: 3,
            }),
            entries: vec![
                SeriesEntry {
                    slug: "part-1".to_string(),
                    title: "Part 1".to_string(),
                    position: 1,
                },
                SeriesEntry {
                    slug: "part-2".to_string(),
                    title: "Part 2".to_string(),
                    position: 2,
                },
                SeriesEntry {
                    slug: "part-3".to_string(),
                    title: "Part 3".to_string(),
                    position: 3,
                },
            ],
        };
        let json = serde_json::to_string(&nav).unwrap();
        let deserialized: SeriesNav = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.series_slug, "my-series");
        assert_eq!(deserialized.current_position, 2);
        assert_eq!(deserialized.total_published, 3);
        assert_eq!(deserialized.prev.unwrap().slug, "part-1");
        assert_eq!(deserialized.next.unwrap().slug, "part-3");
        assert_eq!(deserialized.entries.len(), 3);
    }

    #[test]
    fn test_series_entry_serde_alias() {
        // SurrealDB returns "series_position" but our struct field is "position"
        let json = r#"{"slug":"p1","title":"Part 1","series_position":5}"#;
        let entry: SeriesEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.position, 5);
        assert_eq!(entry.slug, "p1");

        // Also works with the canonical field name
        let json2 = r#"{"slug":"p2","title":"Part 2","position":3}"#;
        let entry2: SeriesEntry = serde_json::from_str(json2).unwrap();
        assert_eq!(entry2.position, 3);
    }

    #[test]
    fn test_series_list_item_serialization() {
        let item = SeriesListItem {
            slug: "weekly-tips".to_string(),
            title: "Weekly Tips".to_string(),
            post_count: 5,
            total_reading_time: 25,
            latest_published_at: Some(chrono::Utc::now()),
        };
        let json = serde_json::to_string(&item).unwrap();
        let deserialized: SeriesListItem = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.slug, "weekly-tips");
        assert_eq!(deserialized.post_count, 5);
        assert_eq!(deserialized.total_reading_time, 25);
        assert!(deserialized.latest_published_at.is_some());

        // Also test with None date
        let item_no_date = SeriesListItem {
            slug: "new".to_string(),
            title: "New".to_string(),
            post_count: 0,
            total_reading_time: 0,
            latest_published_at: None,
        };
        let json2 = serde_json::to_string(&item_no_date).unwrap();
        let de2: SeriesListItem = serde_json::from_str(&json2).unwrap();
        assert!(de2.latest_published_at.is_none());
    }

    #[test]
    fn test_blog_post_with_series_fields() {
        let post = BlogPost {
            id: None,
            slug: "series-post".to_string(),
            title: "A Series Post".to_string(),
            description: String::new(),
            content: "content".to_string(),
            html_content: "<p>content</p>".to_string(),
            published_at: chrono::Utc::now(),
            updated_at: None,
            author: "Author".to_string(),
            tags: vec![],
            featured: false,
            published: true,
            reading_time_minutes: 1,
            embedding: None,
            content_format: ContentFormat::default(),
            source: default_source(),
            content_hash: None,
            series_slug: Some("weekly-tips".to_string()),
            series_title: Some("Weekly Tips".to_string()),
            series_position: Some(3),
        };
        let json = serde_json::to_string(&post).unwrap();
        assert!(json.contains("weekly-tips"));
        assert!(json.contains("Weekly Tips"));

        let deserialized: BlogPost = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.series_slug.as_deref(), Some("weekly-tips"));
        assert_eq!(deserialized.series_title.as_deref(), Some("Weekly Tips"));
        assert_eq!(deserialized.series_position, Some(3));
    }

    #[test]
    fn test_blog_list_item_from_blog_post_propagates_series() {
        let post = BlogPost {
            id: Some("blog_posts:abc".to_string()),
            slug: "part-2".to_string(),
            title: "Part 2".to_string(),
            description: "desc".to_string(),
            content: "body".to_string(),
            html_content: "<p>body</p>".to_string(),
            published_at: chrono::Utc::now(),
            updated_at: None,
            author: "Me".to_string(),
            tags: vec![],
            featured: false,
            published: true,
            reading_time_minutes: 2,
            embedding: None,
            content_format: ContentFormat::default(),
            source: default_source(),
            content_hash: None,
            series_slug: Some("my-series".to_string()),
            series_title: Some("My Series".to_string()),
            series_position: Some(2),
        };

        let list_item = BlogListItem::from(&post);
        assert_eq!(list_item.series_slug.as_deref(), Some("my-series"));
        assert_eq!(list_item.series_title.as_deref(), Some("My Series"));
        assert_eq!(list_item.series_position, Some(2));

        // Also test owned conversion
        let list_item_owned = BlogListItem::from(post);
        assert_eq!(list_item_owned.series_slug.as_deref(), Some("my-series"));
        assert_eq!(list_item_owned.series_position, Some(2));
    }

    #[test]
    fn test_publish_request_series_fields_skip_none() {
        let req = PublishArticleRequest {
            title: Some("Test".to_string()),
            content: "body".to_string(),
            slug: None,
            description: None,
            author: None,
            tags: None,
            featured: None,
            published: None,
            embedding: None,
            content_format: None,
            html_content: None,
            series: None,
            series_title: None,
            series_position: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("series"));
    }

    #[test]
    fn test_publish_request_series_fields_present() {
        let req = PublishArticleRequest {
            title: Some("Test".to_string()),
            content: "body".to_string(),
            slug: None,
            description: None,
            author: None,
            tags: None,
            featured: None,
            published: None,
            embedding: None,
            content_format: None,
            html_content: None,
            series: Some("weekly-tips".to_string()),
            series_title: Some("Weekly Tips".to_string()),
            series_position: Some(1),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"series\":\"weekly-tips\""));
        assert!(json.contains("\"series_title\":\"Weekly Tips\""));
        assert!(json.contains("\"series_position\":1"));
    }

    #[test]
    fn test_source_defaults_to_api() {
        let json = r#"{
            "slug": "test", "title": "T", "content": "c",
            "html_content": "h", "published_at": "2025-01-01T00:00:00Z",
            "author": "A", "tags": [], "reading_time_minutes": 1
        }"#;
        let post: BlogPost = serde_json::from_str(json).unwrap();
        assert_eq!(post.source, "api");
        assert!(post.content_hash.is_none());
    }

    #[test]
    fn test_source_declarative_roundtrip() {
        let mut post = BlogPost {
            id: None,
            slug: "decl-post".to_string(),
            title: "Declarative".to_string(),
            description: String::new(),
            content: "body".to_string(),
            html_content: "<p>body</p>".to_string(),
            published_at: chrono::Utc::now(),
            updated_at: None,
            author: "A".to_string(),
            tags: vec![],
            featured: false,
            published: true,
            reading_time_minutes: 1,
            embedding: None,
            content_format: ContentFormat::default(),
            source: "declarative".to_string(),
            content_hash: Some("abc123hash".to_string()),
            series_slug: None,
            series_title: None,
            series_position: None,
        };
        let json = serde_json::to_string(&post).unwrap();
        assert!(json.contains("\"source\":\"declarative\""));
        assert!(json.contains("\"content_hash\":\"abc123hash\""));

        let deserialized: BlogPost = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.source, "declarative");
        assert_eq!(deserialized.content_hash.as_deref(), Some("abc123hash"));

        // Verify content_hash is skipped when None
        post.content_hash = None;
        let json2 = serde_json::to_string(&post).unwrap();
        assert!(!json2.contains("content_hash"));
    }
}
