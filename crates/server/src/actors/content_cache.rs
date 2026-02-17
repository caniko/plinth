use kameo::Actor;
use kameo::message::{Context, Message};
use std::collections::HashMap;
use surrealdb::Surreal;
use surrealdb::engine::local::Db;

use plinth_shared::{
    BlogListItem, BlogPost, PortfolioItem, SiteContent, Tag, TodoItem, TodoListItem,
};

/// Content cache actor that stores frequently accessed content in memory
/// and queries SurrealDB on cache misses
#[derive(Actor)]
pub struct ContentCache {
    db: Surreal<Db>,
    blog_posts: HashMap<String, BlogPost>,
    blog_list_cache: Option<Vec<BlogListItem>>,
    portfolio_items: HashMap<String, PortfolioItem>,
    portfolio_list_cache: Option<Vec<PortfolioItem>>,
    site_content: HashMap<String, SiteContent>,
    todo_items: HashMap<String, TodoItem>,
    todo_list_cache: Option<Vec<TodoListItem>>,
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
            site_content: HashMap::new(),
            todo_items: HashMap::new(),
            todo_list_cache: None,
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
                        description: if p.description.is_empty() {
                            p.content.chars().take(200).collect::<String>() + "..."
                        } else {
                            p.description.clone()
                        },
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
        // Query via graph: find tag, then reverse-traverse to posts
        // Falls back to denormalized field for compatibility
        let result: Result<Vec<BlogPost>, _> = self
            .db
            .query(
                r#"SELECT * FROM blog_posts
                WHERE published = true AND $tag IN tags
                ORDER BY published_at DESC"#,
            )
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
                        description: if p.description.is_empty() {
                            p.content.chars().take(200).collect::<String>() + "..."
                        } else {
                            p.description.clone()
                        },
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

// Tag queries

/// Get all tags with post counts
pub struct GetAllTags;

impl Message<GetAllTags> for ContentCache {
    type Reply = Result<Vec<Tag>, String>;

    async fn handle(
        &mut self,
        _msg: GetAllTags,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let result: Result<Vec<Tag>, _> = self
            .db
            .query(
                "SELECT *, count(<-tagged<-blog_posts) AS post_count, count(<-todo_tagged<-todos) AS todo_count FROM tags ORDER BY name ASC",
            )
            .await
            .and_then(|mut response| response.take(0));

        match result {
            Ok(tags) => Ok(tags),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }
}

// Site content

/// Get site content by key (e.g. "home-intro", "about")
pub struct GetSiteContent(pub String);

impl Message<GetSiteContent> for ContentCache {
    type Reply = Result<Option<SiteContent>, String>;

    async fn handle(
        &mut self,
        msg: GetSiteContent,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let key = msg.0;

        // Check cache first
        if let Some(content) = self.site_content.get(&key) {
            return Ok(Some(content.clone()));
        }

        // Query SurrealDB
        let result: Result<Option<SiteContent>, _> = self
            .db
            .query("SELECT * FROM site_content WHERE key = $key LIMIT 1")
            .bind(("key", key.clone()))
            .await
            .and_then(|mut response| response.take(0));

        match result {
            Ok(Some(content)) => {
                self.site_content.insert(key, content.clone());
                Ok(Some(content))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }
}

// Messages for TODO items

/// Get a single TODO item by slug
pub struct GetTodoItem(pub String);

impl Message<GetTodoItem> for ContentCache {
    type Reply = Result<Option<TodoItem>, String>;

    async fn handle(
        &mut self,
        msg: GetTodoItem,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let slug = msg.0;

        // Check cache first
        if let Some(item) = self.todo_items.get(&slug) {
            return Ok(Some(item.clone()));
        }

        // Query SurrealDB
        let result: Result<Option<TodoItem>, _> = self
            .db
            .query("SELECT * FROM todos WHERE slug = $slug")
            .bind(("slug", slug.clone()))
            .await
            .and_then(|mut response| response.take(0));

        match result {
            Ok(Some(item)) => {
                self.todo_items.insert(slug, item.clone());
                Ok(Some(item))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }
}

/// Get all TODO items (as list items, pending first)
pub struct GetAllTodos;

impl Message<GetAllTodos> for ContentCache {
    type Reply = Result<Vec<TodoListItem>, String>;

    async fn handle(
        &mut self,
        _msg: GetAllTodos,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Check cache first
        if let Some(ref list) = self.todo_list_cache {
            return Ok(list.clone());
        }

        // Query SurrealDB — pending items first, then by order, then newest first
        let result: Result<Vec<TodoListItem>, _> = self
            .db
            .query("SELECT * FROM todos ORDER BY completed ASC, order ASC, created_at DESC")
            .await
            .and_then(|mut response| response.take(0));

        match result {
            Ok(items) => {
                self.todo_list_cache = Some(items.clone());
                Ok(items)
            }
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }
}

/// Get TODO items by tag
pub struct GetTodosByTag(pub String);

impl Message<GetTodosByTag> for ContentCache {
    type Reply = Result<Vec<TodoListItem>, String>;

    async fn handle(
        &mut self,
        msg: GetTodosByTag,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let result: Result<Vec<TodoListItem>, _> = self
            .db
            .query(
                r#"SELECT * FROM todos
                WHERE $tag IN tags
                ORDER BY completed ASC, order ASC, created_at DESC"#,
            )
            .bind(("tag", msg.0))
            .await
            .and_then(|mut response| response.take(0));

        match result {
            Ok(items) => Ok(items),
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
        self.site_content.clear();
        self.todo_items.clear();
        self.todo_list_cache = None;
    }
}
