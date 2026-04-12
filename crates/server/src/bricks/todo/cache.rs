//! Todo cache actor — extracted from the monolithic ContentCache.

use kameo::Actor;
use kameo::message::{Context, Message};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use surrealdb::Surreal;
use surrealdb::engine::local::Db;

use plinth_shared::{TodoItem, TodoListItem};

use crate::db_helpers::{take_as, take_as_opt};

/// Cache entry TTL — entries older than this are treated as expired.
const CACHE_TTL: Duration = Duration::from_secs(5 * 60);

/// Maximum number of individually-cached items per category.
const MAX_ITEM_CACHE_SIZE: usize = 500;

/// Todo cache actor that stores frequently accessed todo items in memory
/// and queries SurrealDB on cache misses.
#[derive(Actor)]
pub struct TodoCache {
    db: Surreal<Db>,
    todo_items: HashMap<String, TodoItem>,
    todo_list_cache: Option<Vec<TodoListItem>>,
    /// Timestamp of the last cache population / invalidation
    cache_populated_at: Option<Instant>,
}

impl TodoCache {
    /// Create a new TodoCache actor with a SurrealDB connection.
    pub fn new(db: Surreal<Db>) -> Self {
        Self {
            db,
            todo_items: HashMap::new(),
            todo_list_cache: None,
            cache_populated_at: None,
        }
    }

    /// Returns true if the cache has expired and should be cleared.
    fn is_expired(&self) -> bool {
        self.cache_populated_at
            .is_some_and(|t| t.elapsed() > CACHE_TTL)
    }

    /// Clear all caches and reset the population timestamp.
    fn clear_all(&mut self) {
        self.todo_items.clear();
        self.todo_list_cache = None;
        self.cache_populated_at = None;
    }

    /// Mark the cache as freshly populated.
    fn touch(&mut self) {
        if self.cache_populated_at.is_none() {
            self.cache_populated_at = Some(Instant::now());
        }
    }

    /// Expire stale entries if TTL has passed.
    fn expire_if_stale(&mut self) {
        if self.is_expired() {
            self.clear_all();
        }
    }
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// Get a single TODO item by slug.
pub struct GetTodoItem(pub String);

impl Message<GetTodoItem> for TodoCache {
    type Reply = Result<Option<TodoItem>, String>;

    async fn handle(
        &mut self,
        msg: GetTodoItem,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.expire_if_stale();
        let slug = msg.0;

        // Check cache first
        if let Some(item) = self.todo_items.get(&slug) {
            return Ok(Some(item.clone()));
        }

        // Query SurrealDB
        let mut response = self
            .db
            .query("SELECT * FROM todos WHERE slug = $slug")
            .bind(("slug", slug.clone()))
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let item: Option<TodoItem> =
            take_as_opt(&mut response, 0).map_err(|e| format!("Database error: {}", e))?;

        match item {
            Some(item) => {
                if self.todo_items.len() < MAX_ITEM_CACHE_SIZE {
                    self.todo_items.insert(slug, item.clone());
                    self.touch();
                }
                Ok(Some(item))
            }
            None => Ok(None),
        }
    }
}

/// Get all TODO items (as list items, pending first).
pub struct GetAllTodos;

impl Message<GetAllTodos> for TodoCache {
    type Reply = Result<Vec<TodoListItem>, String>;

    async fn handle(
        &mut self,
        _msg: GetAllTodos,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.expire_if_stale();

        // Check cache first
        if let Some(ref list) = self.todo_list_cache {
            return Ok(list.clone());
        }

        // Query SurrealDB — pending items first, then by order, then newest first
        let mut response = self
            .db
            .query("SELECT * FROM todos ORDER BY completed ASC, order ASC, created_at DESC")
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let items: Vec<TodoListItem> =
            take_as(&mut response, 0).map_err(|e| format!("Database error: {}", e))?;

        self.todo_list_cache = Some(items.clone());
        self.touch();
        Ok(items)
    }
}

/// Get TODO items filtered by tag.
pub struct GetTodosByTag(pub String);

impl Message<GetTodosByTag> for TodoCache {
    type Reply = Result<Vec<TodoListItem>, String>;

    async fn handle(
        &mut self,
        msg: GetTodosByTag,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let mut response = self
            .db
            .query(
                r#"SELECT * FROM todos
                WHERE $tag IN tags
                ORDER BY completed ASC, order ASC, created_at DESC"#,
            )
            .bind(("tag", msg.0))
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        take_as(&mut response, 0).map_err(|e| format!("Database error: {}", e))
    }
}

/// Invalidate all todo caches (call after publishing new content).
pub struct InvalidateCache;

impl Message<InvalidateCache> for TodoCache {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: InvalidateCache,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.clear_all();
    }
}
