//! Blog cache actor — extracted from the monolithic ContentCache.
//!
//! Caches blog posts, list views, and series metadata in memory with
//! a TTL-based expiration strategy.  On cache miss the actor queries
//! SurrealDB directly.

use kameo::Actor;
use kameo::message::{Context, Message};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use surrealdb::Surreal;
use surrealdb::engine::local::Db;

use plinth_shared::{BlogListItem, BlogPost, SeriesEntry, SeriesListItem, SeriesNav};

use crate::db_helpers::{take_as, take_as_opt};

/// Cache entry TTL — entries older than this are treated as expired.
const CACHE_TTL: Duration = Duration::from_secs(5 * 60);

/// Maximum number of individually-cached items per category.
const MAX_ITEM_CACHE_SIZE: usize = 500;

/// Blog-specific cache actor that stores frequently accessed blog content
/// in memory and queries SurrealDB on cache misses.
#[derive(Actor)]
pub struct BlogCache {
    db: Surreal<Db>,
    blog_posts: HashMap<String, BlogPost>,
    blog_list_cache: Option<Vec<BlogListItem>>,
    /// Timestamp of the last cache population / invalidation.
    cache_populated_at: Option<Instant>,
}

impl BlogCache {
    /// Create a new BlogCache actor with a SurrealDB connection.
    pub fn new(db: Surreal<Db>) -> Self {
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

        // Query SurrealDB
        let mut response = self
            .db
            .query("SELECT * FROM blog_posts WHERE slug = $slug AND published = true")
            .bind(("slug", slug.clone()))
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let post: Option<BlogPost> =
            take_as_opt(&mut response, 0).map_err(|e| format!("Database error: {}", e))?;

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

        // Query SurrealDB
        let mut response = self
            .db
            .query("SELECT * FROM blog_posts WHERE published = true ORDER BY published_at DESC")
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let posts: Vec<BlogPost> =
            take_as(&mut response, 0).map_err(|e| format!("Database error: {}", e))?;

        // Convert to BlogListItem
        let list: Vec<BlogListItem> = posts.iter().map(BlogListItem::from).collect();

        // Update cache
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
        // Query via denormalized tags array
        let mut response = self
            .db
            .query(
                r#"SELECT * FROM blog_posts
                WHERE published = true AND $tag IN tags
                ORDER BY published_at DESC"#,
            )
            .bind(("tag", msg.0))
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let posts: Vec<BlogPost> =
            take_as(&mut response, 0).map_err(|e| format!("Database error: {}", e))?;

        let list: Vec<BlogListItem> = posts.iter().map(BlogListItem::from).collect();

        Ok(list)
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

        // First find the post's series_slug
        let mut response = self
            .db
            .query("SELECT series_slug, series_title, series_position FROM blog_posts WHERE slug = $slug AND published = true LIMIT 1")
            .bind(("slug", post_slug.clone()))
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        #[derive(serde::Deserialize)]
        struct PostSeriesInfo {
            series_slug: Option<String>,
            series_title: Option<String>,
            series_position: Option<u32>,
        }

        let info: Option<PostSeriesInfo> =
            take_as_opt(&mut response, 0).map_err(|e| format!("Database error: {}", e))?;

        let (series_slug, info) = match info {
            Some(i) => match i.series_slug.clone() {
                Some(slug) => (slug, i),
                None => return Ok(None),
            },
            None => return Ok(None),
        };
        let series_title = info.series_title.unwrap_or_default();
        let current_position = info.series_position.unwrap_or(0);

        // Get all published posts in this series, ordered by position
        let mut response = self
            .db
            .query(
                "SELECT slug, title, series_position FROM blog_posts WHERE series_slug = $series_slug AND published = true ORDER BY series_position ASC",
            )
            .bind(("series_slug", series_slug.clone()))
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let entries: Vec<SeriesEntry> =
            take_as(&mut response, 0).map_err(|e| format!("Database error: {}", e))?;

        let total_published = entries.len() as u32;

        // Find prev/next relative to current position
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
        let mut response = self
            .db
            .query(
                "SELECT * FROM blog_posts WHERE series_slug = $slug AND published = true ORDER BY series_position ASC",
            )
            .bind(("slug", msg.0))
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let posts: Vec<BlogPost> =
            take_as(&mut response, 0).map_err(|e| format!("Database error: {}", e))?;

        Ok(posts.iter().map(BlogListItem::from).collect())
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
        let mut response = self
            .db
            .query(
                r#"SELECT
                    series_slug AS slug,
                    series_title AS title,
                    count() AS post_count,
                    math::sum(reading_time_minutes) AS total_reading_time,
                    math::max(published_at) AS latest_published_at
                FROM blog_posts
                WHERE series_slug IS NOT NONE AND published = true
                GROUP BY series_slug, series_title
                ORDER BY latest_published_at DESC"#,
            )
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        take_as(&mut response, 0).map_err(|e| format!("Database error: {}", e))
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
