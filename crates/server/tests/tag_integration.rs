mod common;

use kameo::actor::Spawn;
use plinth_server::{
    actors::core_cache::{CoreCache, GetAllTags},
    bricks::{
        blog::cache::{BlogCache, GetPostsByTag},
        todo::cache::{GetTodosByTag, TodoCache},
    },
};
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn tags_attach_to_posts_and_todos_and_drive_filtered_lists(pool: PgPool) {
    common::insert_blog_post(&pool, "tagged-post", "Tagged Post", &[])
        .await
        .expect("insert tagged post");
    plinth_server::services::db::create_tags_for_post(
        &pool,
        "tagged-post",
        &["Shared".to_string(), "Blog Only".to_string()],
    )
    .await
    .expect("attach post tags");

    let todo_id = common::insert_todo(&pool, "tagged-todo", "Tagged Todo", 0, false, &[])
        .await
        .expect("insert tagged todo");
    common::attach_tag_to_todo(&pool, todo_id, "Shared")
        .await
        .expect("attach shared todo tag");
    common::attach_tag_to_todo(&pool, todo_id, "Todo Only")
        .await
        .expect("attach todo-only tag");

    assert_eq!(
        common::blog_tag_names(&pool, "tagged-post")
            .await
            .expect("read blog tag names"),
        vec!["Blog Only", "Shared"]
    );
    assert_eq!(
        common::todo_tag_names(&pool, "tagged-todo")
            .await
            .expect("read todo tag names"),
        vec!["Shared", "Todo Only"]
    );

    let core_cache = CoreCache::spawn(CoreCache::new(pool.clone()));
    let tags = core_cache.ask(GetAllTags).await.expect("ask core cache");
    let shared = tags
        .iter()
        .find(|tag| tag.slug == "shared")
        .expect("shared tag");
    assert_eq!(shared.post_count, 1);
    assert_eq!(shared.todo_count, 1);

    let blog_cache = BlogCache::spawn(BlogCache::new(pool.clone()));
    let shared_posts = blog_cache
        .ask(GetPostsByTag("Shared".to_string()))
        .await
        .expect("ask blog cache");
    assert_eq!(shared_posts.len(), 1);
    assert_eq!(shared_posts[0].slug, "tagged-post");

    let todo_cache = TodoCache::spawn(TodoCache::new(pool.clone()));
    let shared_todos = todo_cache
        .ask(GetTodosByTag("Shared".to_string()))
        .await
        .expect("ask todo cache");
    assert_eq!(shared_todos.len(), 1);
    assert_eq!(shared_todos[0].slug, "tagged-todo");
}
