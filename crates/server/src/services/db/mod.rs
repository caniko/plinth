use std::time::Duration;

use pgvector::Vector;
#[cfg(feature = "brick-portfolio")]
use plinth_shared::PortfolioItem;
use sqlx::postgres::PgPoolOptions;
use tracing::{info, instrument};

pub type Db = sqlx::PgPool;

use crate::PlinthDb;

mod seed;
mod tags;
#[cfg(test)]
mod tests;

pub use seed::seed_sample_data;
pub use tags::create_tags_for_post;
pub use tags::sync_post_tags_cache;
pub use tags::sync_todo_tags_cache;

pub(crate) use tags::{
    create_tags_for_post_tx, create_tags_for_todo_tx, sync_post_tags_cache_tx,
    sync_todo_tags_cache_tx,
};

fn redact_db_url(url: &str) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return "<redacted>".to_string();
    };
    let host_and_path = rest.rsplit_once('@').map_or(rest, |(_, after)| after);
    let host_and_path = host_and_path.split('?').next().unwrap_or(host_and_path);
    format!("{scheme}://{host_and_path}")
}

#[instrument(skip(config))]
pub async fn init_db(config: &crate::config::DatabaseConfig) -> Result<PlinthDb, sqlx::Error> {
    info!(database = %redact_db_url(&config.database_url), "Connecting to Postgres");

    let pool = PgPoolOptions::new()
        .max_connections(16)
        .acquire_timeout(Duration::from_secs(10))
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET search_path TO public")
                    .execute(&mut *conn)
                    .await?;
                sqlx::query("SELECT 1").execute(&mut *conn).await?;
                let _ = pgvector::Vector::from(vec![0.0_f32]);
                Ok(())
            })
        })
        .connect(&config.database_url)
        .await?;

    info!("Postgres connected");
    Ok(pool)
}

#[instrument(skip(db))]
pub async fn init_schema(db: &PlinthDb) -> Result<(), sqlx::Error> {
    crate::services::migrations::run_migrations(db).await?;
    Ok(())
}

pub(crate) fn vector_or_none(embedding: Option<Vec<f32>>) -> Option<Vector> {
    embedding.map(Vector::from)
}

#[cfg(feature = "brick-activity")]
pub async fn upsert_activity_item(
    db: &PlinthDb,
    request: &plinth_shared::PublishActivityRequest,
    fetched_at: chrono::DateTime<chrono::Utc>,
) -> Result<i64, sqlx::Error> {
    let embedding = vector_or_none(request.embedding.clone());
    sqlx::query_scalar(
        r#"
        INSERT INTO activity_items (
            forge, repo_owner, repo_name, kind, number, url, title, body, state,
            created_at, closed_at, merged_at, impact, additions, deletions,
            comments_count, labels, repo_stars, embedding, fetched_at,
            featured, published, content_hash
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23)
        ON CONFLICT (forge, repo_owner, repo_name, kind, number) DO UPDATE SET
            url = EXCLUDED.url,
            title = EXCLUDED.title,
            body = EXCLUDED.body,
            state = EXCLUDED.state,
            created_at = EXCLUDED.created_at,
            closed_at = EXCLUDED.closed_at,
            merged_at = EXCLUDED.merged_at,
            impact = EXCLUDED.impact,
            additions = EXCLUDED.additions,
            deletions = EXCLUDED.deletions,
            comments_count = EXCLUDED.comments_count,
            labels = EXCLUDED.labels,
            repo_stars = EXCLUDED.repo_stars,
            embedding = COALESCE(EXCLUDED.embedding, activity_items.embedding),
            fetched_at = EXCLUDED.fetched_at,
            featured = EXCLUDED.featured,
            published = EXCLUDED.published,
            content_hash = EXCLUDED.content_hash
        RETURNING id
        "#,
    )
    .bind(request.forge.as_str())
    .bind(&request.repo_owner)
    .bind(&request.repo_name)
    .bind(request.kind.as_str())
    .bind(request.number)
    .bind(&request.url)
    .bind(&request.title)
    .bind(&request.body)
    .bind(request.state.as_str())
    .bind(request.created_at)
    .bind(request.closed_at)
    .bind(request.merged_at)
    .bind(request.impact)
    .bind(request.additions)
    .bind(request.deletions)
    .bind(request.comments_count)
    .bind(&request.labels)
    .bind(request.repo_stars)
    .bind(embedding)
    .bind(fetched_at)
    .bind(request.featured)
    .bind(request.published)
    .bind(&request.content_hash)
    .fetch_one(db)
    .await
}

#[cfg(feature = "brick-activity")]
pub async fn delete_activity_item(db: &PlinthDb, id: i64) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("DELETE FROM activity_items WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(res.rows_affected())
}

#[cfg(feature = "brick-activity")]
pub async fn patch_activity_item(
    db: &PlinthDb,
    id: i64,
    impact: Option<i16>,
    featured: Option<bool>,
    published: Option<bool>,
) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(
        r#"
        UPDATE activity_items SET
            impact = COALESCE($2, impact),
            featured = COALESCE($3, featured),
            published = COALESCE($4, published)
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(impact)
    .bind(featured)
    .bind(published)
    .execute(db)
    .await?;
    Ok(res.rows_affected() > 0)
}

#[cfg(feature = "brick-portfolio")]
pub async fn upsert_portfolio_item(
    db: &PlinthDb,
    item: &PortfolioItem,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        r##"
        INSERT INTO portfolio_items (
            slug, title, description, content, html_content, tech_stack,
            link, demo, project_url, links, image_url, date, featured, "order"
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        ON CONFLICT (slug) DO UPDATE SET
            title = EXCLUDED.title,
            description = EXCLUDED.description,
            content = EXCLUDED.content,
            html_content = EXCLUDED.html_content,
            tech_stack = EXCLUDED.tech_stack,
            link = EXCLUDED.link,
            demo = EXCLUDED.demo,
            project_url = EXCLUDED.project_url,
            links = EXCLUDED.links,
            image_url = EXCLUDED.image_url,
            date = EXCLUDED.date,
            featured = EXCLUDED.featured,
            "order" = EXCLUDED."order"
        RETURNING id
        "##,
    )
    .bind(&item.slug)
    .bind(&item.title)
    .bind(&item.description)
    .bind(&item.content)
    .bind(&item.html_content)
    .bind(&item.tech_stack)
    .bind(&item.link)
    .bind(&item.demo)
    .bind(&item.project_url)
    .bind(sqlx::types::Json(&item.links))
    .bind(&item.image_url)
    .bind(item.date)
    .bind(item.featured)
    .bind(item.order)
    .fetch_one(db)
    .await
}
