use leptos::prelude::*;

use super::common;

// ── Row parsers ──────────────────────────────────────────────────────────────

#[cfg(all(feature = "brick-todo", feature = "ssr"))]
#[allow(dead_code)]
fn row_todo_list_item(
    row: sqlx::postgres::PgRow,
) -> Result<plinth_shared::TodoListItem, sqlx::Error> {
    use sqlx::Row;

    Ok(plinth_shared::TodoListItem {
        id: common::postgres_id("todos", row.try_get("id")?),
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        tags: row.try_get("tags")?,
        completed: row.try_get("completed")?,
        completed_at: row.try_get("completed_at")?,
        created_at: row.try_get("created_at")?,
        order: row.try_get("order")?,
    })
}

#[cfg(all(feature = "brick-todo", feature = "ssr"))]
#[allow(dead_code)]
fn row_todo_item(row: sqlx::postgres::PgRow) -> Result<plinth_shared::TodoItem, sqlx::Error> {
    use sqlx::Row;

    Ok(plinth_shared::TodoItem {
        id: common::postgres_id("todos", row.try_get("id")?),
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        content: row.try_get("content")?,
        html_content: row.try_get("html_content")?,
        tags: row.try_get("tags")?,
        completed: row.try_get("completed")?,
        completed_at: row.try_get("completed_at")?,
        created_at: row.try_get("created_at")?,
        order: row.try_get("order")?,
    })
}

// ── Query functions ──────────────────────────────────────────────────────────

#[cfg(all(feature = "brick-todo", feature = "ssr"))]
#[allow(dead_code)]
async fn query_todos(db: &sqlx::PgPool) -> Result<Vec<plinth_shared::TodoListItem>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, slug, title, description, tags, completed, completed_at, created_at, "order"
        FROM todos
        ORDER BY completed ASC, "order" ASC, created_at DESC, id DESC
        "#,
    )
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_todo_list_item).collect()
}

#[cfg(all(feature = "brick-todo", feature = "ssr"))]
#[allow(dead_code)]
async fn query_todo_by_slug(
    db: &sqlx::PgPool,
    slug: String,
) -> Result<Option<plinth_shared::TodoItem>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM todos WHERE slug = $1 LIMIT 1")
        .bind(slug)
        .fetch_optional(db)
        .await?;

    row.map(row_todo_item).transpose()
}

#[cfg(all(feature = "brick-todo", feature = "ssr"))]
#[allow(dead_code)]
async fn query_todos_by_tag(
    db: &sqlx::PgPool,
    tag: String,
) -> Result<Vec<plinth_shared::TodoListItem>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT td.id, td.slug, td.title, td.description, td.tags, td.completed,
               td.completed_at, td.created_at, td."order"
        FROM todos td
        JOIN todo_tags tt ON tt.todo_id = td.id
        JOIN tags t ON t.id = tt.tag_id
        WHERE t.name = $1 OR t.slug = $1
        ORDER BY td.completed ASC, td."order" ASC, td.created_at DESC, td.id DESC
        "#,
    )
    .bind(tag)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_todo_list_item).collect()
}

// ── Server functions (SSR) + CSR alternatives ────────────────────────────────

#[cfg(feature = "brick-todo")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetTodos, "/api")]
pub async fn get_todos() -> Result<Vec<plinth_shared::TodoListItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_todos(&db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-todo")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_todos() -> Result<Vec<plinth_shared::TodoListItem>, ServerFnError> {
    common::fetch_json("/api/todos").await
}

#[cfg(feature = "brick-todo")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetTodoBySlug, "/api")]
pub async fn get_todo_by_slug(
    slug: String,
) -> Result<Option<plinth_shared::TodoItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_todo_by_slug(&db, slug)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = slug;
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-todo")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_todo_by_slug(
    slug: String,
) -> Result<Option<plinth_shared::TodoItem>, ServerFnError> {
    common::fetch_json(&format!("/api/todos/{}", common::encode_segment(&slug))).await
}

#[cfg(feature = "brick-todo")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetTodosByTag, "/api")]
pub async fn get_todos_by_tag(
    tag: String,
) -> Result<Vec<plinth_shared::TodoListItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_todos_by_tag(&db, tag)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = tag;
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-todo")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_todos_by_tag(
    tag: String,
) -> Result<Vec<plinth_shared::TodoListItem>, ServerFnError> {
    common::fetch_json(&format!("/api/todos/tag/{}", common::encode_segment(&tag))).await
}
