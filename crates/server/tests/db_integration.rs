//! Integration tests for SurrealDB operations using in-memory backend.
//! These tests do NOT require fastembed — they skip VectorSearch entirely.
//!
//! Note: We use raw SQL for inserts because SurrealDB's SCHEMAFULL mode
//! requires native `datetime` values (via `time::now()` or `d"..."`), and
//! `db.create().content(rust_struct)` serializes chrono::DateTime as an
//! ISO 8601 string which gets rejected by the type checker.

use shared::{BlogPost, PortfolioItem};
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

/// Create an in-memory SurrealDB instance with schema initialized.
async fn setup_test_db() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(())
        .await
        .expect("Failed to create in-memory DB");
    db.use_ns("test")
        .use_db("test")
        .await
        .expect("Failed to select ns/db");
    server::services::db::init_schema(&db)
        .await
        .expect("Failed to init schema");
    db
}

/// Insert a blog post using raw SQL (compatible with SCHEMAFULL datetime).
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
            tags: ["test"],
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

/// Insert a portfolio item using raw SQL.
async fn insert_portfolio_item_sql(
    db: &Surreal<surrealdb::engine::local::Db>,
    slug: &str,
    title: &str,
) {
    db.query(
        r##"
        CREATE portfolio_items CONTENT {
            slug: $slug,
            title: $title,
            description: "A test project",
            content: "# Details",
            html_content: "<h1>Details</h1>",
            tech_stack: ["Rust", "Nix"],
            link: "https://github.com/test",
            demo: NONE,
            image_url: NONE,
            date: time::now(),
            featured: true,
            order: 0
        };
        "##,
    )
    .bind(("slug", slug.to_string()))
    .bind(("title", title.to_string()))
    .await
    .expect("Failed to insert portfolio item");
}

#[tokio::test]
async fn test_init_schema_creates_tables() {
    let db = setup_test_db().await;

    // Insert a blog post via raw SQL — should succeed if schema was created
    insert_blog_post_sql(&db, "test-post", "Test Post").await;

    let posts: Vec<BlogPost> = db
        .query("SELECT * FROM blog_posts WHERE slug = 'test-post'")
        .await
        .unwrap()
        .take(0)
        .unwrap();

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].slug, "test-post");
    assert_eq!(posts[0].title, "Test Post");
}

#[tokio::test]
async fn test_seed_sample_data() {
    let db = setup_test_db().await;

    server::services::db::seed_sample_data(&db)
        .await
        .expect("Should seed data");

    let posts: Vec<BlogPost> = db
        .query("SELECT * FROM blog_posts")
        .await
        .unwrap()
        .take(0)
        .unwrap();

    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].slug, "welcome-to-my-blog");
    assert!(posts[0].published);
}

#[tokio::test]
async fn test_seed_data_idempotent() {
    let db = setup_test_db().await;

    server::services::db::seed_sample_data(&db).await.unwrap();
    server::services::db::seed_sample_data(&db).await.unwrap();

    let posts: Vec<BlogPost> = db
        .query("SELECT * FROM blog_posts")
        .await
        .unwrap()
        .take(0)
        .unwrap();

    assert_eq!(posts.len(), 1);
}

#[tokio::test]
async fn test_unique_slug_constraint() {
    let db = setup_test_db().await;

    insert_blog_post_sql(&db, "duplicate", "First").await;

    // Second insert with same slug should fail due to UNIQUE index
    let result = db
        .query(
            r#"
            CREATE blog_posts CONTENT {
                slug: "duplicate",
                title: "Second",
                content: "dup",
                html_content: "<p>dup</p>",
                published_at: time::now(),
                author: "Test",
                tags: ["test"],
                featured: false,
                published: true,
                reading_time_minutes: 1,
                embedding: NONE
            };
            "#,
        )
        .await;

    // SurrealDB returns the result in the response — check for error
    // With UNIQUE index, duplicates cause a database error
    match result {
        Err(_) => {} // Expected: query-level error
        Ok(mut response) => {
            // Some SurrealDB versions embed the error in the response
            let take_result: Result<Vec<BlogPost>, _> = response.take(0);
            assert!(
                take_result.is_err(),
                "Duplicate slug should produce an error"
            );
        }
    }
}

#[tokio::test]
async fn test_blog_post_query_by_slug() {
    let db = setup_test_db().await;

    insert_blog_post_sql(&db, "my-post", "My Post").await;

    let result: Option<BlogPost> = db
        .query("SELECT * FROM blog_posts WHERE slug = $slug LIMIT 1")
        .bind(("slug", "my-post"))
        .await
        .unwrap()
        .take(0)
        .unwrap();

    assert!(result.is_some());
    assert_eq!(result.unwrap().title, "My Post");
}

#[tokio::test]
async fn test_portfolio_item_crud() {
    let db = setup_test_db().await;

    insert_portfolio_item_sql(&db, "my-project", "My Project").await;

    // Query back
    let queried: Option<PortfolioItem> = db
        .query("SELECT * FROM portfolio_items WHERE slug = $slug LIMIT 1")
        .bind(("slug", "my-project"))
        .await
        .unwrap()
        .take(0)
        .unwrap();

    assert!(queried.is_some());
    let item = queried.unwrap();
    assert_eq!(item.slug, "my-project");
    assert_eq!(item.title, "My Project");
    assert_eq!(item.tech_stack, vec!["Rust", "Nix"]);
    assert!(item.featured);
}
