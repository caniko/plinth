mod common;

use kameo::actor::Spawn;
use plinth_server::bricks::blog::cache::{BlogCache, GetAllBlogPosts, GetBlogPost, GetPostsByTag};
use sqlx::{Error, PgPool};

#[sqlx::test(migrations = "./migrations")]
async fn blog_post_lifecycle_persists_tags_orders_lists_and_deletes(pool: PgPool) {
    common::insert_blog_post(&pool, "first-post", "First Post", &[])
        .await
        .expect("insert first post");
    plinth_server::services::db::create_tags_for_post(
        &pool,
        "first-post",
        &["rust".to_string(), "postgres".to_string()],
    )
    .await
    .expect("attach first post tags");

    common::insert_blog_post(&pool, "second-post", "Second Post", &["rust"])
        .await
        .expect("insert second post");
    plinth_server::services::db::create_tags_for_post(&pool, "second-post", &["rust".to_string()])
        .await
        .expect("attach second post tags");

    let duplicate = common::insert_blog_post(&pool, "first-post", "Duplicate", &[]).await;
    assert!(matches!(duplicate, Err(Error::Database(_))));

    let first_tags = common::blog_tag_names(&pool, "first-post")
        .await
        .expect("read first post tag relations");
    assert_eq!(first_tags, vec!["postgres", "rust"]);
    assert_eq!(
        common::column_text_array(&pool, "blog_posts", "first-post")
            .await
            .expect("read denormalized post tags"),
        vec!["postgres", "rust"]
    );

    let blog_cache = BlogCache::spawn(BlogCache::new(pool.clone()));
    let listed = blog_cache
        .ask(GetAllBlogPosts)
        .await
        .expect("ask blog cache");
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].slug, "second-post");
    assert_eq!(listed[1].slug, "first-post");

    let rust_posts = blog_cache
        .ask(GetPostsByTag("rust".to_string()))
        .await
        .expect("ask blog cache");
    assert_eq!(rust_posts.len(), 2);

    let post = blog_cache
        .ask(GetBlogPost("first-post".to_string()))
        .await
        .expect("ask blog cache")
        .expect("blog post exists");
    assert_eq!(post.title, "First Post");

    let deleted = sqlx::query("DELETE FROM blog_posts WHERE slug = $1")
        .bind("first-post")
        .execute(&pool)
        .await
        .expect("delete first post")
        .rows_affected();
    assert_eq!(deleted, 1);

    let orphaned_relations: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM blog_post_tags bpt
        LEFT JOIN blog_posts bp ON bp.id = bpt.post_id
        WHERE bp.id IS NULL
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("count orphaned blog tag relations");
    assert_eq!(orphaned_relations, 0);
}
