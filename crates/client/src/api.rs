use leptos::prelude::*;
use plinth_shared::{SiteConfig, SiteContent};

#[cfg(all(feature = "csr", not(feature = "ssr")))]
fn encode_segment(value: &str) -> String {
    use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}

#[cfg(all(feature = "csr", target_arch = "wasm32"))]
fn api_url(path: &str) -> String {
    let base = option_env!("PLINTH_CSR_API_BASE")
        .unwrap_or("")
        .trim_end_matches('/');
    if base.is_empty() {
        path.to_string()
    } else {
        format!("{base}{path}")
    }
}

#[cfg(all(feature = "csr", not(feature = "ssr"), target_arch = "wasm32"))]
async fn fetch_json<T>(path: &str) -> Result<T, ServerFnError>
where
    T: serde::de::DeserializeOwned,
{
    send_wrapper::SendWrapper::new(fetch_json_inner(path.to_string())).await
}

#[cfg(all(feature = "csr", not(feature = "ssr"), target_arch = "wasm32"))]
async fn fetch_json_inner<T>(path: String) -> Result<T, ServerFnError>
where
    T: serde::de::DeserializeOwned,
{
    let url = api_url(&path);
    let response = gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if !response.ok() {
        return Err(ServerFnError::new(format!(
            "GET {url} returned HTTP {}",
            response.status()
        )));
    }

    response
        .json()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[cfg(all(feature = "csr", not(feature = "ssr"), not(target_arch = "wasm32")))]
async fn fetch_json<T>(path: &str) -> Result<T, ServerFnError>
where
    T: serde::de::DeserializeOwned,
{
    let _ = path;
    Err(ServerFnError::new(
        "CSR REST fetches are only available in wasm32 builds",
    ))
}

// ── Core server functions (always present) ──────────────────────────────────

#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetSiteConfig, "/api")]
pub async fn get_site_config() -> Result<SiteConfig, ServerFnError> {
    Ok(expect_context::<SiteConfig>())
}

#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_site_config() -> Result<SiteConfig, ServerFnError> {
    fetch_json("/api/config").await
}

#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetSiteContentFn, "/api")]
pub async fn get_site_content(key: String) -> Result<Option<SiteContent>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_site_content(&db, key)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = key;
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_site_content(key: String) -> Result<Option<SiteContent>, ServerFnError> {
    fetch_json(&format!("/api/content/{}", encode_segment(&key))).await
}

// ── Blog server functions ───────────────────────────────────────────────────

#[cfg(feature = "brick-blog")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetBlogPosts, "/api")]
pub async fn get_blog_posts() -> Result<Vec<plinth_shared::BlogListItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_blog_posts(&db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-blog")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_blog_posts() -> Result<Vec<plinth_shared::BlogListItem>, ServerFnError> {
    fetch_json("/api/posts").await
}

#[cfg(feature = "brick-blog")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetBlogPostBySlug, "/api")]
pub async fn get_blog_post_by_slug(
    slug: String,
) -> Result<Option<plinth_shared::BlogPost>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_blog_post_by_slug(&db, slug)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = slug;
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-blog")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_blog_post_by_slug(
    slug: String,
) -> Result<Option<plinth_shared::BlogPost>, ServerFnError> {
    fetch_json(&format!("/api/posts/{}", encode_segment(&slug))).await
}

#[cfg(feature = "brick-blog")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetBlogPostsByTag, "/api")]
pub async fn get_blog_posts_by_tag(
    tag: String,
) -> Result<Vec<plinth_shared::BlogListItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_blog_posts_by_tag(&db, tag)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = tag;
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-blog")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_blog_posts_by_tag(
    tag: String,
) -> Result<Vec<plinth_shared::BlogListItem>, ServerFnError> {
    fetch_json(&format!("/api/posts/tag/{}", encode_segment(&tag))).await
}

#[cfg(feature = "brick-blog")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetSeriesNavFn, "/api")]
pub async fn get_series_nav(
    post_slug: String,
) -> Result<Option<plinth_shared::SeriesNav>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_series_nav(&db, post_slug)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = post_slug;
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-blog")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_series_nav(
    post_slug: String,
) -> Result<Option<plinth_shared::SeriesNav>, ServerFnError> {
    fetch_json(&format!(
        "/api/posts/{}/series-nav",
        encode_segment(&post_slug)
    ))
    .await
}

