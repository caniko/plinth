use axum::{
    Json,
    extract::{Path, State},
};
use chrono::Utc;
use plinth_shared::{AddTagRequest, CreateTodoRequest, UpdateTodoRequest};
use tracing::warn;

use super::cache::InvalidateCache as TodoInvalidateCache;
use crate::{
    AppState,
    actors::core_cache::InvalidateCache as CoreInvalidateCache,
    api::admin::ErrorResponse,
    services::{
        db::{create_tags_for_todo_tx, sync_todo_tags_cache_tx},
        markdown_processor::generate_slug,
        rows,
    },
};

/// Create a new TODO item.
pub async fn create_todo(
    State(state): State<AppState>,
    Json(request): Json<CreateTodoRequest>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let slug = request
        .slug
        .clone()
        .unwrap_or_else(|| generate_slug(&request.title));
    let completed_at = request.completed.then(Utc::now);

    let mut tx = state.db.begin().await.map_err(|e| ErrorResponse {
        error: "Failed to start transaction".to_string(),
        details: Some(e.to_string()),
    })?;

    sqlx::query(
        r#"
        INSERT INTO todos (
            slug, title, description, content, html_content, tags,
            completed, completed_at, created_at, "order"
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, now(), $9)
        "#,
    )
    .bind(&slug)
    .bind(&request.title)
    .bind(&request.description)
    .bind(&request.content)
    .bind(&request.html_content)
    .bind(&request.tags)
    .bind(request.completed)
    .bind(completed_at)
    .bind(request.order)
    .execute(&mut *tx)
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to create TODO item".to_string(),
        details: Some(e.to_string()),
    })?;

    create_tags_for_todo_tx(&mut tx, &slug, &request.tags)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to create tag relation".to_string(),
            details: Some(e.to_string()),
        })?;
    sync_todo_tags_cache_tx(&mut tx, &slug)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to sync tag cache".to_string(),
            details: Some(e.to_string()),
        })?;

    tx.commit().await.map_err(|e| ErrorResponse {
        error: "Failed to commit transaction".to_string(),
        details: Some(e.to_string()),
    })?;

    invalidate_caches(&state).await;

    Ok(Json(serde_json::json!({
        "success": true,
        "slug": slug,
        "message": format!("TODO '{}' created successfully", request.title)
    })))
}

/// Update an existing TODO item.
pub async fn update_todo(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(request): Json<UpdateTodoRequest>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let row = sqlx::query("SELECT * FROM todos WHERE slug = $1 LIMIT 1")
        .bind(&slug)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| ErrorResponse {
            error: "Database query failed".to_string(),
            details: Some(e.to_string()),
        })?;

    let existing = row
        .map(rows::todo_item)
        .transpose()
        .map_err(|e| ErrorResponse {
            error: "Failed to parse query result".to_string(),
            details: Some(e.to_string()),
        })?
        .ok_or_else(|| ErrorResponse {
            error: "TODO item not found".to_string(),
            details: Some(format!("No TODO with slug '{slug}'")),
        })?;

    let title = request.title.unwrap_or(existing.title);
    let description = request.description.unwrap_or(existing.description);
    let content = request.content.or(existing.content);
    let html_content = request.html_content.or(existing.html_content);
    let order = request.order.unwrap_or(existing.order);
    let completed = request.completed.unwrap_or(existing.completed);
    let completed_at = if completed && !existing.completed {
        Some(Utc::now())
    } else if !completed {
        None
    } else {
        existing.completed_at
    };

    let mut tx = state.db.begin().await.map_err(|e| ErrorResponse {
        error: "Failed to start transaction".to_string(),
        details: Some(e.to_string()),
    })?;

    let tags_for_column = request.tags.clone().unwrap_or(existing.tags);

    sqlx::query(
        r#"
        UPDATE todos
        SET title = $1,
            description = $2,
            content = $3,
            html_content = $4,
            "order" = $5,
            completed = $6,
            completed_at = $7,
            tags = $8
        WHERE slug = $9
        "#,
    )
    .bind(title)
    .bind(description)
    .bind(content)
    .bind(html_content)
    .bind(order)
    .bind(completed)
    .bind(completed_at)
    .bind(&tags_for_column)
    .bind(&slug)
    .execute(&mut *tx)
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to update TODO item".to_string(),
        details: Some(e.to_string()),
    })?;

    if let Some(new_tags) = request.tags {
        sqlx::query(
            r#"
            DELETE FROM todo_tags tt
            USING todos td
            WHERE tt.todo_id = td.id AND td.slug = $1
            "#,
        )
        .bind(&slug)
        .execute(&mut *tx)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to update tag relations".to_string(),
            details: Some(e.to_string()),
        })?;

        create_tags_for_todo_tx(&mut tx, &slug, &new_tags)
            .await
            .map_err(|e| ErrorResponse {
                error: "Failed to update tag relations".to_string(),
                details: Some(e.to_string()),
            })?;
        sync_todo_tags_cache_tx(&mut tx, &slug)
            .await
            .map_err(|e| ErrorResponse {
                error: "Failed to sync tag cache".to_string(),
                details: Some(e.to_string()),
            })?;
    }

    tx.commit().await.map_err(|e| ErrorResponse {
        error: "Failed to commit transaction".to_string(),
        details: Some(e.to_string()),
    })?;

    invalidate_caches(&state).await;

    Ok(Json(serde_json::json!({
        "success": true,
        "slug": slug,
        "message": format!("TODO '{}' updated successfully", slug)
    })))
}

