use kameo::Actor;
use kameo::message::{Context, Message};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use surrealdb::Surreal;
use surrealdb::engine::local::Db;

use plinth_shared::{SiteContent, Tag};

use crate::db_helpers::{take_as, take_as_opt};

/// Cache entry TTL — entries older than this are treated as expired.
const CACHE_TTL: Duration = Duration::from_secs(5 * 60);

/// Maximum number of individually-cached items per category.
const MAX_ITEM_CACHE_SIZE: usize = 500;

/// Core cache actor containing only the shared/cross-brick data.
///
/// Brick-specific caches (blog posts, portfolio items, todos, etc.) live in
/// their own dedicated actors. This actor handles tags and site content which
/// are referenced by multiple bricks.
#[derive(Actor)]
pub struct CoreCache {
    db: Surreal<Db>,
    site_content: HashMap<String, SiteContent>,
    /// Timestamp of the last cache population / invalidation.
    cache_populated_at: Option<Instant>,
}

impl CoreCache {
    /// Create a new CoreCache actor with a SurrealDB connection.
    pub fn new(db: Surreal<Db>) -> Self {
        Self {
            db,
            site_content: HashMap::new(),
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
        self.site_content.clear();
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
// Tags
// ---------------------------------------------------------------------------

/// Get all tags with counts that are conditional on enabled brick features.
pub struct GetAllTags;

impl Message<GetAllTags> for CoreCache {
    type Reply = Result<Vec<Tag>, String>;

    #[allow(clippy::vec_init_then_push)]
    async fn handle(
        &mut self,
        _msg: GetAllTags,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Build query dynamically based on enabled brick features
        let mut count_parts: Vec<&str> = Vec::new();
        #[cfg(feature = "brick-blog")]
        count_parts.push("count(<-tagged<-blog_posts) AS post_count");
        #[cfg(feature = "brick-todo")]
        count_parts.push("count(<-todo_tagged<-todos) AS todo_count");

        let query = if count_parts.is_empty() {
            "SELECT * FROM tags ORDER BY name ASC".to_string()
        } else {
            format!(
                "SELECT *, {} FROM tags ORDER BY name ASC",
                count_parts.join(", ")
            )
        };

        let mut response = self
            .db
            .query(&query)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        take_as(&mut response, 0).map_err(|e| format!("Database error: {}", e))
    }
}

// ---------------------------------------------------------------------------
// Site content
// ---------------------------------------------------------------------------

/// Get site content by key (e.g. "home-intro", "about").
pub struct GetSiteContent(pub String);

impl Message<GetSiteContent> for CoreCache {
    type Reply = Result<Option<SiteContent>, String>;

    async fn handle(
        &mut self,
        msg: GetSiteContent,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.expire_if_stale();
        let key = msg.0;

        // Check cache first
        if let Some(content) = self.site_content.get(&key) {
            return Ok(Some(content.clone()));
        }

        // Query SurrealDB
        let mut response = self
            .db
            .query("SELECT * FROM site_content WHERE key = $key LIMIT 1")
            .bind(("key", key.clone()))
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let content: Option<SiteContent> =
            take_as_opt(&mut response, 0).map_err(|e| format!("Database error: {}", e))?;

        match content {
            Some(content) => {
                if self.site_content.len() < MAX_ITEM_CACHE_SIZE {
                    self.site_content.insert(key, content.clone());
                    self.touch();
                }
                Ok(Some(content))
            }
            None => Ok(None),
        }
    }
}

// ---------------------------------------------------------------------------
// Cache invalidation
// ---------------------------------------------------------------------------

/// Invalidate all caches (call after publishing new content).
pub struct InvalidateCache;

impl Message<InvalidateCache> for CoreCache {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: InvalidateCache,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.clear_all();
    }
}
