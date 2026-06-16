use leptos::prelude::*;

// ── Activity helpers (SSR) ───────────────────────────────────────────────────

/// A page visit drives freshness: ask the activity cache actor to consider a
/// stale-while-revalidate forge refresh. Fire-and-forget — never blocks the
/// render, and is a no-op when the hook is absent (e.g. a build without the
/// server-installed context) or the data is still fresh. The hook is a
/// type-erased `ActivityRefreshHook` the server provides into the SSR context, so
/// this crate stays independent of `plinth-server`.
#[cfg(all(feature = "brick-activity", feature = "ssr"))]
#[allow(dead_code)] // used by the SSR server-fns; some feature combos elide the call sites
fn poke_activity_refresh() {
    if let Some(hook) = use_context::<std::sync::Arc<dyn plinth_shared::ActivityRefreshHook>>() {
        hook.poke();
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

// ── Row helpers ──────────────────────────────────────────────────────────────

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

// ── Query functions ──────────────────────────────────────────────────────────

#[cfg(all(feature = "brick-activity", feature = "ssr"))]
#[allow(dead_code)]
async fn query_activity_list(
    db: &sqlx::PgPool,
    ranking: &plinth_shared::toml_config::RankingConfig,
    limit: Option<i64>,
) -> Result<Vec<plinth_shared::ActivityListItem>, sqlx::Error> {
    use sqlx::Row;

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

// ── Server functions (SSR) + CSR alternatives ────────────────────────────────

/// Fetch ranked activity list (up to 50 items), triggering a
/// stale-while-revalidate forge refresh.
#[cfg(feature = "brick-activity")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetActivityList, "/api")]
pub async fn get_activity_list() -> Result<Vec<plinth_shared::ActivityListItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        let config = plinth_shared::toml_config::PlinthConfig::load()
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        let items = query_activity_list(&db, &config.ranking, Some(50))
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        poke_activity_refresh();
        Ok(items)
    }
    #[cfg(not(feature = "ssr"))]
    {
        unreachable!("server fn body only runs under ssr")
    }
}

/// CSR fallback — fetches activity list from `GET /api/activity?limit=50`.
#[cfg(feature = "brick-activity")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_activity_list() -> Result<Vec<plinth_shared::ActivityListItem>, ServerFnError> {
    super::common::fetch_json("/api/activity?limit=50").await
}

/// Fetch a single activity item by its numeric ID.
#[cfg(feature = "brick-activity")]
#[cfg(any(not(feature = "csr"), feature = "ssr"))]
#[server(GetActivityItemById, "/api")]
pub async fn get_activity_item_by_id(
    id: i64,
) -> Result<Option<plinth_shared::ActivityItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = expect_context::<sqlx::PgPool>();
        let item = query_activity_item(&db, id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        poke_activity_refresh();
        Ok(item)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = id;
        unreachable!("server fn body only runs under ssr")
    }
}

/// CSR fallback — fetches an activity item from `GET /api/activity/{id}`.
#[cfg(feature = "brick-activity")]
#[cfg(all(feature = "csr", not(feature = "ssr")))]
pub async fn get_activity_item_by_id(
    id: i64,
) -> Result<Option<plinth_shared::ActivityItem>, ServerFnError> {
    super::common::fetch_json(&format!("/api/activity/{id}")).await
}
