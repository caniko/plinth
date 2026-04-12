//! Integration tests for the database migration system.

use plinth_server::services::migrations;
use surrealdb::Surreal;
use surrealdb::engine::local::Mem;

async fn fresh_db() -> Surreal<surrealdb::engine::local::Db> {
    let db = Surreal::new::<Mem>(())
        .await
        .expect("Failed to create in-memory DB");
    db.use_ns("test")
        .use_db("test")
        .await
        .expect("Failed to select ns/db");
    db
}

#[tokio::test]
async fn test_migration_creates_all_tables() {
    let db = fresh_db().await;
    migrations::run_migrations(&db).await.unwrap();

    // Verify we can insert into all 6 main tables
    db.query(
        r#"
        CREATE blog_posts CONTENT {
            slug: "t1", title: "T", content: "", html_content: "",
            published_at: time::now(), author: "T", tags: [],
            featured: false, published: true, reading_time_minutes: 1,
            embedding: NONE
        };
        "#,
    )
    .await
    .expect("blog_posts should exist");

    db.query(
        r#"
        CREATE portfolio_items CONTENT {
            slug: "t1", title: "T", description: "T",
            tech_stack: [], date: time::now(), featured: false, order: 0
        };
        "#,
    )
    .await
    .expect("portfolio_items should exist");

    db.query(
        r#"
        CREATE site_content CONTENT {
            key: "t1", content: "T", html_content: "T", updated_at: time::now()
        };
        "#,
    )
    .await
    .expect("site_content should exist");

    db.query(
        r#"
        CREATE tags CONTENT {
            name: "T", slug: "t1", created_at: time::now()
        };
        "#,
    )
    .await
    .expect("tags should exist");

    db.query(
        r#"
        CREATE todos CONTENT {
            slug: "t1", title: "T", description: "T", tags: [],
            completed: false, completed_at: NONE,
            created_at: time::now(), order: 0
        };
        "#,
    )
    .await
    .expect("todos should exist");

    // Verify relation tables exist by creating relations
    db.query(
        r#"
        LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = 't1' LIMIT 1)[0];
        LET $tag = (SELECT VALUE id FROM tags WHERE slug = 't1' LIMIT 1)[0];
        RELATE $post->tagged->$tag CONTENT { created_at: time::now() };
        "#,
    )
    .await
    .expect("tagged relation should work");

    db.query(
        r#"
        LET $todo = (SELECT VALUE id FROM todos WHERE slug = 't1' LIMIT 1)[0];
        LET $tag = (SELECT VALUE id FROM tags WHERE slug = 't1' LIMIT 1)[0];
        RELATE $todo->todo_tagged->$tag CONTENT { created_at: time::now() };
        "#,
    )
    .await
    .expect("todo_tagged relation should work");
}

#[tokio::test]
async fn test_migration_v2_adds_series_fields() {
    let db = fresh_db().await;
    migrations::run_migrations(&db).await.unwrap();

    // Insert a post with series fields (added by blog brick migration v2)
    db.query(
        r#"
        CREATE blog_posts CONTENT {
            slug: "series-post", title: "Part 1",
            content: "c", html_content: "h",
            published_at: time::now(), author: "A", tags: [],
            featured: false, published: true, reading_time_minutes: 1,
            embedding: NONE,
            series_slug: "my-series",
            series_title: "My Series",
            series_position: 1
        };
        "#,
    )
    .await
    .expect("Should accept series fields after migration v2");

    let mut response = db
        .query("SELECT VALUE series_slug FROM blog_posts WHERE slug = 'series-post'")
        .await
        .unwrap();
    let slugs: Vec<Option<String>> = response.take(0).unwrap();
    assert_eq!(slugs[0], Some("my-series".to_string()));
}

#[tokio::test]
async fn test_migration_status_all_applied() {
    let db = fresh_db().await;
    migrations::run_migrations(&db).await.unwrap();

    let statuses = migrations::migration_status(&db).await.unwrap();
    assert!(!statuses.is_empty());
    for (brick, version, name, applied) in &statuses {
        assert!(
            *applied,
            "Migration {}/v{} '{}' should be applied",
            brick, version, name
        );
    }
}

#[tokio::test]
async fn test_partial_migration_then_full() {
    let db = fresh_db().await;

    // Run migrations
    let first_run = migrations::run_migrations(&db).await.unwrap();
    assert!(first_run > 0);

    // Run again — should apply 0
    let second_run = migrations::run_migrations(&db).await.unwrap();
    assert_eq!(second_run, 0);
}

#[tokio::test]
async fn test_latest_available_version_counts_all_migrations() {
    let latest = migrations::latest_available_version();
    // core(1) + blog(2) + portfolio(1) + todo(1) = 5
    assert!(
        latest >= 5,
        "Should count all brick migrations, got {}",
        latest
    );
}
