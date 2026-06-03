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

// ── Activity server functions ───────────────────────────────────────────────

#[cfg(feature = "brick-activity")]
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
