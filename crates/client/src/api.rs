use leptos::prelude::*;
use plinth_shared::{SiteConfig, SiteContent};

// ── Core server functions (always present) ──────────────────────────────────

#[server(GetSiteConfig, "/api")]
pub async fn get_site_config() -> Result<SiteConfig, ServerFnError> {
    Ok(expect_context::<SiteConfig>())
}

#[server(GetSiteContentFn, "/api")]
pub async fn get_site_content(key: String) -> Result<Option<SiteContent>, ServerFnError> {
    let _ = key;
    todo!("phase 03")
}

// ── Blog server functions ───────────────────────────────────────────────────

#[cfg(feature = "brick-blog")]
#[server(GetBlogPosts, "/api")]
pub async fn get_blog_posts() -> Result<Vec<plinth_shared::BlogListItem>, ServerFnError> {
    todo!("phase 03")
}

#[cfg(feature = "brick-blog")]
#[server(GetBlogPostBySlug, "/api")]
pub async fn get_blog_post_by_slug(
    slug: String,
) -> Result<Option<plinth_shared::BlogPost>, ServerFnError> {
    let _ = slug;
    todo!("phase 03")
}

#[cfg(feature = "brick-blog")]
#[server(GetBlogPostsByTag, "/api")]
pub async fn get_blog_posts_by_tag(
    tag: String,
) -> Result<Vec<plinth_shared::BlogListItem>, ServerFnError> {
    let _ = tag;
    todo!("phase 03")
}

#[cfg(feature = "brick-blog")]
#[server(GetSeriesNavFn, "/api")]
pub async fn get_series_nav(
    post_slug: String,
) -> Result<Option<plinth_shared::SeriesNav>, ServerFnError> {
    let _ = post_slug;
    todo!("phase 03")
}

#[cfg(feature = "brick-blog")]
#[server(GetSeriesPostsFn, "/api")]
pub async fn get_series_posts(
    series_slug: String,
) -> Result<Vec<plinth_shared::BlogListItem>, ServerFnError> {
    let _ = series_slug;
    todo!("phase 03")
}

#[cfg(feature = "brick-blog")]
#[server(GetAllSeriesFn, "/api")]
pub async fn get_all_series() -> Result<Vec<plinth_shared::SeriesListItem>, ServerFnError> {
    todo!("phase 03")
}

// ── Portfolio server functions ──────────────────────────────────────────────

#[cfg(feature = "brick-portfolio")]
#[server(GetPortfolioItems, "/api")]
pub async fn get_portfolio_items() -> Result<Vec<plinth_shared::PortfolioItem>, ServerFnError> {
    todo!("phase 03")
}

#[cfg(feature = "brick-portfolio")]
#[server(GetPortfolioItemBySlug, "/api")]
pub async fn get_portfolio_item_by_slug(
    slug: String,
) -> Result<Option<plinth_shared::PortfolioItem>, ServerFnError> {
    let _ = slug;
    todo!("phase 03")
}

// ── Todo server functions ───────────────────────────────────────────────────

#[cfg(feature = "brick-todo")]
#[server(GetTodos, "/api")]
pub async fn get_todos() -> Result<Vec<plinth_shared::TodoListItem>, ServerFnError> {
    todo!("phase 03")
}

#[cfg(feature = "brick-todo")]
#[server(GetTodoBySlug, "/api")]
pub async fn get_todo_by_slug(
    slug: String,
) -> Result<Option<plinth_shared::TodoItem>, ServerFnError> {
    let _ = slug;
    todo!("phase 03")
}

#[cfg(feature = "brick-todo")]
#[server(GetTodosByTag, "/api")]
pub async fn get_todos_by_tag(
    tag: String,
) -> Result<Vec<plinth_shared::TodoListItem>, ServerFnError> {
    let _ = tag;
    todo!("phase 03")
}
