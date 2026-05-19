use crate::PlinthDb;
use kameo::Actor;
use kameo::message::{Context, Message};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use plinth_shared::{SiteContent, Tag};

use crate::services::rows;

/// Cache entry TTL — entries older than this are treated as expired.
const CACHE_TTL: Duration = Duration::from_secs(5 * 60);

/// Maximum number of individually-cached site-content entries.
const MAX_ITEM_CACHE_SIZE: usize = 500;

/// Core cache actor containing only the shared/cross-brick data.
///
/// Brick-specific caches (blog posts, portfolio items, todos, etc.) live in
/// their own dedicated actors. This actor handles tags and site content which
/// are referenced by multiple bricks.
#[derive(Actor)]
pub struct CoreCache {
    db: PlinthDb,
    site_content: HashMap<String, SiteContent>,
    /// Timestamp of the last cache population / invalidation.
    cache_populated_at: Option<Instant>,
}

impl CoreCache {
    /// Create a new CoreCache actor with a database connection.
    pub fn new(db: PlinthDb) -> Self {
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
        let rows = sqlx::query(
            r#"
            SELECT
                tags.id,
                tags.name,
                tags.slug,
                COALESCE(blog_counts.post_count, 0)::integer AS post_count,
                COALESCE(todo_counts.todo_count, 0)::integer AS todo_count
            FROM tags
            LEFT JOIN (
                SELECT tag_id, COUNT(*)::integer AS post_count
                FROM blog_post_tags
                GROUP BY tag_id
            ) blog_counts ON blog_counts.tag_id = tags.id
            LEFT JOIN (
                SELECT tag_id, COUNT(*)::integer AS todo_count
                FROM todo_tags
                GROUP BY tag_id
            ) todo_counts ON todo_counts.tag_id = tags.id
            ORDER BY tags.name ASC, tags.id ASC
            "#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

        rows.into_iter()
            .map(rows::tag)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Database error: {e}"))
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

        let row = sqlx::query("SELECT * FROM site_content WHERE key = $1 LIMIT 1")
            .bind(&key)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| format!("Database error: {e}"))?;

        let content = row
            .map(rows::site_content)
            .transpose()
            .map_err(|e| format!("Database error: {e}"))?;

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