#[cfg(feature = "brick-blog")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetSeriesPostsFn, "/api")]
pub async fn get_series_posts(
    series_slug: String,
) -> Result<Vec<plinth_shared::BlogListItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_series_posts(&db, series_slug)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = series_slug;
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-blog")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_series_posts(
    series_slug: String,
) -> Result<Vec<plinth_shared::BlogListItem>, ServerFnError> {
    fetch_json(&format!(
        "/api/series/{}/posts",
        encode_segment(&series_slug)
    ))
    .await
}

#[cfg(feature = "brick-blog")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetAllSeriesFn, "/api")]
pub async fn get_all_series() -> Result<Vec<plinth_shared::SeriesListItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_all_series(&db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-blog")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_all_series() -> Result<Vec<plinth_shared::SeriesListItem>, ServerFnError> {
    fetch_json("/api/series").await
}

// ── Portfolio server functions ──────────────────────────────────────────────

#[cfg(feature = "brick-portfolio")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetPortfolioItems, "/api")]
pub async fn get_portfolio_items() -> Result<Vec<plinth_shared::PortfolioItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_portfolio_items(&db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-portfolio")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_portfolio_items() -> Result<Vec<plinth_shared::PortfolioItem>, ServerFnError> {
    fetch_json("/api/portfolio").await
}

