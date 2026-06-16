use leptos::prelude::*;

use super::common;

// ── Row parser ───────────────────────────────────────────────────────────────

#[cfg(all(feature = "brick-portfolio", feature = "ssr"))]
#[allow(dead_code)]
fn row_portfolio_item(
    row: sqlx::postgres::PgRow,
) -> Result<plinth_shared::PortfolioItem, sqlx::Error> {
    use sqlx::Row;

    Ok(plinth_shared::PortfolioItem {
        id: common::postgres_id("portfolio_items", row.try_get("id")?),
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        description: row.try_get("description")?,
        content: row.try_get("content")?,
        html_content: row.try_get("html_content")?,
        tech_stack: row.try_get("tech_stack")?,
        link: row.try_get("link")?,
        demo: row.try_get("demo")?,
        project_url: row.try_get("project_url")?,
        links: row
            .try_get::<sqlx::types::Json<Vec<plinth_shared::ExternalLink>>, _>("links")?
            .0,
        image_url: row.try_get("image_url")?,
        date: row.try_get("date")?,
        featured: row.try_get("featured")?,
        order: row.try_get("order")?,
    })
}

// ── Query functions ──────────────────────────────────────────────────────────

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

// ── Server functions (SSR) + CSR alternatives ────────────────────────────────

/// Fetch all portfolio items ordered by priority and date.
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

/// CSR fallback — fetches portfolio items from `GET /api/portfolio`.
#[cfg(feature = "brick-portfolio")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_portfolio_items() -> Result<Vec<plinth_shared::PortfolioItem>, ServerFnError> {
    common::fetch_json("/api/portfolio").await
}

/// Fetch a single portfolio item by its URL slug.
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

/// CSR fallback — fetches a portfolio item from `GET /api/portfolio/{slug}`.
#[cfg(feature = "brick-portfolio")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_portfolio_item_by_slug(
    slug: String,
) -> Result<Option<plinth_shared::PortfolioItem>, ServerFnError> {
    common::fetch_json(&format!("/api/portfolio/{}", common::encode_segment(&slug))).await
}
