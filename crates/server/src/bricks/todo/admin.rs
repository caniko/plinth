use axum::{
    Json,
    extract::{Path, State},
};
use plinth_shared::{AddTagRequest, CreateTodoRequest, TodoItem, UpdateTodoRequest};
use tracing::warn;

use super::cache::InvalidateCache as TodoInvalidateCache;
use crate::{
    AppState,
    actors::core_cache::InvalidateCache as CoreInvalidateCache,
    api::admin::ErrorResponse,
    db_helpers::take_as,
    services::{db::sync_todo_tags_cache, markdown_processor::generate_slug},
};

/// Create a new TODO item
pub async fn create_todo(
    State(state): State<AppState>,
    Json(request): Json<CreateTodoRequest>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let db = &state.db;
    let slug = request
        .slug
        .unwrap_or_else(|| generate_slug(&request.title));

    // Use raw SQL for datetime fields (SurrealDB SCHEMAFULL constraint)
    let completed_at_expr = if request.completed {
        "time::now()"
    } else {
        "NONE"
    };

    db.query(format!(
        r#"CREATE todos CONTENT {{
            slug: $slug,
            title: $title,
            description: $description,
            content: $content,
            html_content: $html_content,
            tags: $tags,
            completed: $completed,
            completed_at: {completed_at_expr},
            created_at: time::now(),
            order: $order
        }}"#
    ))
    .bind(("slug", slug.clone()))
    .bind(("title", request.title.clone()))
    .bind(("description", request.description))
    .bind(("content", request.content))
    .bind(("html_content", request.html_content))
    .bind(("tags", request.tags.clone()))
    .bind(("completed", request.completed))
    .bind(("order", request.order))
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to create TODO item".to_string(),
        details: Some(e.to_string()),
    })?;

    // Create tag graph relations
    for tag_name in &request.tags {
        let tag_slug = generate_slug(tag_name);
        db.query(
            r#"
            IF (SELECT count() FROM tags WHERE slug = $tag_slug) = 0 THEN
                CREATE tags CONTENT {
                    name: $tag_name,
                    slug: $tag_slug,
                    created_at: time::now()
                }
            END;

            LET $todo = (SELECT VALUE id FROM todos WHERE slug = $todo_slug LIMIT 1)[0];
            LET $tag = (SELECT VALUE id FROM tags WHERE slug = $tag_slug LIMIT 1)[0];
            RELATE $todo->todo_tagged->$tag CONTENT { created_at: time::now() };
        "#,
        )
        .bind(("tag_name", tag_name.to_string()))
        .bind(("tag_slug", tag_slug))
        .bind(("todo_slug", slug.clone()))
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to create tag relation".to_string(),
            details: Some(e.to_string()),
        })?;
    }

    // Invalidate caches
    if let Err(e) = state.core_cache.ask(CoreInvalidateCache).await {
        warn!("Cache invalidation failed: {e}");
    }
    if let Err(e) = state.todo_cache.ask(TodoInvalidateCache).await {
        warn!("Cache invalidation failed: {e}");
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "slug": slug,
        "message": format!("TODO '{}' created successfully", request.title)
    })))
}

