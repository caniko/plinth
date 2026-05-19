mod common;

use kameo::actor::Spawn;
use plinth_server::actors::core_cache::{CoreCache, GetAllTags, GetSiteContent};
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn site_content_crud_round_trips_through_cache(pool: PgPool) {
    sqlx::query(
        r#"
        INSERT INTO site_content (key, title, content, html_content)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind("about")
    .bind("About")
    .bind("Plain about body")
    .bind("<p>Plain about body</p>")
    .execute(&pool)
    .await
    .expect("insert site content");

    let core_cache = CoreCache::spawn(CoreCache::new(pool.clone()));
    let cached = core_cache
        .ask(GetSiteContent("about".to_string()))
        .await
        .expect("ask core cache")
        .expect("site content exists");
    assert_eq!(cached.title.as_deref(), Some("About"));
    assert_eq!(cached.content, "Plain about body");

    sqlx::query(
        r#"
        UPDATE site_content
        SET title = $1, content = $2, html_content = $3
        WHERE key = $4
        "#,
    )
    .bind("About updated")
    .bind("Updated body")
    .bind("<p>Updated body</p>")
    .bind("about")
    .execute(&pool)
    .await
    .expect("update site content");

    let row_title: String = sqlx::query_scalar("SELECT title FROM site_content WHERE key = $1")
        .bind("about")
        .fetch_one(&pool)
        .await
        .expect("read updated content");
    assert_eq!(row_title, "About updated");

    let deleted = sqlx::query("DELETE FROM site_content WHERE key = $1")
        .bind("about")
        .execute(&pool)
        .await
        .expect("delete site content")
        .rows_affected();
    assert_eq!(deleted, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn tags_are_unique_and_counts_include_posts_and_todos(pool: PgPool) {
    let rust_id = common::ensure_tag(&pool, "Rust")
        .await
        .expect("create Rust tag");
    let rust_again = common::ensure_tag(&pool, "Rust")
        .await
        .expect("upsert Rust tag");
    assert_eq!(rust_id, rust_again);

    let post_id = common::insert_blog_post(&pool, "rust-post", "Rust Post", &[])
        .await
        .expect("insert blog post");
    sqlx::query("INSERT INTO blog_post_tags (post_id, tag_id) VALUES ($1, $2)")
        .bind(post_id)
        .bind(rust_id)
        .execute(&pool)
        .await
        .expect("attach post tag");
    plinth_server::services::db::sync_post_tags_cache(&pool, "rust-post")
        .await
        .expect("sync post tag cache");

    let todo_id = common::insert_todo(&pool, "rust-todo", "Rust Todo", 0, false, &[])
        .await
        .expect("insert todo");
    common::attach_tag_to_todo(&pool, todo_id, "Rust")
        .await
        .expect("attach todo tag");

    let core_cache = CoreCache::spawn(CoreCache::new(pool.clone()));
    let tags = core_cache.ask(GetAllTags).await.expect("ask core cache");
    let rust = tags
        .iter()
        .find(|tag| tag.slug == "rust")
        .expect("Rust tag");
    assert_eq!(rust.name, "Rust");
    assert_eq!(rust.post_count, 1);
    assert_eq!(rust.todo_count, 1);
}
