use leptos::prelude::*;
use plinth_shared::{SiteConfig, SiteContent};

/// Helper to deserialize SurrealDB query results via Value::into_json_value().
///
/// SurrealDB 3.0's `SurrealValue` impl for `serde_json::Value` can't convert
/// native Datetime or RecordId types. We instead take the raw `Value`,
/// convert via `into_json_value()` (which handles all types), then deserialize.
#[cfg(feature = "ssr")]
fn take_as<T: serde::de::DeserializeOwned>(
    result: &mut surrealdb::IndexedResults,
    idx: usize,
) -> Result<Vec<T>, ServerFnError> {
    use surrealdb::types::Value;
    let value: Value = result
        .take(idx)
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let values = match value {
        Value::Array(arr) => arr.into_vec(),
        Value::None => return Ok(vec![]),
        other => vec![other],
    };
    values
        .into_iter()
        .map(|v| {
            let json = v.into_json_value();
            serde_json::from_value(json).map_err(|e| ServerFnError::new(e.to_string()))
        })
        .collect()
}

// ── Core server functions (always present) ──────────────────────────────────

#[server(GetSiteConfig, "/api")]
pub async fn get_site_config() -> Result<SiteConfig, ServerFnError> {
    Ok(expect_context::<SiteConfig>())
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
    let items: Vec<SiteContent> = take_as(&mut result, 0)?;
    Ok(items.into_iter().next())
}

// ── Blog server functions ───────────────────────────────────────────────────

#[cfg(feature = "brick-blog")]
#[server(GetBlogPosts, "/api")]
pub async fn get_blog_posts() -> Result<Vec<plinth_shared::BlogListItem>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query("SELECT * FROM blog_posts WHERE published = true ORDER BY published_at DESC")
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    take_as(&mut result, 0)
}

#[cfg(feature = "brick-blog")]
#[server(GetBlogPostBySlug, "/api")]
pub async fn get_blog_post_by_slug(
    slug: String,
) -> Result<Option<plinth_shared::BlogPost>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query("SELECT * FROM blog_posts WHERE slug = $slug AND published = true")
        .bind(("slug", slug))
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    let posts: Vec<plinth_shared::BlogPost> = take_as(&mut result, 0)?;
    Ok(posts.into_iter().next())
}

#[cfg(feature = "brick-blog")]
#[server(GetBlogPostsByTag, "/api")]
pub async fn get_blog_posts_by_tag(
    tag: String,
) -> Result<Vec<plinth_shared::BlogListItem>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query(
            "SELECT * FROM blog_posts WHERE published = true AND $tag IN tags ORDER BY published_at DESC",
        )
        .bind(("tag", tag))
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    take_as(&mut result, 0)
}

#[cfg(feature = "brick-blog")]
#[server(GetSeriesNavFn, "/api")]
pub async fn get_series_nav(
    post_slug: String,
) -> Result<Option<plinth_shared::SeriesNav>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();

    let mut result = db
        .query("SELECT series_slug, series_title, series_position FROM blog_posts WHERE slug = $slug AND published = true LIMIT 1")
        .bind(("slug", post_slug.clone()))
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;

    #[derive(serde::Deserialize)]
    struct PostSeriesInfo {
        series_slug: Option<String>,
        series_title: Option<String>,
        series_position: Option<u32>,
    }

    let infos: Vec<PostSeriesInfo> = take_as(&mut result, 0)?;
    let info = match infos.into_iter().next() {
        Some(i) if i.series_slug.is_some() => i,
        _ => return Ok(None),
    };

    let series_slug = info.series_slug.unwrap();
    let series_title = info.series_title.unwrap_or_default();
    let current_position = info.series_position.unwrap_or(0);

    let mut result = db
        .query("SELECT slug, title, series_position FROM blog_posts WHERE series_slug = $series_slug AND published = true ORDER BY series_position ASC")
        .bind(("series_slug", series_slug.clone()))
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;

    let entries: Vec<plinth_shared::SeriesEntry> = take_as(&mut result, 0)?;
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

    Ok(Some(plinth_shared::SeriesNav {
        series_slug,
        series_title,
        current_position,
        total_published,
        prev,
        next,
        entries,
    }))
}

#[cfg(feature = "brick-blog")]
#[server(GetSeriesPostsFn, "/api")]
pub async fn get_series_posts(
    series_slug: String,
) -> Result<Vec<plinth_shared::BlogListItem>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query("SELECT * FROM blog_posts WHERE series_slug = $slug AND published = true ORDER BY series_position ASC")
        .bind(("slug", series_slug))
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;

    let posts: Vec<plinth_shared::BlogPost> = take_as(&mut result, 0)?;
    Ok(posts
        .iter()
        .map(plinth_shared::BlogListItem::from)
        .collect())
}

#[cfg(feature = "brick-blog")]
#[server(GetAllSeriesFn, "/api")]
pub async fn get_all_series() -> Result<Vec<plinth_shared::SeriesListItem>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
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
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    take_as(&mut result, 0)
}

// ── Portfolio server functions ──────────────────────────────────────────────

#[cfg(feature = "brick-portfolio")]
#[server(GetPortfolioItems, "/api")]
pub async fn get_portfolio_items() -> Result<Vec<plinth_shared::PortfolioItem>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query("SELECT * FROM portfolio_items ORDER BY order ASC, date DESC")
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    take_as(&mut result, 0)
}

#[cfg(feature = "brick-portfolio")]
#[server(GetPortfolioItemBySlug, "/api")]
pub async fn get_portfolio_item_by_slug(
    slug: String,
) -> Result<Option<plinth_shared::PortfolioItem>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query("SELECT * FROM portfolio_items WHERE slug = $slug")
        .bind(("slug", slug))
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    let items: Vec<plinth_shared::PortfolioItem> = take_as(&mut result, 0)?;
    Ok(items.into_iter().next())
}

// ── Todo server functions ───────────────────────────────────────────────────

#[cfg(feature = "brick-todo")]
#[server(GetTodos, "/api")]
pub async fn get_todos() -> Result<Vec<plinth_shared::TodoListItem>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query("SELECT * FROM todos ORDER BY completed ASC, order ASC, created_at DESC")
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    take_as(&mut result, 0)
}

#[cfg(feature = "brick-todo")]
#[server(GetTodoBySlug, "/api")]
pub async fn get_todo_by_slug(
    slug: String,
) -> Result<Option<plinth_shared::TodoItem>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query("SELECT * FROM todos WHERE slug = $slug")
        .bind(("slug", slug))
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    let items: Vec<plinth_shared::TodoItem> = take_as(&mut result, 0)?;
    Ok(items.into_iter().next())
}

#[cfg(feature = "brick-todo")]
#[server(GetTodosByTag, "/api")]
pub async fn get_todos_by_tag(
    tag: String,
) -> Result<Vec<plinth_shared::TodoListItem>, ServerFnError> {
    use surrealdb::{Surreal, engine::local::Db};
    let db = expect_context::<Surreal<Db>>();
    let mut result = db
        .query(
            "SELECT * FROM todos WHERE $tag IN tags ORDER BY completed ASC, order ASC, created_at DESC",
        )
        .bind(("tag", tag))
        .await
        .map_err(|e| -> ServerFnError { ServerFnError::ServerError(e.to_string()) })?;
    take_as(&mut result, 0)
}
