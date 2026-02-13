use kameo::message::{Context, Message};
use kameo::Actor;
use std::collections::HashMap;
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

use shared::{BlogListItem, BlogPost, PortfolioItem};

/// Content cache actor that stores frequently accessed content in memory
/// and queries SurrealDB on cache misses
#[derive(Actor)]
pub struct ContentCache {
    db: Surreal<Db>,
    blog_posts: HashMap<String, BlogPost>,
    blog_list_cache: Option<Vec<BlogListItem>>,
    portfolio_items: HashMap<String, PortfolioItem>,
    portfolio_list_cache: Option<Vec<PortfolioItem>>,
}

impl ContentCache {
    /// Create a new ContentCache actor with a SurrealDB connection
    pub fn new(db: Surreal<Db>) -> Self {
        Self {
            db,
            blog_posts: HashMap::new(),
            blog_list_cache: None,
            portfolio_items: HashMap::new(),
            portfolio_list_cache: None,
        }
    }
}

// Messages for blog posts

/// Get a single blog post by slug
pub struct GetBlogPost(pub String);

impl Message<GetBlogPost> for ContentCache {
    type Reply = Result<Option<BlogPost>, String>;

    async fn handle(
        &mut self,
        msg: GetBlogPost,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let slug = msg.0;

        // Check cache first
        if let Some(post) = self.blog_posts.get(&slug) {
            return Ok(Some(post.clone()));
        }

        // Query SurrealDB
        let result: Result<Option<BlogPost>, _> = self
            .db
            .query("SELECT * FROM blog_posts WHERE slug = $slug AND published = true")
            .bind(("slug", slug.clone()))
            .await
            .and_then(|mut response| response.take(0));

        match result {
            Ok(Some(post)) => {
                // Update cache
                self.blog_posts.insert(slug, post.clone());
                Ok(Some(post))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }
}

/// Get all published blog posts (as list items)
pub struct GetAllBlogPosts;

impl Message<GetAllBlogPosts> for ContentCache {
    type Reply = Result<Vec<BlogListItem>, String>;

    async fn handle(
        &mut self,
        _msg: GetAllBlogPosts,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Check cache first
        if let Some(ref list) = self.blog_list_cache {
            return Ok(list.clone());
        }

        // Query SurrealDB
        let result: Result<Vec<BlogPost>, _> = self
            .db
            .query("SELECT * FROM blog_posts WHERE published = true ORDER BY published_at DESC")
            .await
            .and_then(|mut response| response.take(0));

        match result {
            Ok(posts) => {
                // Convert to BlogListItem
                let list: Vec<BlogListItem> = posts
                    .iter()
                    .map(|p| BlogListItem {
                        id: p.id.clone(),
                        slug: p.slug.clone(),
                        title: p.title.clone(),
                        description: p.content.chars().take(200).collect::<String>() + "...",
                        published_at: p.published_at,
                        author: p.author.clone(),
                        tags: p.tags.clone(),
                        featured: p.featured,
                        reading_time_minutes: p.reading_time_minutes,
                    })
                    .collect();

                // Update cache
                self.blog_list_cache = Some(list.clone());

                Ok(list)
            }
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }
}

/// Get blog posts by tag
pub struct GetPostsByTag(pub String);

impl Message<GetPostsByTag> for ContentCache {
    type Reply = Result<Vec<BlogListItem>, String>;

    async fn handle(
        &mut self,
        msg: GetPostsByTag,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Query SurrealDB (don't cache tag-specific queries for now)
        let result: Result<Vec<BlogPost>, _> = self.db
            .query("SELECT * FROM blog_posts WHERE published = true AND $tag IN tags ORDER BY published_at DESC")
            .bind(("tag", msg.0))
            .await
            .and_then(|mut response| response.take(0));

        match result {
            Ok(posts) => {
                let list: Vec<BlogListItem> = posts
                    .iter()
                    .map(|p| BlogListItem {
                        id: p.id.clone(),
                        slug: p.slug.clone(),
                        title: p.title.clone(),
                        description: p.content.chars().take(200).collect::<String>() + "...",
                        published_at: p.published_at,
                        author: p.author.clone(),
                        tags: p.tags.clone(),
                        featured: p.featured,
                        reading_time_minutes: p.reading_time_minutes,
                    })
                    .collect();

                Ok(list)
            }
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }
}

// Messages for portfolio items

/// Get a single portfolio item by slug
pub struct GetPortfolioItem(pub String);

impl Message<GetPortfolioItem> for ContentCache {
    type Reply = Result<Option<PortfolioItem>, String>;

    async fn handle(
        &mut self,
        msg: GetPortfolioItem,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let slug = msg.0;

        // Check cache first
        if let Some(item) = self.portfolio_items.get(&slug) {
            return Ok(Some(item.clone()));
        }

        // Query SurrealDB
        let result: Result<Option<PortfolioItem>, _> = self
            .db
            .query("SELECT * FROM portfolio_items WHERE slug = $slug")
            .bind(("slug", slug.clone()))
            .await
            .and_then(|mut response| response.take(0));

        match result {
            Ok(Some(item)) => {
                // Update cache
                self.portfolio_items.insert(slug, item.clone());
                Ok(Some(item))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }
}

/// Get all portfolio items
pub struct GetAllPortfolioItems;

impl Message<GetAllPortfolioItems> for ContentCache {
    type Reply = Result<Vec<PortfolioItem>, String>;

    async fn handle(
        &mut self,
        _msg: GetAllPortfolioItems,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Check cache first
        if let Some(ref list) = self.portfolio_list_cache {
            return Ok(list.clone());
        }

        // Query SurrealDB
        let result: Result<Vec<PortfolioItem>, _> = self
            .db
            .query("SELECT * FROM portfolio_items ORDER BY order ASC, date DESC")
            .await
            .and_then(|mut response| response.take(0));

        match result {
            Ok(items) => {
                // Update cache
                self.portfolio_list_cache = Some(items.clone());
                Ok(items)
            }
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }
}

// Cache invalidation

/// Invalidate all caches (call after publishing new content)
pub struct InvalidateCache;

impl Message<InvalidateCache> for ContentCache {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: InvalidateCache,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.blog_posts.clear();
        self.blog_list_cache = None;
        self.portfolio_items.clear();
        self.portfolio_list_cache = None;

        println!("🔄 Content cache invalidated");
    }
}
