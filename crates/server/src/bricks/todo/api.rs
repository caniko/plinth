//! Todo public API handlers.

use axum::{
    Json,
    extract::{Path, State},
};
use plinth_shared::{TodoItem, TodoListItem};

use super::cache::{GetAllTodos, GetTodoItem, GetTodosByTag};
use crate::{AppState, error::PlinthError};

/// GET /api/todos
pub async fn list_todos(
    State(state): State<AppState>,
) -> Result<Json<Vec<TodoListItem>>, PlinthError> {
    let todos = state
        .todo_cache
        .ask(GetAllTodos)
        .await
        .map_err(PlinthError::actor)?;

    Ok(Json(todos))
}

/// GET /api/todos/{slug}
pub async fn get_todo(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Option<TodoItem>>, PlinthError> {
    let item = state
        .todo_cache
        .ask(GetTodoItem(slug))
        .await
        .map_err(PlinthError::actor)?;

    Ok(Json(item))
}

/// GET /api/todos/tag/{tag}
pub async fn list_todos_by_tag(
    State(state): State<AppState>,
    Path(tag): Path<String>,
) -> Result<Json<Vec<TodoListItem>>, PlinthError> {
    let todos = state
        .todo_cache
        .ask(GetTodosByTag(tag))
        .await
        .map_err(PlinthError::actor)?;

    Ok(Json(todos))
}