#[cfg(feature = "brick-portfolio")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetPortfolioItemBySlug, "/api")]
pub async fn get_portfolio_item_by_slug(
    slug: String,
) -> Result<Option<plinth_shared::PortfolioItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_portfolio_item_by_slug(&db, slug)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = slug;
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-portfolio")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_portfolio_item_by_slug(
    slug: String,
) -> Result<Option<plinth_shared::PortfolioItem>, ServerFnError> {
    fetch_json(&format!("/api/portfolio/{}", encode_segment(&slug))).await
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
fn postgres_id(table: &str, value: i64) -> Option<String> {
    Some(format!("{table}:{value}"))
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
fn decode_error(message: impl Into<String>) -> sqlx::Error {
    sqlx::Error::Decode(Box::new(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        message.into(),
    )))
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
fn as_u32(value: i32, column: &str) -> Result<u32, sqlx::Error> {
    value
        .try_into()
        .map_err(|_| decode_error(format!("{column} contained negative value {value}")))
}

#[cfg(all(feature = "brick-blog", feature = "ssr"))]
#[allow(dead_code)]
fn content_format(value: String) -> Result<plinth_shared::ContentFormat, sqlx::Error> {
    match value.as_str() {
        "markdown" => Ok(plinth_shared::ContentFormat::Markdown),
        "typst" => Ok(plinth_shared::ContentFormat::Typst),
        other => Err(decode_error(format!("unknown content format '{other}'"))),
    }
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
fn row_site_content(row: sqlx::postgres::PgRow) -> Result<SiteContent, sqlx::Error> {
    use sqlx::Row;

    Ok(SiteContent {
        id: postgres_id("site_content", row.try_get("id")?),
        key: row.try_get("key")?,
        title: row.try_get("title")?,
        content: row.try_get("content")?,
        html_content: row.try_get("html_content")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
async fn query_site_content(
    db: &sqlx::PgPool,
    key: String,
) -> Result<Option<SiteContent>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM site_content WHERE key = $1 LIMIT 1")
        .bind(key)
        .fetch_optional(db)
        .await?;

    row.map(row_site_content).transpose()
}

#[cfg(all(feature = "brick-blog", feature = "ssr"))]
#[allow(dead_code)]
fn row_blog_list_item(
    row: sqlx::postgres::PgRow,
) -> Result<plinth_shared::BlogListItem, sqlx::Error> {
    use sqlx::Row;

    Ok(plinth_shared::BlogListItem {
        id: postgres_id("blog_posts", row.try_get("id")?),
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        published_at: row.try_get("published_at")?,
        author: row.try_get("author")?,
        tags: row.try_get("tags")?,
        featured: row.try_get("featured")?,
        reading_time_minutes: as_u32(row.try_get("reading_time_minutes")?, "reading_time_minutes")?,
        series_slug: row.try_get("series_slug")?,
        series_title: row.try_get("series_title")?,
        series_position: row
            .try_get::<Option<i32>, _>("series_position")?
            .map(|v| as_u32(v, "series_position"))
            .transpose()?,
    })
}

#[cfg(all(feature = "brick-blog", feature = "ssr"))]
#[allow(dead_code)]
fn row_blog_post(row: sqlx::postgres::PgRow) -> Result<plinth_shared::BlogPost, sqlx::Error> {
    use sqlx::Row;

    Ok(plinth_shared::BlogPost {
        id: postgres_id("blog_posts", row.try_get("id")?),
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        content: row.try_get("content")?,
        html_content: row.try_get("html_content")?,
        published_at: row.try_get("published_at")?,
        updated_at: row.try_get("updated_at")?,
        author: row.try_get("author")?,
        tags: row.try_get("tags")?,
        featured: row.try_get("featured")?,
        published: row.try_get("published")?,
        reading_time_minutes: as_u32(row.try_get("reading_time_minutes")?, "reading_time_minutes")?,
        embedding: None,
        content_format: content_format(row.try_get("content_format")?)?,
        source: row.try_get("source")?,
        content_hash: row.try_get("content_hash")?,
        series_slug: row.try_get("series_slug")?,
        series_title: row.try_get("series_title")?,
        series_position: row
            .try_get::<Option<i32>, _>("series_position")?
            .map(|v| as_u32(v, "series_position"))
            .transpose()?,
    })
}

#[cfg(all(feature = "brick-blog", feature = "ssr"))]
#[allow(dead_code)]
fn row_series_entry(row: sqlx::postgres::PgRow) -> Result<plinth_shared::SeriesEntry, sqlx::Error> {
    use sqlx::Row;

    Ok(plinth_shared::SeriesEntry {
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        position: as_u32(row.try_get("position")?, "position")?,
    })
}

#[cfg(all(feature = "brick-blog", feature = "ssr"))]
#[allow(dead_code)]
fn row_series_list_item(
    row: sqlx::postgres::PgRow,
) -> Result<plinth_shared::SeriesListItem, sqlx::Error> {
    use sqlx::Row;

    Ok(plinth_shared::SeriesListItem {
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        post_count: as_u32(row.try_get("post_count")?, "post_count")?,
        total_reading_time: as_u32(row.try_get("total_reading_time")?, "total_reading_time")?,
        latest_published_at: row.try_get("latest_published_at")?,
    })
}

#[cfg(all(feature = "brick-blog", feature = "ssr"))]
#[allow(dead_code)]
async fn query_blog_posts(
    db: &sqlx::PgPool,
) -> Result<Vec<plinth_shared::BlogListItem>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, slug, title, description, published_at, author, tags, featured,
               reading_time_minutes, series_slug, series_title, series_position
        FROM blog_posts
        WHERE published = true
        ORDER BY published_at DESC, id DESC
        "#,
    )
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_blog_list_item).collect()
}

#[cfg(all(feature = "brick-blog", feature = "ssr"))]
#[allow(dead_code)]
async fn query_blog_post_by_slug(
    db: &sqlx::PgPool,
    slug: String,
) -> Result<Option<plinth_shared::BlogPost>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, slug, title, description, content, html_content, published_at,
               updated_at, author, tags, featured, published, reading_time_minutes,
               content_format, source, content_hash, series_slug, series_title,
               series_position
        FROM blog_posts
        WHERE slug = $1 AND published = true
        LIMIT 1
        "#,
    )
    .bind(slug)
    .fetch_optional(db)
    .await?;

    row.map(row_blog_post).transpose()
}

