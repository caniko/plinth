//! Integration tests for TODO/bucket-list CRUD operations using in-memory SurrealDB.
//!
//! Uses raw SQL for inserts (SCHEMAFULL datetime constraint).

use plinth_server::db_helpers::{take_as, take_as_opt};
use plinth_shared::TodoItem;
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

async fn insert_todo_sql(
    db: &Surreal<surrealdb::engine::local::Db>,
    slug: &str,
    title: &str,
    order: i64,
) {
    db.query(
        r#"
        CREATE todos CONTENT {
            slug: $slug,
            title: $title,
            description: "A test todo",
            content: NONE,
            html_content: NONE,
            tags: [],
            completed: false,
            completed_at: NONE,
            created_at: time::now(),
            order: $order
        };
        "#,
    )
    .bind(("slug", slug.to_string()))
    .bind(("title", title.to_string()))
    .bind(("order", order))
    .await
    .expect("Failed to insert todo");
}

#[tokio::test]
async fn test_todo_insert_and_query() {
    let db = setup_test_db().await;
    insert_todo_sql(&db, "learn-rust", "Learn Rust", 0).await;

    let mut response = db
        .query("SELECT * FROM todos WHERE slug = $slug LIMIT 1")
        .bind(("slug", "learn-rust"))
        .await
        .unwrap();
    let item: Option<TodoItem> = take_as_opt(&mut response, 0).unwrap();

    assert!(item.is_some());
    let item = item.unwrap();
    assert_eq!(item.slug, "learn-rust");
    assert_eq!(item.title, "Learn Rust");
    assert_eq!(item.description, "A test todo");
    assert!(!item.completed);
    assert!(item.completed_at.is_none());
}

#[tokio::test]
async fn test_todo_unique_slug() {
    let db = setup_test_db().await;
    insert_todo_sql(&db, "unique-todo", "First", 0).await;

    let result = db
        .query(
            r#"
            CREATE todos CONTENT {
                slug: "unique-todo",
                title: "Second",
                description: "dup",
                tags: [],
                completed: false,
                completed_at: NONE,
                created_at: time::now(),
                order: 0
            };
            "#,
        )
        .await;

    match result {
        Err(_) => {}
        Ok(mut response) => {
            let take_result: Result<Vec<serde_json::Value>, _> = response.take(0);
            assert!(
                take_result.is_err(),
                "Duplicate todo slug should produce an error"
            );
        }
    }
}

#[tokio::test]
async fn test_todo_update_fields() {
    let db = setup_test_db().await;
    insert_todo_sql(&db, "update-me", "Original Title", 0).await;

    // Update using parameterized query (same pattern as the fixed update_todo handler)
    db.query(
        r#"UPDATE todos SET
            title = $title,
            description = $description,
            order = $order
        WHERE slug = $slug"#,
    )
    .bind(("slug", "update-me".to_string()))
    .bind(("title", "Updated Title".to_string()))
    .bind(("description", "Updated description".to_string()))
    .bind(("order", 5i64))
    .await
    .expect("Update should succeed");

    let mut response = db
        .query("SELECT * FROM todos WHERE slug = $slug LIMIT 1")
        .bind(("slug", "update-me"))
        .await
        .unwrap();
    let item: Option<TodoItem> = take_as_opt(&mut response, 0).unwrap();

    let item = item.expect("Todo should still exist after update");
    assert_eq!(item.title, "Updated Title");
    assert_eq!(item.description, "Updated description");
    assert_eq!(item.order, 5);
}

#[tokio::test]
async fn test_todo_completion_toggle() {
    let db = setup_test_db().await;
    insert_todo_sql(&db, "toggle-me", "Toggle Todo", 0).await;

    // Mark as completed
    db.query("UPDATE todos SET completed = true, completed_at = time::now() WHERE slug = $slug")
        .bind(("slug", "toggle-me".to_string()))
        .await
        .expect("Should mark completed");

    let mut response = db
        .query("SELECT * FROM todos WHERE slug = $slug LIMIT 1")
        .bind(("slug", "toggle-me"))
        .await
        .unwrap();
    let item: TodoItem = take_as_opt(&mut response, 0).unwrap().unwrap();
    assert!(item.completed);
    assert!(item.completed_at.is_some());

    // Mark as not completed
    db.query("UPDATE todos SET completed = false, completed_at = NONE WHERE slug = $slug")
        .bind(("slug", "toggle-me".to_string()))
        .await
        .expect("Should mark not completed");

    let mut response = db
        .query("SELECT * FROM todos WHERE slug = $slug LIMIT 1")
        .bind(("slug", "toggle-me"))
        .await
        .unwrap();
    let item: TodoItem = take_as_opt(&mut response, 0).unwrap().unwrap();
    assert!(!item.completed);
    assert!(item.completed_at.is_none());
}

#[tokio::test]
async fn test_todo_delete() {
    let db = setup_test_db().await;
    insert_todo_sql(&db, "delete-me", "Delete Me", 0).await;

    // Verify exists
    let mut response = db
        .query("SELECT VALUE slug FROM todos WHERE slug = 'delete-me'")
        .await
        .unwrap();
    let slugs: Vec<String> = response.take(0).unwrap();
    assert_eq!(slugs.len(), 1);

    // Delete
    db.query("DELETE FROM todos WHERE slug = $slug")
        .bind(("slug", "delete-me".to_string()))
        .await
        .expect("Delete should succeed");

    // Verify gone
    let mut response = db
        .query("SELECT VALUE slug FROM todos WHERE slug = 'delete-me'")
        .await
        .unwrap();
    let slugs: Vec<String> = response.take(0).unwrap();
    assert!(slugs.is_empty());
}

#[tokio::test]
async fn test_todo_ordering() {
    let db = setup_test_db().await;
    insert_todo_sql(&db, "third", "Third", 3).await;
    insert_todo_sql(&db, "first", "First", 1).await;
    insert_todo_sql(&db, "second", "Second", 2).await;

    let mut response = db
        .query("SELECT * FROM todos ORDER BY order ASC")
        .await
        .unwrap();
    let items: Vec<TodoItem> = take_as(&mut response, 0).unwrap();

    assert_eq!(items.len(), 3);
    assert_eq!(items[0].slug, "first");
    assert_eq!(items[1].slug, "second");
    assert_eq!(items[2].slug, "third");
}
