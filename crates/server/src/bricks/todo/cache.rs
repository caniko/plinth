//! Todo cache actor — extracted from the monolithic ContentCache.

use crate::PlinthDb;
use kameo::Actor;
use kameo::message::{Context, Message};
use std::collections::HashMap;

use plinth_shared::{TodoItem, TodoListItem};

use crate::actors::cache_ttl::{MAX_ITEM_CACHE_SIZE, TtlClock};
use crate::services::rows;

/// Todo cache actor that stores frequently accessed todo items in memory
/// and queries the database on cache misses.
#[derive(Actor)]
pub struct TodoCache {
    db: PlinthDb,
    todo_items: HashMap<String, TodoItem>,
    todo_list_cache: Option<Vec<TodoListItem>>,
    /// Timestamp of the last cache population / invalidation
    ttl_clock: TtlClock,
}

impl TodoCache {
    /// Create a new TodoCache actor with a database connection.
    pub fn new(db: PlinthDb) -> Self {
        Self {
            db,
            todo_items: HashMap::new(),
            todo_list_cache: None,
            ttl_clock: TtlClock::default(),
        }
    }

    /// Returns true if the cache has expired and should be cleared.
    fn is_expired(&self) -> bool {
        self.ttl_clock.is_expired()
    }

    /// Clear all caches and reset the population timestamp.
    fn clear_all(&mut self) {
        self.todo_items.clear();
        self.todo_list_cache = None;
        self.ttl_clock.reset();
    }

    /// Mark the cache as freshly populated.
    fn touch(&mut self) {
        self.ttl_clock.touch();
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

        let row = sqlx::query("SELECT * FROM todos WHERE slug = $1 LIMIT 1")
            .bind(&slug)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| format!("Database error: {e}"))?;

        let item = row
            .map(rows::todo_item)
            .transpose()
            .map_err(|e| format!("Database error: {e}"))?;

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

        let rows = sqlx::query(
            r#"
            SELECT id, slug, title, description, tags, completed, completed_at, created_at, "order"
            FROM todos
            ORDER BY completed ASC, "order" ASC, created_at DESC, id DESC
            "#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

        let items = rows
            .into_iter()
            .map(rows::todo_list_item)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Database error: {e}"))?;

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
        let rows = sqlx::query(
            r#"
            SELECT id, slug, title, description, tags, completed, completed_at, created_at, "order"
            FROM todos
            WHERE $1 = ANY(tags)
            ORDER BY completed ASC, "order" ASC, created_at DESC, id DESC
            "#,
        )
        .bind(msg.0)
        .fetch_all(&self.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

        rows.into_iter()
            .map(rows::todo_list_item)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Database error: {e}"))
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