#[cfg(all(feature = "brick-blog", feature = "ssr"))]
#[allow(dead_code)]
async fn query_blog_posts_by_tag(
    db: &sqlx::PgPool,
    tag: String,
) -> Result<Vec<plinth_shared::BlogListItem>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT bp.id, bp.slug, bp.title, bp.description, bp.published_at,
               bp.author, bp.tags, bp.featured, bp.reading_time_minutes,
               bp.series_slug, bp.series_title, bp.series_position
        FROM blog_posts bp
        JOIN blog_post_tags bpt ON bpt.post_id = bp.id
        JOIN tags t ON t.id = bpt.tag_id
        WHERE bp.published = true AND (t.name = $1 OR t.slug = $1)
        ORDER BY bp.published_at DESC, bp.id DESC
        "#,
    )
    .bind(tag)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_blog_list_item).collect()
}

#[cfg(all(feature = "brick-blog", feature = "ssr"))]
#[allow(dead_code)]
async fn query_series_nav(
    db: &sqlx::PgPool,
    post_slug: String,
) -> Result<Option<plinth_shared::SeriesNav>, sqlx::Error> {
    use sqlx::Row;

    let info = sqlx::query(
        r#"
        SELECT series_slug, series_title, series_position
        FROM blog_posts
        WHERE slug = $1 AND published = true
        LIMIT 1
        "#,
    )
    .bind(post_slug)
    .fetch_optional(db)
    .await?;

    let Some(info) = info else {
        return Ok(None);
    };

    let Some(series_slug) = info.try_get::<Option<String>, _>("series_slug")? else {
        return Ok(None);
    };
    let series_title = info
        .try_get::<Option<String>, _>("series_title")?
        .unwrap_or_else(|| plinth_shared::humanize_slug(&series_slug));
    let current_position = info
        .try_get::<Option<i32>, _>("series_position")?
        .map(|v| as_u32(v, "series_position"))
        .transpose()?
        .unwrap_or(0);

    let rows = sqlx::query(
        r#"
        SELECT slug, title, COALESCE(series_position, 0) AS position
        FROM blog_posts
        WHERE series_slug = $1 AND published = true
        ORDER BY series_position ASC NULLS LAST, published_at ASC, id ASC
        "#,
    )
    .bind(&series_slug)
    .fetch_all(db)
    .await?;

    let entries = rows
        .into_iter()
        .map(row_series_entry)
        .collect::<Result<Vec<_>, _>>()?;

    let total_published = entries.len() as u32;
    let current_index = entries
        .iter()
        .position(|entry| entry.position == current_position);
    let prev = current_index.and_then(|i| i.checked_sub(1).map(|prev| entries[prev].clone()));
    let next = current_index.and_then(|i| entries.get(i + 1).cloned());

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

#[cfg(all(feature = "brick-blog", feature = "ssr"))]
#[allow(dead_code)]
async fn query_series_posts(
    db: &sqlx::PgPool,
    series_slug: String,
) -> Result<Vec<plinth_shared::BlogListItem>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, slug, title, description, published_at, author, tags, featured,
               reading_time_minutes, series_slug, series_title, series_position
        FROM blog_posts
        WHERE series_slug = $1 AND published = true
        ORDER BY series_position ASC NULLS LAST, published_at ASC, id ASC
        "#,
    )
    .bind(series_slug)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_blog_list_item).collect()
}

#[cfg(all(feature = "brick-blog", feature = "ssr"))]
#[allow(dead_code)]
async fn query_all_series(
    db: &sqlx::PgPool,
) -> Result<Vec<plinth_shared::SeriesListItem>, sqlx::Error> {
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
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_series_list_item).collect()
}

#[cfg(all(feature = "brick-portfolio", feature = "ssr"))]
#[allow(dead_code)]
fn row_portfolio_item(
    row: sqlx::postgres::PgRow,
) -> Result<plinth_shared::PortfolioItem, sqlx::Error> {
    use sqlx::Row;

    Ok(plinth_shared::PortfolioItem {
        id: postgres_id("portfolio_items", row.try_get("id")?),
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        content: row.try_get("content")?,
        html_content: row.try_get("html_content")?,
        tech_stack: row.try_get("tech_stack")?,
        link: row.try_get("link")?,
        demo: row.try_get("demo")?,
        image_url: row.try_get("image_url")?,
        date: row.try_get("date")?,
        featured: row.try_get("featured")?,
        order: row.try_get("order")?,
    })
}

