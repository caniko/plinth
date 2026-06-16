use leptos::prelude::*;

use super::common;

// ── SSR internal helpers ────────────────────────────────────────────────────

#[cfg(all(feature = "brick-blog", feature = "ssr"))]
#[allow(dead_code)]
fn content_format(value: String) -> Result<plinth_shared::ContentFormat, sqlx::Error> {
    match value.as_str() {
        "markdown" => Ok(plinth_shared::ContentFormat::Markdown),
        "typst" => Ok(plinth_shared::ContentFormat::Typst),
        other => Err(common::decode_error(format!(
            "unknown content format '{other}'"
        ))),
    }
}

// ── Row parsers ──────────────────────────────────────────────────────────────

#[cfg(all(feature = "brick-blog", feature = "ssr"))]
#[allow(dead_code)]
fn row_blog_list_item(
    row: sqlx::postgres::PgRow,
) -> Result<plinth_shared::BlogListItem, sqlx::Error> {
    use sqlx::Row;

    Ok(plinth_shared::BlogListItem {
        id: common::postgres_id("blog_posts", row.try_get("id")?),
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        published_at: row.try_get("published_at")?,
        author: row.try_get("author")?,
        tags: row.try_get("tags")?,
        featured: row.try_get("featured")?,
        reading_time_minutes: common::as_u32(
            row.try_get("reading_time_minutes")?,
            "reading_time_minutes",
        )?,
        series_slug: row.try_get("series_slug")?,
        series_title: row.try_get("series_title")?,
        series_position: row
            .try_get::<Option<i32>, _>("series_position")?
            .map(|v| common::as_u32(v, "series_position"))
            .transpose()?,
    })
}

#[cfg(all(feature = "brick-blog", feature = "ssr"))]
#[allow(dead_code)]
fn row_blog_post(row: sqlx::postgres::PgRow) -> Result<plinth_shared::BlogPost, sqlx::Error> {
    use sqlx::Row;

    Ok(plinth_shared::BlogPost {
        id: common::postgres_id("blog_posts", row.try_get("id")?),
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
        reading_time_minutes: common::as_u32(
            row.try_get("reading_time_minutes")?,
            "reading_time_minutes",
        )?,
        embedding: None,
        content_format: content_format(row.try_get("content_format")?)?,
        source: row.try_get("source")?,
        content_hash: row.try_get("content_hash")?,
        series_slug: row.try_get("series_slug")?,
        series_title: row.try_get("series_title")?,
        series_position: row
            .try_get::<Option<i32>, _>("series_position")?
            .map(|v| common::as_u32(v, "series_position"))
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
        position: common::as_u32(row.try_get("position")?, "position")?,
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
        post_count: common::as_u32(row.try_get("post_count")?, "post_count")?,
        total_reading_time: common::as_u32(
            row.try_get("total_reading_time")?,
            "total_reading_time",
        )?,
        latest_published_at: row.try_get("latest_published_at")?,
    })
}

// ── Query functions ──────────────────────────────────────────────────────────

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
        .map(|v| common::as_u32(v, "series_position"))
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

// ── Server functions (SSR) + CSR alternatives ────────────────────────────────

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
    common::fetch_json("/api/posts").await
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
    common::fetch_json(&format!("/api/posts/{}", common::encode_segment(&slug))).await
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
    common::fetch_json(&format!("/api/posts/tag/{}", common::encode_segment(&tag))).await
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
    common::fetch_json(&format!(
        "/api/posts/{}/series-nav",
        common::encode_segment(&post_slug)
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
    common::fetch_json(&format!(
        "/api/series/{}/posts",
        common::encode_segment(&series_slug)
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
    common::fetch_json("/api/series").await
}
