//! Blog cache actor — extracted from the monolithic ContentCache.
//!
//! Caches blog posts, list views, and series metadata in memory with
//! a TTL-based expiration strategy.

use crate::PlinthDb;
use kameo::Actor;
use kameo::message::{Context, Message};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use plinth_shared::{BlogListItem, BlogPost, SeriesListItem, SeriesNav};

use crate::services::rows;

/// Cache entry TTL — entries older than this are treated as expired.
const CACHE_TTL: Duration = Duration::from_secs(5 * 60);

/// Maximum number of individually-cached posts.
const MAX_ITEM_CACHE_SIZE: usize = 500;

/// Blog-specific cache actor that stores frequently accessed blog content
/// in memory and queries the database on cache misses.
#[derive(Actor)]
pub struct BlogCache {
    db: PlinthDb,
    blog_posts: HashMap<String, BlogPost>,
    blog_list_cache: Option<Vec<BlogListItem>>,
    /// Timestamp of the last cache population / invalidation.
    cache_populated_at: Option<Instant>,
}

impl BlogCache {
    /// Create a new BlogCache actor with a database connection.
    pub fn new(db: PlinthDb) -> Self {
        Self {
            db,
            blog_posts: HashMap::new(),
            blog_list_cache: None,
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
        self.blog_posts.clear();
        self.blog_list_cache = None;
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
// Messages for blog posts
// ---------------------------------------------------------------------------

/// Get a single blog post by slug.
pub struct GetBlogPost(pub String);

impl Message<GetBlogPost> for BlogCache {
    type Reply = Result<Option<BlogPost>, String>;

    async fn handle(
        &mut self,
        msg: GetBlogPost,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.expire_if_stale();
        let slug = msg.0;

        // Check cache first
        if let Some(post) = self.blog_posts.get(&slug) {
            return Ok(Some(post.clone()));
        }

        let row = sqlx::query(
            r#"
            SELECT * FROM blog_posts
            WHERE slug = $1 AND published = true
            LIMIT 1
            "#,
        )
        .bind(&slug)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

        let post = row
            .map(rows::blog_post)
            .transpose()
            .map_err(|e| format!("Database error: {e}"))?;

        match post {
            Some(post) => {
                if self.blog_posts.len() < MAX_ITEM_CACHE_SIZE {
                    self.blog_posts.insert(slug, post.clone());
                    self.touch();
                }
                Ok(Some(post))
            }
            None => Ok(None),
        }
    }
}

/// Get all published blog posts (as list items).
pub struct GetAllBlogPosts;

impl Message<GetAllBlogPosts> for BlogCache {
    type Reply = Result<Vec<BlogListItem>, String>;

    async fn handle(
        &mut self,
        _msg: GetAllBlogPosts,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.expire_if_stale();

        // Check cache first
        if let Some(ref list) = self.blog_list_cache {
            return Ok(list.clone());
        }

        let rows = sqlx::query(
            r#"
            SELECT id, slug, title, description, published_at, author, tags, featured,
                   reading_time_minutes, series_slug, series_title, series_position
            FROM blog_posts
            WHERE published = true
            ORDER BY published_at DESC, id DESC
            "#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

        let list = rows
            .into_iter()
            .map(rows::blog_list_item)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Database error: {e}"))?;

        self.blog_list_cache = Some(list.clone());
        self.touch();
        Ok(list)
    }
}

/// Get blog posts by tag.
pub struct GetPostsByTag(pub String);

impl Message<GetPostsByTag> for BlogCache {
    type Reply = Result<Vec<BlogListItem>, String>;

    async fn handle(
        &mut self,
        msg: GetPostsByTag,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let rows = sqlx::query(
            r#"
            SELECT id, slug, title, description, published_at, author, tags, featured,
                   reading_time_minutes, series_slug, series_title, series_position
            FROM blog_posts
            WHERE published = true AND $1 = ANY(tags)
            ORDER BY published_at DESC, id DESC
            "#,
        )
        .bind(msg.0)
        .fetch_all(&self.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

        rows.into_iter()
            .map(rows::blog_list_item)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Database error: {e}"))
    }
}

// ---------------------------------------------------------------------------
// Messages for series
// ---------------------------------------------------------------------------

/// Get series navigation context for a post (prev/next/TOC within its series).
pub struct GetSeriesNav(pub String); // post slug

impl Message<GetSeriesNav> for BlogCache {
    type Reply = Result<Option<SeriesNav>, String>;

    async fn handle(
        &mut self,
        msg: GetSeriesNav,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let post_slug = msg.0;

        let info = sqlx::query(
            r#"
            SELECT series_slug, series_title, series_position
            FROM blog_posts
            WHERE slug = $1 AND published = true
            LIMIT 1
            "#,
        )
        .bind(post_slug)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

        let Some(info) = info else {
            return Ok(None);
        };

        use sqlx::Row;
        let Some(series_slug) = info
            .try_get::<Option<String>, _>("series_slug")
            .map_err(|e| format!("Database error: {e}"))?
        else {
            return Ok(None);
        };
        let series_title = info
            .try_get::<Option<String>, _>("series_title")
            .map_err(|e| format!("Database error: {e}"))?
            .unwrap_or_default();
        let current_position = info
            .try_get::<Option<i32>, _>("series_position")
            .map_err(|e| format!("Database error: {e}"))?
            .unwrap_or(0)
            .max(0) as u32;

        let rows = sqlx::query(
            r#"
            SELECT slug, title, series_position AS position
            FROM blog_posts
            WHERE series_slug = $1 AND published = true
            ORDER BY series_position ASC NULLS LAST, published_at ASC, id ASC
            "#,
        )
        .bind(&series_slug)
        .fetch_all(&self.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

        let entries = rows
            .into_iter()
            .map(rows::series_entry)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Database error: {e}"))?;

        let total_published = entries.len() as u32;
        let mut prev = None;
        let mut next = None;
        for (i, entry) in entries.iter().enumerate() {
            if entry.position == current_position {
                if i > 0 {
                    prev = Some(entries[i - 1].clone());
                }
                if i + 1 < entries.len() {
                    next = Some(entries[i + 1].clone());
                }
                break;
            }
        }

        Ok(Some(SeriesNav {
            series_slug,
            series_title,
            current_position,
            total_published,
            prev,
            next,
            entries,
        }))
    }
}

/// Get all posts in a series (as list items, ordered by position).
pub struct GetSeriesPosts(pub String); // series slug

impl Message<GetSeriesPosts> for BlogCache {
    type Reply = Result<Vec<BlogListItem>, String>;

    async fn handle(
        &mut self,
        msg: GetSeriesPosts,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let rows = sqlx::query(
            r#"
            SELECT id, slug, title, description, published_at, author, tags, featured,
                   reading_time_minutes, series_slug, series_title, series_position
            FROM blog_posts
            WHERE series_slug = $1 AND published = true
            ORDER BY series_position ASC NULLS LAST, published_at ASC, id ASC
            "#,
        )
        .bind(msg.0)
        .fetch_all(&self.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

        rows.into_iter()
            .map(rows::blog_list_item)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Database error: {e}"))
    }
}

/// Get all unique series with metadata.
pub struct GetAllSeries;

impl Message<GetAllSeries> for BlogCache {
    type Reply = Result<Vec<SeriesListItem>, String>;

    async fn handle(
        &mut self,
        _msg: GetAllSeries,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let rows = sqlx::query(
            r#"
            SELECT
                series_slug AS slug,
                COALESCE(series_title, series_slug) AS title,
                COUNT(*)::integer AS post_count,
                COALESCE(SUM(reading_time_minutes), 0)::integer AS total_reading_time,
                MAX(published_at) AS latest_published_at
            FROM blog_posts
            WHERE series_slug IS NOT NULL AND published = true
            GROUP BY series_slug, series_title
            ORDER BY latest_published_at DESC NULLS LAST, series_slug ASC
            "#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

        rows.into_iter()
            .map(rows::series_list_item)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Database error: {e}"))
    }
}

// ---------------------------------------------------------------------------
// Cache invalidation
// ---------------------------------------------------------------------------

/// Invalidate all caches (call after publishing new content).
pub struct InvalidateCache;

impl Message<InvalidateCache> for BlogCache {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: InvalidateCache,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.clear_all();
    }
}