#[cfg(all(feature = "brick-portfolio", feature = "ssr"))]
#[allow(dead_code)]
async fn query_portfolio_items(
    db: &sqlx::PgPool,
) -> Result<Vec<plinth_shared::PortfolioItem>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT *
        FROM portfolio_items
        ORDER BY "order" ASC, date DESC, id DESC
        "#,
    )
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_portfolio_item).collect()
}

#[cfg(all(feature = "brick-portfolio", feature = "ssr"))]
#[allow(dead_code)]
async fn query_portfolio_item_by_slug(
    db: &sqlx::PgPool,
    slug: String,
) -> Result<Option<plinth_shared::PortfolioItem>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM portfolio_items WHERE slug = $1 LIMIT 1")
        .bind(slug)
        .fetch_optional(db)
        .await?;

    row.map(row_portfolio_item).transpose()
}

// ── Activity server functions ───────────────────────────────────────────────

#[cfg(feature = "brick-activity")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetActivityList, "/api")]
pub async fn get_activity_list() -> Result<Vec<plinth_shared::ActivityListItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        let config = plinth_shared::toml_config::PlinthConfig::load()
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        query_activity_list(&db, &config.ranking, Some(50))
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-activity")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_activity_list() -> Result<Vec<plinth_shared::ActivityListItem>, ServerFnError> {
    fetch_json("/api/activity?limit=50").await
}

#[cfg(feature = "brick-activity")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetActivityItemById, "/api")]
pub async fn get_activity_item_by_id(
    id: i64,
) -> Result<Option<plinth_shared::ActivityItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_activity_item(&db, id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = id;
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-activity")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_activity_item_by_id(
    id: i64,
) -> Result<Option<plinth_shared::ActivityItem>, ServerFnError> {
    fetch_json(&format!("/api/activity/{id}")).await
}

#[cfg(all(feature = "brick-activity", feature = "ssr"))]
#[allow(dead_code)]
fn activity_ref_date_sql() -> &'static str {
    "coalesce(merged_at, closed_at, created_at)"
}

#[cfg(all(feature = "brick-activity", feature = "ssr"))]
#[allow(dead_code)]
fn activity_age_days_sql() -> String {
    format!(
        "(extract(epoch from (now() - {})) / 86400.0)",
        activity_ref_date_sql()
    )
}

#[cfg(all(feature = "brick-activity", feature = "ssr"))]
#[allow(dead_code)]
fn activity_score_sql(strategy: plinth_shared::RankingStrategy, param_placeholder: &str) -> String {
    let age = activity_age_days_sql();
    match strategy {
        plinth_shared::RankingStrategy::Exponential => {
            format!(
                "(impact::float8 * power(0.5, {age} / greatest({param_placeholder}, 0.000001)))"
            )
        }
        plinth_shared::RankingStrategy::Linear => {
            format!(
                "(impact::float8 * greatest(0.0, 1.0 - {age} / greatest({param_placeholder}, 0.000001)))"
            )
        }
        plinth_shared::RankingStrategy::Pure => "(impact::float8)".to_string(),
    }
}

#[cfg(all(feature = "brick-activity", feature = "ssr"))]
#[allow(dead_code)]
fn activity_score_param(ranking: &plinth_shared::toml_config::RankingConfig) -> f64 {
    match ranking.strategy {
        plinth_shared::RankingStrategy::Exponential => ranking.half_life_days,
        plinth_shared::RankingStrategy::Linear => ranking.window_days,
        plinth_shared::RankingStrategy::Pure => 1.0,
    }
}

