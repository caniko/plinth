//! Integration tests for tag graph relations using in-memory SurrealDB.
//!
//! Tests the tagged/todo_tagged RELATION tables and the sync_*_tags_cache helpers.

use plinth_server::db_helpers::take_as;
use plinth_server::services::db::{sync_post_tags_cache, sync_todo_tags_cache};
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
    plinth_server::services::db::init_schema(&db)
        .await
        .expect("Failed to init schema");
    db
}

async fn insert_blog_post_sql(db: &Surreal<surrealdb::engine::local::Db>, slug: &str, title: &str) {
    db.query(
        r#"
        CREATE blog_posts CONTENT {
            slug: $slug,
            title: $title,
            content: "Hello world",
            html_content: "<p>Hello world</p>",
            published_at: time::now(),
            author: "Test",
            tags: [],
            featured: false,
            published: true,
            reading_time_minutes: 1,
            embedding: NONE
        };
        "#,
    )
    .bind(("slug", slug.to_string()))
    .bind(("title", title.to_string()))
    .await
    .expect("Failed to insert blog post");
}

async fn insert_todo_sql(db: &Surreal<surrealdb::engine::local::Db>, slug: &str, title: &str) {
    db.query(
        r#"
        CREATE todos CONTENT {
            slug: $slug,
            title: $title,
            description: "Test",
            tags: [],
            completed: false,
            completed_at: NONE,
            created_at: time::now(),
            order: 0
        };
        "#,
    )
    .bind(("slug", slug.to_string()))
    .bind(("title", title.to_string()))
    .await
    .expect("Failed to insert todo");
}

async fn insert_tag_sql(db: &Surreal<surrealdb::engine::local::Db>, name: &str, slug: &str) {
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
    .expect("Failed to insert tag");
}

#[tokio::test]
async fn test_tag_creation() {
    let db = setup_test_db().await;
    insert_tag_sql(&db, "Rust", "rust").await;

    let mut response = db
        .query("SELECT VALUE name FROM tags WHERE slug = 'rust'")
        .await
        .unwrap();
    let names: Vec<String> = response.take(0).unwrap();
    assert_eq!(names, vec!["Rust"]);
}

#[tokio::test]
async fn test_tag_unique_name() {
    let db = setup_test_db().await;
    insert_tag_sql(&db, "Rust", "rust").await;

    let result = db
        .query(
            r#"CREATE tags CONTENT {
                name: "Rust",
                slug: "rust-2",
                created_at: time::now()
            }"#,
        )
        .await;

    match result {
        Err(_) => {}
        Ok(mut response) => {
            let take_result: Result<Vec<serde_json::Value>, _> = response.take(0);
            assert!(take_result.is_err(), "Duplicate tag name should fail");
        }
    }
}

#[tokio::test]
async fn test_tag_post_relation() {
    let db = setup_test_db().await;
    insert_blog_post_sql(&db, "my-post", "My Post").await;
    insert_tag_sql(&db, "Rust", "rust").await;

    // Create graph relation
    db.query(
        r#"
        LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = $post_slug LIMIT 1)[0];
        LET $tag = (SELECT VALUE id FROM tags WHERE slug = $tag_slug LIMIT 1)[0];
        RELATE $post->tagged->$tag CONTENT { created_at: time::now() };
        "#,
    )
    .bind(("post_slug", "my-post".to_string()))
    .bind(("tag_slug", "rust".to_string()))
    .await
    .expect("Should create relation");

    // Verify graph traversal
    let mut response = db
        .query(
            r#"
            LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = 'my-post' LIMIT 1)[0];
            SELECT VALUE name FROM $post->tagged->tags;
            "#,
        )
        .await
        .unwrap();
    let tag_names: Vec<String> = response.take(1).unwrap();
    assert_eq!(tag_names, vec!["Rust"]);
}

#[tokio::test]
async fn test_tag_todo_relation() {
    let db = setup_test_db().await;
    insert_todo_sql(&db, "my-todo", "My Todo").await;
    insert_tag_sql(&db, "Travel", "travel").await;

    db.query(
        r#"
        LET $todo = (SELECT VALUE id FROM todos WHERE slug = $todo_slug LIMIT 1)[0];
        LET $tag = (SELECT VALUE id FROM tags WHERE slug = $tag_slug LIMIT 1)[0];
        RELATE $todo->todo_tagged->$tag CONTENT { created_at: time::now() };
        "#,
    )
    .bind(("todo_slug", "my-todo".to_string()))
    .bind(("tag_slug", "travel".to_string()))
    .await
    .expect("Should create todo-tag relation");

    let mut response = db
        .query(
            r#"
            LET $todo = (SELECT VALUE id FROM todos WHERE slug = 'my-todo' LIMIT 1)[0];
            SELECT VALUE name FROM $todo->todo_tagged->tags;
            "#,
        )
        .await
        .unwrap();
    let tag_names: Vec<String> = response.take(1).unwrap();
    assert_eq!(tag_names, vec!["Travel"]);
}

