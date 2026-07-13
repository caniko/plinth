use plinth_shared::{ActivityItem, ActivityListItem, RankingStrategy, toml_config::RankingConfig};

use crate::{PlinthDb, services::rows};

/// Reference date expression used for ranking and tiebreaking.
pub const REF_DATE_SQL: &str = "coalesce(merged_at, closed_at, created_at)";

/// Age in days from the reference date to now.
pub fn age_days_sql() -> String {
    format!("(extract(epoch from (now() - {REF_DATE_SQL})) / 86400.0)")
}

/// Score expression for a strategy. The placeholder is the bound ranking param.
pub fn score_sql(strategy: RankingStrategy, param_placeholder: &str) -> String {
    let age = age_days_sql();
    match strategy {
        RankingStrategy::Exponential => {
            format!(
                "(impact::float8 * power(0.5, {age} / greatest({param_placeholder}, 0.000001)))"
            )
        }
        RankingStrategy::Linear => {
            format!(
                "(impact::float8 * greatest(0.0, 1.0 - {age} / greatest({param_placeholder}, 0.000001)))"
            )
        }
        RankingStrategy::Pure => "(impact::float8)".to_string(),
    }
}

/// Numeric ranking parameter to bind for the selected strategy.
pub fn score_param(strategy: RankingStrategy, half_life_days: f64, window_days: f64) -> f64 {
    match strategy {
        RankingStrategy::Exponential => half_life_days,
        RankingStrategy::Linear => window_days,
        RankingStrategy::Pure => 1.0,
    }
}

/// Canonical ranked-list read shared by the cache actor and later refresh work.
pub async fn query_ranked_list(
    db: &PlinthDb,
    ranking: &RankingConfig,
    featured_only: bool,
    limit: Option<i64>,
) -> Result<Vec<ActivityListItem>, sqlx::Error> {
    let strategy = ranking.strategy;
    let score_expr = score_sql(strategy, "$1");
    let ref_date = REF_DATE_SQL;
    let where_featured = if featured_only {
        "AND featured = true"
    } else {
        ""
    };
    let limit_clause = if limit.is_some() { "LIMIT $2" } else { "" };
    let sql = format!(
        r#"
        SELECT
            id, forge, repo_owner, repo_name, kind, number, url, title,
            state, created_at, closed_at, merged_at, impact,
            labels, featured,
            {score_expr} AS score
        FROM activity_items
        WHERE published = true {where_featured}
        ORDER BY score DESC, {ref_date} DESC
        {limit_clause}
        "#
    );

    let param = score_param(strategy, ranking.half_life_days, ranking.window_days);
    let mut q = sqlx::query(sqlx::AssertSqlSafe(&*sql)).bind(param);
    if let Some(n) = limit {
        q = q.bind(n.max(0));
    }
    let rows = q.fetch_all(db).await?;
    rows.into_iter().map(rows::activity_list_item).collect()
}

/// Read one published activity item directly from PostgreSQL.
///
/// The Dioxus SSR path intentionally uses this uncached read so a successful
/// admin write is visible on the next request.  The activity actor remains
/// responsible for stale-while-revalidate forge refreshes and is poked by the
/// caller after this query completes.
pub async fn query_item(db: &PlinthDb, id: i64) -> Result<Option<ActivityItem>, sqlx::Error> {
    let row =
        sqlx::query("SELECT * FROM activity_items WHERE id = $1 AND published = true LIMIT 1")
            .bind(id)
            .fetch_optional(db)
            .await?;

    row.map(rows::activity_item).transpose()
}