#[cfg(all(feature = "brick-activity", feature = "ssr"))]
#[allow(dead_code)]
async fn query_activity_list(
    db: &sqlx::PgPool,
    ranking: &plinth_shared::toml_config::RankingConfig,
    limit: Option<i64>,
) -> Result<Vec<plinth_shared::ActivityListItem>, sqlx::Error> {
    use sqlx::Row;

    #[cfg(debug_assertions)]
    if let Ok(delay_ms) = std::env::var("PLINTH_TEST_ACTIVITY_DELAY_MS")
        && let Ok(delay_ms) = delay_ms.parse::<u64>()
    {
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
    }

    let score_expr = activity_score_sql(ranking.strategy, "$1");
    let limit_clause = if limit.is_some() { "LIMIT $2" } else { "" };
    let sql = format!(
        r#"
        SELECT
            id, forge, repo_owner, repo_name, kind, number, url, title,
            state, created_at, closed_at, merged_at, impact,
            labels, featured,
            {score_expr} AS score
        FROM activity_items
        WHERE published = true
        ORDER BY score DESC, {} DESC
        {limit_clause}
        "#,
        activity_ref_date_sql()
    );

    let mut query = sqlx::query(&sql).bind(activity_score_param(ranking));
    if let Some(limit) = limit {
        query = query.bind(limit.max(0));
    }

    let rows = query.fetch_all(db).await?;
    rows.into_iter()
        .map(|row| {
            Ok(plinth_shared::ActivityListItem {
                id: row.try_get("id")?,
                forge: parse_activity_token(row.try_get::<String, _>("forge")?)?,
                repo_owner: row.try_get("repo_owner")?,
                repo_name: row.try_get("repo_name")?,
                kind: parse_activity_token(row.try_get::<String, _>("kind")?)?,
                number: row.try_get("number")?,
                url: row.try_get("url")?,
                title: row.try_get("title")?,
                state: parse_activity_token(row.try_get::<String, _>("state")?)?,
                created_at: row.try_get("created_at")?,
                closed_at: row.try_get("closed_at")?,
                merged_at: row.try_get("merged_at")?,
                impact: row.try_get("impact")?,
                labels: row.try_get("labels")?,
                featured: row.try_get("featured")?,
                score: row.try_get("score")?,
            })
        })
        .collect()
}

#[cfg(all(feature = "brick-activity", feature = "ssr"))]
#[allow(dead_code)]
async fn query_activity_item(
    db: &sqlx::PgPool,
    id: i64,
) -> Result<Option<plinth_shared::ActivityItem>, sqlx::Error> {
    use sqlx::Row;

    let row =
        sqlx::query("SELECT * FROM activity_items WHERE id = $1 AND published = true LIMIT 1")
            .bind(id)
            .fetch_optional(db)
            .await?;

    row.map(|row| {
        Ok(plinth_shared::ActivityItem {
            id: row.try_get("id")?,
            forge: parse_activity_token(row.try_get::<String, _>("forge")?)?,
            repo_owner: row.try_get("repo_owner")?,
            repo_name: row.try_get("repo_name")?,
            kind: parse_activity_token(row.try_get::<String, _>("kind")?)?,
            number: row.try_get("number")?,
            url: row.try_get("url")?,
            title: row.try_get("title")?,
            body: row.try_get("body")?,
            state: parse_activity_token(row.try_get::<String, _>("state")?)?,
            created_at: row.try_get("created_at")?,
            closed_at: row.try_get("closed_at")?,
            merged_at: row.try_get("merged_at")?,
            impact: row.try_get("impact")?,
            additions: row.try_get("additions")?,
            deletions: row.try_get("deletions")?,
            comments_count: row.try_get("comments_count")?,
            labels: row.try_get("labels")?,
            repo_stars: row.try_get("repo_stars")?,
            fetched_at: row.try_get("fetched_at")?,
            featured: row.try_get("featured")?,
            published: row.try_get("published")?,
            content_hash: row.try_get("content_hash")?,
        })
    })
    .transpose()
}

#[cfg(all(feature = "brick-activity", feature = "ssr"))]
#[allow(dead_code)]
fn parse_activity_token<T>(value: String) -> Result<T, sqlx::Error>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value.parse::<T>().map_err(|e| {
        sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        )))
    })
}

// ── Todo server functions ───────────────────────────────────────────────────

#[cfg(feature = "brick-todo")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetTodos, "/api")]
pub async fn get_todos() -> Result<Vec<plinth_shared::TodoListItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_todos(&db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-todo")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_todos() -> Result<Vec<plinth_shared::TodoListItem>, ServerFnError> {
    fetch_json("/api/todos").await
}

