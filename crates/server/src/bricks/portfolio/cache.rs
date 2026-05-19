//! Portfolio cache actor — extracted from the monolithic ContentCache.

use crate::PlinthDb;
use kameo::Actor;
use kameo::message::{Context, Message};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use plinth_shared::PortfolioItem;

use crate::services::rows;

/// Cache entry TTL — entries older than this are treated as expired.
const CACHE_TTL: Duration = Duration::from_secs(5 * 60);

/// Maximum number of individually-cached portfolio items.
const MAX_ITEM_CACHE_SIZE: usize = 500;

/// Portfolio cache actor that stores frequently accessed portfolio items in memory
/// and queries the database on cache misses.
#[derive(Actor)]
pub struct PortfolioCache {
    db: PlinthDb,
    portfolio_items: HashMap<String, PortfolioItem>,
    portfolio_list_cache: Option<Vec<PortfolioItem>>,
    /// Timestamp of the last cache population / invalidation
    cache_populated_at: Option<Instant>,
}

impl PortfolioCache {
    /// Create a new PortfolioCache actor with a database connection.
    pub fn new(db: PlinthDb) -> Self {
        Self {
            db,
            portfolio_items: HashMap::new(),
            portfolio_list_cache: None,
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
        self.portfolio_items.clear();
        self.portfolio_list_cache = None;
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

/// Get a single portfolio item by slug.
pub struct GetPortfolioItem(pub String);

impl Message<GetPortfolioItem> for PortfolioCache {
    type Reply = Result<Option<PortfolioItem>, String>;

    async fn handle(
        &mut self,
        msg: GetPortfolioItem,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.expire_if_stale();
        let slug = msg.0;

        // Check cache first
        if let Some(item) = self.portfolio_items.get(&slug) {
            return Ok(Some(item.clone()));
        }

        let row = sqlx::query("SELECT * FROM portfolio_items WHERE slug = $1 LIMIT 1")
            .bind(&slug)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| format!("Database error: {e}"))?;

        let item = row
            .map(rows::portfolio_item)
            .transpose()
            .map_err(|e| format!("Database error: {e}"))?;

        match item {
            Some(item) => {
                if self.portfolio_items.len() < MAX_ITEM_CACHE_SIZE {
                    self.portfolio_items.insert(slug, item.clone());
                    self.touch();
                }
                Ok(Some(item))
            }
            None => Ok(None),
        }
    }
}

/// Get all portfolio items, ordered by display order then date.
pub struct GetAllPortfolioItems;

impl Message<GetAllPortfolioItems> for PortfolioCache {
    type Reply = Result<Vec<PortfolioItem>, String>;

    async fn handle(
        &mut self,
        _msg: GetAllPortfolioItems,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.expire_if_stale();

        // Check cache first
        if let Some(ref list) = self.portfolio_list_cache {
            return Ok(list.clone());
        }

        let rows = sqlx::query(
            r#"
            SELECT *
            FROM portfolio_items
            ORDER BY "order" ASC, date DESC, id DESC
            "#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

        let items = rows
            .into_iter()
            .map(rows::portfolio_item)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Database error: {e}"))?;

        self.portfolio_list_cache = Some(items.clone());
        self.touch();
        Ok(items)
    }
}

/// Invalidate all portfolio caches (call after publishing new content).
pub struct InvalidateCache;

impl Message<InvalidateCache> for PortfolioCache {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: InvalidateCache,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.clear_all();
    }
}