/// Update an existing TODO item (partial update)
pub async fn update_todo(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(request): Json<UpdateTodoRequest>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let db = &state.db;

    // Fetch existing item
    let mut result = db
        .query("SELECT * FROM todos WHERE slug = $slug LIMIT 1")
        .bind(("slug", slug.clone()))
        .await
        .map_err(|e| ErrorResponse {
            error: "Database query failed".to_string(),
            details: Some(e.to_string()),
        })?;

    let existing: Vec<TodoItem> = take_as(&mut result, 0).map_err(|e| ErrorResponse {
        error: "Failed to parse query result".to_string(),
        details: Some(e),
    })?;

    if existing.is_empty() {
        return Err(ErrorResponse {
            error: "TODO item not found".to_string(),
            details: Some(format!("No TODO with slug '{}'", slug)),
        });
    }

    let existing = &existing[0];

    // Build the updated values, falling back to existing values for unset fields.
    // This uses parameter binding for all values to prevent SQL injection.
    let title = request.title.unwrap_or_else(|| existing.title.clone());
    let description = request
        .description
        .unwrap_or_else(|| existing.description.clone());
    let content = request.content.or_else(|| existing.content.clone());
    let html_content = request
        .html_content
        .or_else(|| existing.html_content.clone());
    let order = request.order.unwrap_or(existing.order);
    let completed = request.completed.unwrap_or(existing.completed);

    // Determine completed_at handling:
    // - transitioning to completed → set to time::now()
    // - transitioning to not completed → clear to NONE
    // - no change → keep existing
    let set_completed_at = if completed && !existing.completed {
        "completed_at = time::now()"
    } else if !completed && existing.completed {
        "completed_at = NONE"
    } else {
        // Keep existing value — no-op SET to itself
        "completed_at = completed_at"
    };

    db.query(format!(
        r#"UPDATE todos SET
            title = $title,
            description = $description,
            content = $content,
            html_content = $html_content,
            order = $order,
            completed = $completed,
            {set_completed_at}
        WHERE slug = $slug"#
    ))
    .bind(("slug", slug.clone()))
    .bind(("title", title))
    .bind(("description", description))
    .bind(("content", content))
    .bind(("html_content", html_content))
    .bind(("order", order as i64))
    .bind(("completed", completed))
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to update TODO item".to_string(),
        details: Some(e.to_string()),
    })?;

    // Handle tag updates
    if let Some(ref new_tags) = request.tags {
        // Clear existing relations and create new ones in a single query batch
        let mut tag_sql = String::from(
            "BEGIN TRANSACTION;\nLET $todo = (SELECT VALUE id FROM todos WHERE slug = $slug LIMIT 1)[0];\nDELETE todo_tagged WHERE in = $todo;\n",
        );
        let mut binds: Vec<(String, String)> = vec![("slug".into(), slug.clone())];

        for (i, tag_name) in new_tags.iter().enumerate() {
            let tag_slug = generate_slug(tag_name);
            let name_key = format!("tag_name_{i}");
            let slug_key = format!("tag_slug_{i}");
            tag_sql.push_str(&format!(
                r#"
                IF (SELECT count() FROM tags WHERE slug = ${slug_key}) = 0 THEN
                    CREATE tags CONTENT {{
                        name: ${name_key},
                        slug: ${slug_key},
                        created_at: time::now()
                    }}
                END;
                LET $tag_{i} = (SELECT VALUE id FROM tags WHERE slug = ${slug_key} LIMIT 1)[0];
                RELATE $todo->todo_tagged->$tag_{i} CONTENT {{ created_at: time::now() }};
                "#
            ));
            binds.push((name_key, tag_name.to_string()));
            binds.push((slug_key, tag_slug));
        }
        tag_sql.push_str("COMMIT TRANSACTION;");

        let mut q = db.query(&tag_sql);
        for (key, value) in binds {
            q = q.bind((key, value));
        }
        q.await.map_err(|e| ErrorResponse {
            error: "Failed to update tag relations".to_string(),
            details: Some(e.to_string()),
        })?;

        // Sync denormalized tags array
        sync_todo_tags_cache(db, &slug)
            .await
            .map_err(|e| ErrorResponse {
                error: "Failed to sync tag cache".to_string(),
                details: Some(e.to_string()),
            })?;
    }

    if let Err(e) = state.core_cache.ask(CoreInvalidateCache).await {
        warn!("Cache invalidation failed: {e}");
    }
    if let Err(e) = state.todo_cache.ask(TodoInvalidateCache).await {
        warn!("Cache invalidation failed: {e}");
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "slug": slug,
        "message": format!("TODO '{}' updated successfully", slug)
    })))
}

