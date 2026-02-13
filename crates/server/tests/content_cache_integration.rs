//! Integration tests for the ContentCache actor using in-memory SurrealDB.
//! These tests do NOT require fastembed — they only use ContentCache (not VectorSearch).

use kameo::actor::Spawn;
use server::actors::content_cache::*;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;

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

#[tokio::test]
async fn test_content_cache_get_all_empty() {
    let db = setup_test_db().await;
    let cache = ContentCache::spawn(ContentCache::new(db));

    let result = cache.ask(GetAllBlogPosts).await;
    let posts = result.unwrap();
    assert!(posts.is_empty());
}

#[tokio::test]
async fn test_content_cache_get_blog_post_not_found() {
    let db = setup_test_db().await;
    let cache = ContentCache::spawn(ContentCache::new(db));

    let result = cache.ask(GetBlogPost("nonexistent".to_string())).await;
    let post = result.unwrap();
    assert!(post.is_none());
}

#[tokio::test]
async fn test_content_cache_returns_seeded_data() {
    let db = setup_test_db().await;
    server::services::db::seed_sample_data(&db).await.unwrap();

    let cache = ContentCache::spawn(ContentCache::new(db));

    let result = cache.ask(GetAllBlogPosts).await;
    let posts = result.unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].slug, "welcome-to-my-blog");
}

#[tokio::test]
async fn test_content_cache_get_single_post() {
    let db = setup_test_db().await;
    server::services::db::seed_sample_data(&db).await.unwrap();

    let cache = ContentCache::spawn(ContentCache::new(db));

    let result = cache
        .ask(GetBlogPost("welcome-to-my-blog".to_string()))
        .await;
    let post = result.unwrap();
    assert!(post.is_some());
    assert_eq!(post.unwrap().title, "Welcome to My Blog");
}

#[tokio::test]
async fn test_content_cache_invalidation() {
    let db = setup_test_db().await;
    server::services::db::seed_sample_data(&db).await.unwrap();

    let cache = ContentCache::spawn(ContentCache::new(db));

    // Prime cache
    let posts = cache.ask(GetAllBlogPosts).await.unwrap();
    assert_eq!(posts.len(), 1);

    // Invalidate
    let _ = cache.ask(InvalidateCache).await;

    // Should re-query DB (same result, but exercises the invalidation path)
    let posts = cache.ask(GetAllBlogPosts).await.unwrap();
    assert_eq!(posts.len(), 1);
}

#[tokio::test]
async fn test_content_cache_portfolio_empty() {
    let db = setup_test_db().await;
    let cache = ContentCache::spawn(ContentCache::new(db));

    let result = cache.ask(GetAllPortfolioItems).await;
    let items = result.unwrap();
    assert!(items.is_empty());
}

#[tokio::test]
async fn test_content_cache_portfolio_seeded() {
    let db = setup_test_db().await;
    server::services::db::seed_sample_data(&db).await.unwrap();

    let cache = ContentCache::spawn(ContentCache::new(db));

    let result = cache.ask(GetAllPortfolioItems).await;
    let items = result.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].slug, "sample-project");
}
