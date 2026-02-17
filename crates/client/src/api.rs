use leptos::prelude::*;
use plinth_shared::{
    BlogListItem, BlogPost, PortfolioItem, SiteConfig, SiteContent, TodoItem, TodoListItem,
};

#[server(GetSiteConfig, "/api")]
pub async fn get_site_config() -> Result<SiteConfig, ServerFnError> {
    Ok(expect_context::<SiteConfig>())
}

#[server(GetBlogPosts, "/api")]
pub async fn get_blog_posts() -> Result<Vec<BlogListItem>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query("SELECT * FROM blog_posts WHERE published = true ORDER BY published_at DESC")
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    let posts: Vec<BlogListItem> = result
        .take(0)
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    Ok(posts)
}

#[server(GetBlogPostBySlug, "/api")]
pub async fn get_blog_post_by_slug(slug: String) -> Result<Option<BlogPost>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query("SELECT * FROM blog_posts WHERE slug = $slug AND published = true")
        .bind(("slug", slug))
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    let posts: Vec<BlogPost> = result
        .take(0)
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    Ok(posts.into_iter().next())
}

#[server(GetBlogPostsByTag, "/api")]
pub async fn get_blog_posts_by_tag(tag: String) -> Result<Vec<BlogListItem>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query(
            "SELECT * FROM blog_posts WHERE published = true AND $tag IN tags ORDER BY published_at DESC",
        )
        .bind(("tag", tag))
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    let posts: Vec<BlogListItem> = result
        .take(0)
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    Ok(posts)
}

#[server(GetPortfolioItems, "/api")]
pub async fn get_portfolio_items() -> Result<Vec<PortfolioItem>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query("SELECT * FROM portfolio_items ORDER BY order ASC, date DESC")
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    let items: Vec<PortfolioItem> = result
        .take(0)
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    Ok(items)
}

#[server(GetPortfolioItemBySlug, "/api")]
pub async fn get_portfolio_item_by_slug(
    slug: String,
) -> Result<Option<PortfolioItem>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query("SELECT * FROM portfolio_items WHERE slug = $slug")
        .bind(("slug", slug))
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    let items: Vec<PortfolioItem> = result
        .take(0)
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    Ok(items.into_iter().next())
}

#[server(GetSiteContentFn, "/api")]
pub async fn get_site_content(key: String) -> Result<Option<SiteContent>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query("SELECT * FROM site_content WHERE key = $key LIMIT 1")
        .bind(("key", key))
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    let items: Vec<SiteContent> = result
        .take(0)
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    Ok(items.into_iter().next())
}

#[server(GetTodos, "/api")]
pub async fn get_todos() -> Result<Vec<TodoListItem>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query("SELECT * FROM todos ORDER BY completed ASC, order ASC, created_at DESC")
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    let items: Vec<TodoListItem> = result
        .take(0)
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    Ok(items)
}

#[server(GetTodoBySlug, "/api")]
pub async fn get_todo_by_slug(slug: String) -> Result<Option<TodoItem>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query("SELECT * FROM todos WHERE slug = $slug")
        .bind(("slug", slug))
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    let items: Vec<TodoItem> = result
        .take(0)
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    Ok(items.into_iter().next())
}

#[server(GetTodosByTag, "/api")]
pub async fn get_todos_by_tag(tag: String) -> Result<Vec<TodoListItem>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query(
            "SELECT * FROM todos WHERE $tag IN tags ORDER BY completed ASC, order ASC, created_at DESC",
        )
        .bind(("tag", tag))
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    let items: Vec<TodoListItem> = result
        .take(0)
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    Ok(items)
}
