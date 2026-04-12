//! Tests for From<BlogPost> for BlogListItem and From<&BlogPost> for BlogListItem.

use chrono::Utc;
use plinth_shared::{BlogListItem, BlogPost, ContentFormat};

fn sample_post() -> BlogPost {
    BlogPost {
        id: Some("blog_posts:abc".to_string()),
        slug: "test-post".to_string(),
        title: "Test Post".to_string(),
        description: "A short description".to_string(),
        content: "Full markdown content here".to_string(),
        html_content: "<p>Full markdown content here</p>".to_string(),
        published_at: Utc::now(),
        updated_at: None,
        author: "Author".to_string(),
        tags: vec!["rust".to_string(), "web".to_string()],
        featured: true,
        published: true,
        reading_time_minutes: 3,
        embedding: None,
        content_format: ContentFormat::Markdown,
        source: "api".to_string(),
        content_hash: None,
        series_slug: None,
        series_title: None,
        series_position: None,
    }
}

#[test]
fn test_from_ref_basic() {
    let post = sample_post();
    let item = BlogListItem::from(&post);

    assert_eq!(item.id, Some("blog_posts:abc".to_string()));
    assert_eq!(item.slug, "test-post");
    assert_eq!(item.title, "Test Post");
    assert_eq!(item.description, "A short description");
    assert_eq!(item.published_at, post.published_at);
    assert_eq!(item.author, "Author");
    assert_eq!(item.tags, vec!["rust", "web"]);
    assert!(item.featured);
    assert_eq!(item.reading_time_minutes, 3);
}

#[test]
fn test_from_owned_basic() {
    let post = sample_post();
    let published_at = post.published_at;
    let item = BlogListItem::from(post);

    assert_eq!(item.slug, "test-post");
    assert_eq!(item.title, "Test Post");
    assert_eq!(item.published_at, published_at);
    assert_eq!(item.reading_time_minutes, 3);
}

#[test]
fn test_empty_description_uses_content_preview() {
    let mut post = sample_post();
    post.description = String::new();
    post.content = "a".repeat(300);

    let item = BlogListItem::from(&post);

    // Should be first 200 chars + "..."
    assert_eq!(item.description.len(), 203);
    assert!(item.description.ends_with("..."));
    assert!(item.description.starts_with("aaa"));
}

#[test]
fn test_nonempty_description_preserved() {
    let mut post = sample_post();
    post.description = "Custom description".to_string();

    let item = BlogListItem::from(&post);
    assert_eq!(item.description, "Custom description");
}

#[test]
fn test_short_content_with_empty_description() {
    let mut post = sample_post();
    post.description = String::new();
    post.content = "Short".to_string();

    let item = BlogListItem::from(&post);
    assert_eq!(item.description, "Short...");
}

#[test]
fn test_owned_empty_description_uses_content() {
    let mut post = sample_post();
    post.description = String::new();
    post.content = "Owned content preview test".to_string();

    let item = BlogListItem::from(post);
    assert_eq!(item.description, "Owned content preview test...");
}

#[test]
fn test_none_id_preserved() {
    let mut post = sample_post();
    post.id = None;

    let item = BlogListItem::from(&post);
    assert!(item.id.is_none());
}

#[test]
fn test_empty_tags_preserved() {
    let mut post = sample_post();
    post.tags = vec![];

    let item = BlogListItem::from(&post);
    assert!(item.tags.is_empty());
}