/// Delete a TODO item by slug
pub async fn delete_todo(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let db = &state.db;

    // Find the item
    let mut result = db
        .query("SELECT * FROM todos WHERE slug = $slug LIMIT 1")
        .bind(("slug", slug.clone()))
        .await
        .map_err(|e| ErrorResponse {
            error: "Database query failed".to_string(),
            details: Some(e.to_string()),
        })?;

    let items: Vec<TodoItem> = take_as(&mut result, 0).map_err(|e| ErrorResponse {
        error: "Failed to parse query result".to_string(),
        details: Some(e),
    })?;

    if items.is_empty() {
        return Err(ErrorResponse {
            error: "TODO item not found".to_string(),
            details: Some(format!("No TODO with slug '{}'", slug)),
        });
    }

    // Delete tag relations and the item itself in a transaction
    db.query(
        r#"
        BEGIN TRANSACTION;
        LET $todo = (SELECT VALUE id FROM todos WHERE slug = $slug LIMIT 1)[0];
        DELETE todo_tagged WHERE in = $todo;
        DELETE FROM todos WHERE slug = $slug;
        COMMIT TRANSACTION;
    "#,
    )
    .bind(("slug", slug.clone()))
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to delete TODO item".to_string(),
        details: Some(e.to_string()),
    })?;

    if let Err(e) = state.core_cache.ask(CoreInvalidateCache).await {
        warn!("Cache invalidation failed: {e}");
    }
    if let Err(e) = state.todo_cache.ask(TodoInvalidateCache).await {
        warn!("Cache invalidation failed: {e}");
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("TODO '{}' deleted successfully", slug)
    })))
}

/// Add a tag to a TODO item
pub async fn add_tag_to_todo(
    State(state): State<AppState>,
    Path(todo_slug): Path<String>,
    Json(request): Json<AddTagRequest>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let db = &state.db;
    let tag_slug = generate_slug(&request.tag);

    // Ensure tag exists
    db.query(
        r#"
        IF (SELECT count() FROM tags WHERE slug = $tag_slug) = 0 THEN
            CREATE tags CONTENT {
                name: $tag_name,
                slug: $tag_slug,
                created_at: time::now()
            }
        END;
    "#,
    )
    .bind(("tag_name", request.tag.clone()))
    .bind(("tag_slug", tag_slug.clone()))
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to ensure tag exists".to_string(),
        details: Some(e.to_string()),
    })?;

    // Create relation
    db.query(
        r#"
        LET $todo = (SELECT VALUE id FROM todos WHERE slug = $todo_slug LIMIT 1)[0];
        LET $tag = (SELECT VALUE id FROM tags WHERE slug = $tag_slug LIMIT 1)[0];
        RELATE $todo->todo_tagged->$tag CONTENT { created_at: time::now() };
    "#,
    )
    .bind(("todo_slug", todo_slug.clone()))
    .bind(("tag_slug", tag_slug))
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to add tag to TODO".to_string(),
        details: Some(e.to_string()),
    })?;

    // Sync denormalized cache
    sync_todo_tags_cache(db, &todo_slug)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to sync tag cache".to_string(),
            details: Some(e.to_string()),
        })?;

    if let Err(e) = state.core_cache.ask(CoreInvalidateCache).await {
        warn!("Cache invalidation failed: {e}");
    }
    if let Err(e) = state.todo_cache.ask(TodoInvalidateCache).await {
        warn!("Cache invalidation failed: {e}");
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Tag '{}' added to TODO '{}'", request.tag, todo_slug)
    })))
}

/// Remove a tag from a TODO item
pub async fn remove_tag_from_todo(
    State(state): State<AppState>,
    Path((todo_slug, tag_slug)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let db = &state.db;

    // Delete the relation
    db.query(
        r#"
        LET $todo = (SELECT VALUE id FROM todos WHERE slug = $todo_slug LIMIT 1)[0];
        LET $tag = (SELECT VALUE id FROM tags WHERE slug = $tag_slug LIMIT 1)[0];
        DELETE todo_tagged WHERE in = $todo AND out = $tag;
    "#,
    )
    .bind(("todo_slug", todo_slug.clone()))
    .bind(("tag_slug", tag_slug.clone()))
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to remove tag from TODO".to_string(),
        details: Some(e.to_string()),
    })?;

    // Sync denormalized cache
    sync_todo_tags_cache(db, &todo_slug)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to sync tag cache".to_string(),
            details: Some(e.to_string()),
        })?;

    if let Err(e) = state.core_cache.ask(CoreInvalidateCache).await {
        warn!("Cache invalidation failed: {e}");
    }
    if let Err(e) = state.todo_cache.ask(TodoInvalidateCache).await {
        warn!("Cache invalidation failed: {e}");
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Tag '{}' removed from TODO '{}'", tag_slug, todo_slug)
    })))
}