#[tokio::test]
async fn test_sync_post_tags_cache() {
    let db = setup_test_db().await;
    insert_blog_post_sql(&db, "tagged-post", "Tagged Post").await;
    insert_tag_sql(&db, "Rust", "rust").await;
    insert_tag_sql(&db, "Web", "web").await;

    // Create relations
    db.query(
        r#"
        LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = 'tagged-post' LIMIT 1)[0];
        LET $tag1 = (SELECT VALUE id FROM tags WHERE slug = 'rust' LIMIT 1)[0];
        LET $tag2 = (SELECT VALUE id FROM tags WHERE slug = 'web' LIMIT 1)[0];
        RELATE $post->tagged->$tag1 CONTENT { created_at: time::now() };
        RELATE $post->tagged->$tag2 CONTENT { created_at: time::now() };
        "#,
    )
    .await
    .expect("Should create relations");

    // Before sync, post.tags should be empty
    let mut response = db
        .query("SELECT * FROM blog_posts WHERE slug = 'tagged-post'")
        .await
        .unwrap();
    let posts: Vec<BlogPost> = take_as(&mut response, 0).unwrap();
    assert!(posts[0].tags.is_empty());

    // Sync
    sync_post_tags_cache(&db, "tagged-post")
        .await
        .expect("Sync should succeed");

    // After sync, post.tags should contain both tag names
    let mut response = db
        .query("SELECT * FROM blog_posts WHERE slug = 'tagged-post'")
        .await
        .unwrap();
    let posts: Vec<BlogPost> = take_as(&mut response, 0).unwrap();
    assert_eq!(posts[0].tags.len(), 2);
    assert!(posts[0].tags.contains(&"Rust".to_string()));
    assert!(posts[0].tags.contains(&"Web".to_string()));
}

#[tokio::test]
async fn test_sync_todo_tags_cache() {
    let db = setup_test_db().await;
    insert_todo_sql(&db, "tagged-todo", "Tagged Todo").await;
    insert_tag_sql(&db, "Travel", "travel").await;

    db.query(
        r#"
        LET $todo = (SELECT VALUE id FROM todos WHERE slug = 'tagged-todo' LIMIT 1)[0];
        LET $tag = (SELECT VALUE id FROM tags WHERE slug = 'travel' LIMIT 1)[0];
        RELATE $todo->todo_tagged->$tag CONTENT { created_at: time::now() };
        "#,
    )
    .await
    .expect("Should create relation");

    sync_todo_tags_cache(&db, "tagged-todo")
        .await
        .expect("Sync should succeed");

    let mut response = db
        .query("SELECT VALUE tags FROM todos WHERE slug = 'tagged-todo'")
        .await
        .unwrap();
    let tags_arrays: Vec<Vec<String>> = response.take(0).unwrap();
    assert_eq!(tags_arrays[0], vec!["Travel"]);
}

#[tokio::test]
async fn test_tag_removal() {
    let db = setup_test_db().await;
    insert_blog_post_sql(&db, "rm-tag-post", "Post").await;
    insert_tag_sql(&db, "Remove", "remove").await;

    // Add relation
    db.query(
        r#"
        LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = 'rm-tag-post' LIMIT 1)[0];
        LET $tag = (SELECT VALUE id FROM tags WHERE slug = 'remove' LIMIT 1)[0];
        RELATE $post->tagged->$tag CONTENT { created_at: time::now() };
        "#,
    )
    .await
    .unwrap();

    // Remove relation
    db.query(
        r#"
        LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = 'rm-tag-post' LIMIT 1)[0];
        LET $tag = (SELECT VALUE id FROM tags WHERE slug = 'remove' LIMIT 1)[0];
        DELETE tagged WHERE in = $post AND out = $tag;
        "#,
    )
    .await
    .unwrap();

    // Verify graph traversal returns empty
    let mut response = db
        .query(
            r#"
            LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = 'rm-tag-post' LIMIT 1)[0];
            SELECT VALUE name FROM $post->tagged->tags;
            "#,
        )
        .await
        .unwrap();
    let tag_names: Vec<String> = response.take(1).unwrap();
    assert!(tag_names.is_empty());
}
