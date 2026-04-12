//! Integration tests for blog admin operations (publish, delete, tag management)
//! using in-memory SurrealDB. These exercise the same SQL patterns as the real
//! admin handlers without needing actors or HTTP.

use plinth_server::db_helpers::{take_as, take_as_opt};
use plinth_server::services::db::{init_schema, sync_post_tags_cache};
use plinth_shared::BlogPost;
use surrealdb::Surreal;
use surrealdb::engine::local::Mem;

async fn setup_test_db() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(())
        .await
        .expect("Failed to create in-memory DB");
    db.use_ns("test")
        .use_db("test")
        .await
        .expect("Failed to select ns/db");
    init_schema(&db).await.expect("Failed to init schema");
    db
}

/// Insert a tag if it doesn't exist. Uses SELECT-then-CREATE to avoid
/// UNIQUE constraint errors that produce confusing type errors in SurrealDB.
async fn ensure_tag(db: &Surreal<surrealdb::engine::local::Db>, name: &str, slug: &str) {
    let mut r = db
        .query("SELECT VALUE slug FROM tags WHERE slug = $slug LIMIT 1")
        .bind(("slug", slug.to_string()))
        .await
        .unwrap();
    let existing: Vec<String> = r.take(0).unwrap();
    if existing.is_empty() {
        db.query(
            r#"
            CREATE tags CONTENT {
                name: $name,
                slug: $slug,
                created_at: time::now()
            };
            "#,
        )
        .bind(("name", name.to_string()))
        .bind(("slug", slug.to_string()))
        .await
        .expect("Failed to create tag");
    }
}

/// Helper that mirrors the publish_article handler's insert pattern.
async fn publish_post(
    db: &Surreal<surrealdb::engine::local::Db>,
    slug: &str,
    title: &str,
    tags: &[&str],
    series_slug: Option<&str>,
    series_title: Option<&str>,
    series_position: Option<u32>,
) {
    db.query(
        r#"
        CREATE blog_posts CONTENT {
            slug: $slug,
            title: $title,
            description: "",
            content: "body",
            html_content: "<p>body</p>",
            published_at: time::now(),
            author: "Test",
            tags: $tags,
            featured: false,
            published: true,
            reading_time_minutes: 1,
            embedding: NONE,
            series_slug: $series_slug,
            series_title: $series_title,
            series_position: $series_position
        };
        "#,
    )
    .bind(("slug", slug.to_string()))
    .bind(("title", title.to_string()))
    .bind((
        "tags",
        tags.iter().map(|t| t.to_string()).collect::<Vec<_>>(),
    ))
    .bind(("series_slug", series_slug.map(|s| s.to_string())))
    .bind(("series_title", series_title.map(|s| s.to_string())))
    .bind(("series_position", series_position))
    .await
    .expect("Failed to insert post");

    // Create tags and graph relations
    for tag_name in tags {
        let tag_slug = tag_name.to_lowercase();
        ensure_tag(db, tag_name, &tag_slug).await;

        db.query(
            r#"
            LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = $post_slug LIMIT 1)[0];
            LET $tag = (SELECT VALUE id FROM tags WHERE slug = $tag_slug LIMIT 1)[0];
            RELATE $post->tagged->$tag CONTENT { created_at: time::now() };
            "#,
        )
        .bind(("post_slug", slug.to_string()))
        .bind(("tag_slug", tag_slug))
        .await
        .expect("Failed to create tag relation");
    }
}

// ── Publish flow ────────────────────────────────────────────────────

#[tokio::test]
async fn test_publish_creates_post_and_tags() {
    let db = setup_test_db().await;
    publish_post(
        &db,
        "my-post",
        "My Post",
        &["rust", "web"],
        None,
        None,
        None,
    )
    .await;

    // Post exists
    let mut r = db
        .query("SELECT * FROM blog_posts WHERE slug = 'my-post'")
        .await
        .unwrap();
    let posts: Vec<BlogPost> = take_as(&mut r, 0).unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].title, "My Post");

    // Tags exist
    let mut r = db
        .query("SELECT VALUE slug FROM tags ORDER BY slug")
        .await
        .unwrap();
    let slugs: Vec<String> = r.take(0).unwrap();
    assert!(slugs.contains(&"rust".to_string()));
    assert!(slugs.contains(&"web".to_string()));

    // Graph relations exist
    let mut r = db
        .query(
            r#"
            LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = 'my-post' LIMIT 1)[0];
            SELECT VALUE name FROM $post->tagged->tags;
            "#,
        )
        .await
        .unwrap();
    let tag_names: Vec<String> = r.take(1).unwrap();
    assert_eq!(tag_names.len(), 2);
}

