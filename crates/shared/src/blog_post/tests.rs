use super::post::default_source;
use super::*;
use crate::content_format::ContentFormat;

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
    // Database rows may use "series_position" while the struct field is "position".
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