/// Delete a TODO item by slug.
pub async fn delete_todo(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let mut tx = state.db.begin().await.map_err(|e| ErrorResponse {
        error: "Failed to start transaction".to_string(),
        details: Some(e.to_string()),
    })?;

    let deleted = sqlx::query("DELETE FROM todos WHERE slug = $1")
        .bind(&slug)
        .execute(&mut *tx)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to delete TODO item".to_string(),
            details: Some(e.to_string()),
        })?
        .rows_affected();

    if deleted == 0 {
        return Err(ErrorResponse {
            error: "TODO item not found".to_string(),
            details: Some(format!("No TODO with slug '{slug}'")),
        });
    }

    tx.commit().await.map_err(|e| ErrorResponse {
        error: "Failed to commit transaction".to_string(),
        details: Some(e.to_string()),
    })?;

    invalidate_caches(&state).await;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("TODO '{}' deleted successfully", slug)
    })))
}

/// Add a tag to a TODO item.
pub async fn add_tag_to_todo(
    State(state): State<AppState>,
    Path(todo_slug): Path<String>,
    Json(request): Json<AddTagRequest>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let mut tx = state.db.begin().await.map_err(|e| ErrorResponse {
        error: "Failed to start transaction".to_string(),
        details: Some(e.to_string()),
    })?;

    create_tags_for_todo_tx(&mut tx, &todo_slug, std::slice::from_ref(&request.tag))
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to add tag to TODO".to_string(),
            details: Some(e.to_string()),
        })?;
    sync_todo_tags_cache_tx(&mut tx, &todo_slug)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to sync tag cache".to_string(),
            details: Some(e.to_string()),
        })?;

    tx.commit().await.map_err(|e| ErrorResponse {
        error: "Failed to commit transaction".to_string(),
        details: Some(e.to_string()),
    })?;

    invalidate_caches(&state).await;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Tag '{}' added to TODO '{}'", request.tag, todo_slug)
    })))
}

/// Remove a tag from a TODO item.
pub async fn remove_tag_from_todo(
    State(state): State<AppState>,
    Path((todo_slug, tag_slug)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let mut tx = state.db.begin().await.map_err(|e| ErrorResponse {
        error: "Failed to start transaction".to_string(),
        details: Some(e.to_string()),
    })?;

    sqlx::query(
        r#"
        DELETE FROM todo_tags tt
        USING todos td, tags t
        WHERE tt.todo_id = td.id
          AND tt.tag_id = t.id
          AND td.slug = $1
          AND t.slug = $2
        "#,
    )
    .bind(&todo_slug)
    .bind(&tag_slug)
    .execute(&mut *tx)
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to remove tag from TODO".to_string(),
        details: Some(e.to_string()),
    })?;

    sync_todo_tags_cache_tx(&mut tx, &todo_slug)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to sync tag cache".to_string(),
            details: Some(e.to_string()),
        })?;

    tx.commit().await.map_err(|e| ErrorResponse {
        error: "Failed to commit transaction".to_string(),
        details: Some(e.to_string()),
    })?;

    invalidate_caches(&state).await;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Tag '{}' removed from TODO '{}'", tag_slug, todo_slug)
    })))
}

async fn invalidate_caches(state: &AppState) {
    if let Err(e) = state.core_cache.ask(CoreInvalidateCache).await {
        warn!("Cache invalidation failed: {e}");
    }
    if let Err(e) = state.todo_cache.ask(TodoInvalidateCache).await {
        warn!("Cache invalidation failed: {e}");
    }
}