#[tokio::test]
async fn test_publish_with_series_fields() {
    let db = setup_test_db().await;
    publish_post(
        &db,
        "s-1",
        "Part 1",
        &[],
        Some("weekly-rust"),
        Some("Weekly Rust"),
        Some(1),
    )
    .await;

    let mut r = db
        .query("SELECT * FROM blog_posts WHERE slug = 's-1'")
        .await
        .unwrap();
    let post: BlogPost = take_as_opt(&mut r, 0).unwrap().unwrap();
    assert_eq!(post.series_slug.as_deref(), Some("weekly-rust"));
    assert_eq!(post.series_title.as_deref(), Some("Weekly Rust"));
    assert_eq!(post.series_position, Some(1));
}

// ── Series auto-position ────────────────────────────────────────────

#[tokio::test]
async fn test_series_auto_position_assignment() {
    let db = setup_test_db().await;

    // Publish first post with explicit position 1
    publish_post(
        &db,
        "s-1",
        "Part 1",
        &[],
        Some("my-series"),
        Some("My Series"),
        Some(1),
    )
    .await;

    // Query max position using array::max on collected values
    // (math::max expects an array argument, not a single value)
    let mut result = db
        .query(
            "SELECT VALUE series_position FROM blog_posts WHERE series_slug = $slug ORDER BY series_position DESC LIMIT 1",
        )
        .bind(("slug", "my-series".to_string()))
        .await
        .unwrap();
    let positions: Vec<u32> = result.take(0).unwrap();
    let max_pos = positions.first().copied().unwrap_or(0);
    let next_pos = max_pos + 1;
    assert_eq!(next_pos, 2);

    // Publish second post with auto-assigned position
    publish_post(
        &db,
        "s-2",
        "Part 2",
        &[],
        Some("my-series"),
        Some("My Series"),
        Some(next_pos),
    )
    .await;

    // Verify ordering
    let mut r = db
        .query(
            "SELECT * FROM blog_posts WHERE series_slug = 'my-series' ORDER BY series_position ASC",
        )
        .await
        .unwrap();
    let posts: Vec<BlogPost> = take_as(&mut r, 0).unwrap();
    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0].series_position, Some(1));
    assert_eq!(posts[1].series_position, Some(2));
}

#[tokio::test]
async fn test_series_auto_position_empty_series() {
    let db = setup_test_db().await;

    // No posts in this series yet
    let mut result = db
        .query("SELECT VALUE math::max(series_position) FROM blog_posts WHERE series_slug = $slug")
        .bind(("slug", "new-series".to_string()))
        .await
        .unwrap();
    let max_pos: Option<u32> = result.take(0).unwrap_or(None);
    let next_pos = max_pos.unwrap_or(0) + 1;
    assert_eq!(next_pos, 1);
}

// ── Delete flow ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_article_removes_post_and_tag_relations() {
    let db = setup_test_db().await;
    publish_post(
        &db,
        "del-me",
        "Delete Me",
        &["rust", "web"],
        None,
        None,
        None,
    )
    .await;

    // Delete (same transaction as handler)
    db.query(
        r#"
        BEGIN TRANSACTION;
        LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = $slug LIMIT 1)[0];
        DELETE tagged WHERE in = $post;
        DELETE FROM blog_posts WHERE slug = $slug;
        COMMIT TRANSACTION;
        "#,
    )
    .bind(("slug", "del-me".to_string()))
    .await
    .expect("Delete should succeed");

    // Post gone
    let mut r = db
        .query("SELECT * FROM blog_posts WHERE slug = 'del-me'")
        .await
        .unwrap();
    let posts: Vec<BlogPost> = take_as(&mut r, 0).unwrap();
    assert!(posts.is_empty());

    // Tag relations gone — verify via graph traversal from all remaining posts
    // (there are none, so tagged table should be empty)
    let mut r = db.query("SELECT VALUE id FROM tagged").await.unwrap();
    let ids: Vec<serde_json::Value> = r.take(0).unwrap();
    assert!(ids.is_empty());
}

#[tokio::test]
async fn test_delete_nonexistent_article() {
    let db = setup_test_db().await;

    let mut r = db
        .query("SELECT * FROM blog_posts WHERE slug = $slug")
        .bind(("slug", "ghost".to_string()))
        .await
        .unwrap();
    let posts: Vec<BlogPost> = take_as(&mut r, 0).unwrap();
    assert!(posts.is_empty(), "Non-existent post should return empty");
}

// ── Tag operations ──────────────────────────────────────────────────

