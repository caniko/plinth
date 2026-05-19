mod common;

use kameo::actor::Spawn;
use plinth_server::bricks::todo::cache::{GetAllTodos, GetTodoItem, GetTodosByTag, TodoCache};
use sqlx::{Error, PgPool};

#[sqlx::test(migrations = "./migrations")]
async fn todo_lifecycle_persists_tags_completion_ordering_and_deletes(pool: PgPool) {
    let first_id = common::insert_todo(&pool, "first-todo", "First Todo", 20, false, &[])
        .await
        .expect("insert first todo");
    common::attach_tag_to_todo(&pool, first_id, "work")
        .await
        .expect("attach work tag");

    let second_id = common::insert_todo(&pool, "second-todo", "Second Todo", 10, false, &[])
        .await
        .expect("insert second todo");
    common::attach_tag_to_todo(&pool, second_id, "work")
        .await
        .expect("attach second work tag");
    common::attach_tag_to_todo(&pool, second_id, "home")
        .await
        .expect("attach second home tag");

    let duplicate = common::insert_todo(&pool, "first-todo", "Duplicate", 0, false, &[]).await;
    assert!(matches!(duplicate, Err(Error::Database(_))));

    assert_eq!(
        common::todo_tag_names(&pool, "second-todo")
            .await
            .expect("read todo tag relations"),
        vec!["home", "work"]
    );
    assert_eq!(
        common::column_text_array(&pool, "todos", "second-todo")
            .await
            .expect("read denormalized todo tags"),
        vec!["home", "work"]
    );

    sqlx::query(
        r#"
        UPDATE todos
        SET completed = true, completed_at = now(), "order" = 5
        WHERE slug = $1
        "#,
    )
    .bind("first-todo")
    .execute(&pool)
    .await
    .expect("complete first todo");

    let todo_cache = TodoCache::spawn(TodoCache::new(pool.clone()));
    let all = todo_cache.ask(GetAllTodos).await.expect("ask todo cache");
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].slug, "second-todo");
    assert!(!all[0].completed);
    assert_eq!(all[1].slug, "first-todo");
    assert!(all[1].completed);
    assert!(all[1].completed_at.is_some());

    let work_todos = todo_cache
        .ask(GetTodosByTag("work".to_string()))
        .await
        .expect("ask todo cache");
    assert_eq!(work_todos.len(), 2);

    let second = todo_cache
        .ask(GetTodoItem("second-todo".to_string()))
        .await
        .expect("ask todo cache")
        .expect("second todo exists");
    assert_eq!(second.title, "Second Todo");

    let deleted = sqlx::query("DELETE FROM todos WHERE slug = $1")
        .bind("second-todo")
        .execute(&pool)
        .await
        .expect("delete second todo")
        .rows_affected();
    assert_eq!(deleted, 1);

    let orphaned_relations: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM todo_tags tt
        LEFT JOIN todos td ON td.id = tt.todo_id
        WHERE td.id IS NULL
        "#,
    )
    .fetch_one(&pool)
    .await
    .expect("count orphaned todo tag relations");
    assert_eq!(orphaned_relations, 0);
}