#[cfg(feature = "brick-todo")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetTodoBySlug, "/api")]
pub async fn get_todo_by_slug(
    slug: String,
) -> Result<Option<plinth_shared::TodoItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_todo_by_slug(&db, slug)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = slug;
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-todo")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_todo_by_slug(
    slug: String,
) -> Result<Option<plinth_shared::TodoItem>, ServerFnError> {
    fetch_json(&format!("/api/todos/{}", encode_segment(&slug))).await
}

#[cfg(feature = "brick-todo")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetTodosByTag, "/api")]
pub async fn get_todos_by_tag(
    tag: String,
) -> Result<Vec<plinth_shared::TodoListItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        query_todos_by_tag(&db, tag)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = tag;
        unreachable!("server fn body only runs under ssr")
    }
}

#[cfg(feature = "brick-todo")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_todos_by_tag(
    tag: String,
) -> Result<Vec<plinth_shared::TodoListItem>, ServerFnError> {
    fetch_json(&format!("/api/todos/tag/{}", encode_segment(&tag))).await
}

#[cfg(all(feature = "brick-todo", feature = "ssr"))]
#[allow(dead_code)]
fn row_todo_list_item(
    row: sqlx::postgres::PgRow,
) -> Result<plinth_shared::TodoListItem, sqlx::Error> {
    use sqlx::Row;

    Ok(plinth_shared::TodoListItem {
        id: postgres_id("todos", row.try_get("id")?),
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        tags: row.try_get("tags")?,
        completed: row.try_get("completed")?,
        completed_at: row.try_get("completed_at")?,
        created_at: row.try_get("created_at")?,
        order: row.try_get("order")?,
    })
}

#[cfg(all(feature = "brick-todo", feature = "ssr"))]
#[allow(dead_code)]
fn row_todo_item(row: sqlx::postgres::PgRow) -> Result<plinth_shared::TodoItem, sqlx::Error> {
    use sqlx::Row;

    Ok(plinth_shared::TodoItem {
        id: postgres_id("todos", row.try_get("id")?),
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        content: row.try_get("content")?,
        html_content: row.try_get("html_content")?,
        tags: row.try_get("tags")?,
        completed: row.try_get("completed")?,
        completed_at: row.try_get("completed_at")?,
        created_at: row.try_get("created_at")?,
        order: row.try_get("order")?,
    })
}

#[cfg(all(feature = "brick-todo", feature = "ssr"))]
#[allow(dead_code)]
async fn query_todos(db: &sqlx::PgPool) -> Result<Vec<plinth_shared::TodoListItem>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, slug, title, description, tags, completed, completed_at, created_at, "order"
        FROM todos
        ORDER BY completed ASC, "order" ASC, created_at DESC, id DESC
        "#,
    )
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_todo_list_item).collect()
}

#[cfg(all(feature = "brick-todo", feature = "ssr"))]
#[allow(dead_code)]
async fn query_todo_by_slug(
    db: &sqlx::PgPool,
    slug: String,
) -> Result<Option<plinth_shared::TodoItem>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM todos WHERE slug = $1 LIMIT 1")
        .bind(slug)
        .fetch_optional(db)
        .await?;

    row.map(row_todo_item).transpose()
}

#[cfg(all(feature = "brick-todo", feature = "ssr"))]
#[allow(dead_code)]
async fn query_todos_by_tag(
    db: &sqlx::PgPool,
    tag: String,
) -> Result<Vec<plinth_shared::TodoListItem>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT td.id, td.slug, td.title, td.description, td.tags, td.completed,
               td.completed_at, td.created_at, td."order"
        FROM todos td
        JOIN todo_tags tt ON tt.todo_id = td.id
        JOIN tags t ON t.id = tt.tag_id
        WHERE t.name = $1 OR t.slug = $1
        ORDER BY td.completed ASC, td."order" ASC, td.created_at DESC, td.id DESC
        "#,
    )
    .bind(tag)
    .fetch_all(db)
    .await?;

    rows.into_iter().map(row_todo_list_item).collect()
}