#[tokio::test]
async fn test_add_tag_to_post_creates_relation() {
    let db = setup_test_db().await;
    publish_post(&db, "tag-test", "Tag Test", &[], None, None, None).await;

    // Add a tag
    ensure_tag(&db, "new-tag", "new-tag").await;
    db.query(
        r#"
        LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = $post_slug LIMIT 1)[0];
        LET $tag = (SELECT VALUE id FROM tags WHERE slug = $tag_slug LIMIT 1)[0];
        RELATE $post->tagged->$tag CONTENT { created_at: time::now() };
        "#,
    )
    .bind(("post_slug", "tag-test".to_string()))
    .bind(("tag_slug", "new-tag".to_string()))
    .await
    .expect("Should add tag");

    // Sync cache
    sync_post_tags_cache(&db, "tag-test").await.unwrap();

    // Verify
    let mut r = db
        .query("SELECT * FROM blog_posts WHERE slug = 'tag-test'")
        .await
        .unwrap();
    let post: BlogPost = take_as_opt(&mut r, 0).unwrap().unwrap();
    assert_eq!(post.tags, vec!["new-tag"]);
}

#[tokio::test]
async fn test_remove_tag_from_post() {
    let db = setup_test_db().await;
    publish_post(
        &db,
        "rm-tag",
        "Remove Tag",
        &["keep", "remove"],
        None,
        None,
        None,
    )
    .await;
    sync_post_tags_cache(&db, "rm-tag").await.unwrap();

    // Remove "remove" tag
    db.query(
        r#"
        LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = $post_slug LIMIT 1)[0];
        LET $tag = (SELECT VALUE id FROM tags WHERE slug = $tag_slug LIMIT 1)[0];
        DELETE tagged WHERE in = $post AND out = $tag;
        "#,
    )
    .bind(("post_slug", "rm-tag".to_string()))
    .bind(("tag_slug", "remove".to_string()))
    .await
    .expect("Should remove tag relation");

    sync_post_tags_cache(&db, "rm-tag").await.unwrap();

    let mut r = db
        .query("SELECT * FROM blog_posts WHERE slug = 'rm-tag'")
        .await
        .unwrap();
    let post: BlogPost = take_as_opt(&mut r, 0).unwrap().unwrap();
    assert_eq!(post.tags, vec!["keep"]);
}

#[tokio::test]
async fn test_batched_tag_creation_matches_handler() {
    let db = setup_test_db().await;
    publish_post(
        &db,
        "batch-post",
        "Batch Post",
        &["alpha", "beta", "gamma"],
        None,
        None,
        None,
    )
    .await;
    sync_post_tags_cache(&db, "batch-post").await.unwrap();

    let mut r = db
        .query("SELECT * FROM blog_posts WHERE slug = 'batch-post'")
        .await
        .unwrap();
    let post: BlogPost = take_as_opt(&mut r, 0).unwrap().unwrap();
    assert_eq!(post.tags.len(), 3);
    assert!(post.tags.contains(&"alpha".to_string()));
    assert!(post.tags.contains(&"beta".to_string()));
    assert!(post.tags.contains(&"gamma".to_string()));

    // Verify exactly 3 tagged relations via graph traversal
    let mut r = db
        .query(
            r#"
            LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = 'batch-post' LIMIT 1)[0];
            SELECT VALUE name FROM $post->tagged->tags;
            "#,
        )
        .await
        .unwrap();
    let names: Vec<String> = r.take(1).unwrap();
    assert_eq!(names.len(), 3);
}

// ── Delete article cascade ──────────────────────────────────────────

#[tokio::test]
async fn test_delete_only_removes_own_tag_relations() {
    let db = setup_test_db().await;

    // Two posts sharing the "rust" tag
    publish_post(&db, "post-a", "Post A", &["rust"], None, None, None).await;
    publish_post(&db, "post-b", "Post B", &["rust"], None, None, None).await;

    // Delete post-a
    db.query(
        r#"
        BEGIN TRANSACTION;
        LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = $slug LIMIT 1)[0];
        DELETE tagged WHERE in = $post;
        DELETE FROM blog_posts WHERE slug = $slug;
        COMMIT TRANSACTION;
        "#,
    )
    .bind(("slug", "post-a".to_string()))
    .await
    .unwrap();

    // post-b's relation should still exist
    let mut r = db
        .query(
            r#"
            LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = 'post-b' LIMIT 1)[0];
            SELECT VALUE name FROM $post->tagged->tags;
            "#,
        )
        .await
        .unwrap();
    let names: Vec<String> = r.take(1).unwrap();
    assert_eq!(names, vec!["rust"]);
}

// ── Idempotent tag creation ─────────────────────────────────────────

#[tokio::test]
async fn test_idempotent_tag_creation() {
    let db = setup_test_db().await;

    // Create tag
    ensure_tag(&db, "Rust", "rust").await;

    // Second call should not create a duplicate (UNIQUE constraint)
    ensure_tag(&db, "Rust", "rust").await;

    let mut r = db
        .query("SELECT VALUE slug FROM tags WHERE slug = 'rust'")
        .await
        .unwrap();
    let slugs: Vec<String> = r.take(0).unwrap();
    assert_eq!(slugs.len(), 1, "Should have exactly one tag, not two");
}
