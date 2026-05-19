//! Declarative content loader for Nix-managed blog articles.

use std::collections::HashMap;
use std::path::Path;

use pgvector::Vector;
use plinth_shared::{BlogPost, ContentFormat};
use serde::Deserialize;
use sqlx::Row;
use tracing::{info, warn};

use crate::PlinthDb;
use crate::config::PlinthConfig;
use crate::services::db::{create_tags_for_post_tx, sync_post_tags_cache_tx};
use crate::services::markdown_processor::parse_markdown;

/// A single entry in the declarative content manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestEntry {
    pub slug: String,
    pub filename: String,
    /// Pre-compiled HTML filename for Typst articles compiled at Nix build time.
    pub html_filename: Option<String>,
    /// `markdown` or `typst`.
    pub format: String,
    pub published: bool,
    /// SHA-256 hash of the source file content.
    pub content_hash: String,
}

/// Statistics from a declarative content load operation.
#[derive(Debug, Default)]
pub struct LoadStats {
    pub inserted: usize,
    pub updated: usize,
    pub deleted: usize,
    pub skipped: usize,
}

struct ExistingArticle {
    source: String,
    content_hash: Option<String>,
}

/// Load declarative articles from a content directory into the database.
pub async fn load_declarative_articles(
    db: &PlinthDb,
    content_dir: &str,
    config: &PlinthConfig,
) -> Result<LoadStats, Box<dyn std::error::Error>> {
    let content_path = Path::new(content_dir);
    let manifest_path = content_path.join("manifest.json");

    if !manifest_path.exists() {
        return Err(format!(
            "Declarative content manifest not found: {}",
            manifest_path.display()
        )
        .into());
    }

    let manifest_content = std::fs::read_to_string(&manifest_path)?;
    let manifest: HashMap<String, ManifestEntry> = serde_json::from_str(&manifest_content)?;

    if manifest.is_empty() {
        info!("Declarative content manifest is empty, nothing to load");
        return Ok(LoadStats::default());
    }

    let articles_dir = content_path.join("articles");
    let mut stats = LoadStats::default();
    let active_slugs: Vec<String> = manifest.values().map(|e| e.slug.clone()).collect();

    for entry in manifest.values() {
        if let Some(existing) = existing_article(db, &entry.slug).await?
            && existing.source != "declarative"
        {
            return Err(format!(
                "Slug conflict: declarative article '{}' collides with an existing API-published article. Either rename the declarative slug or delete the API-published article via the admin API.",
                entry.slug
            )
            .into());
        }
    }

    for entry in manifest.values() {
        let existing = existing_article(db, &entry.slug).await?;
        if let Some(ref existing) = existing
            && existing.content_hash.as_deref() == Some(&entry.content_hash)
        {
            stats.skipped += 1;
            continue;
        }

        let source_path = articles_dir.join(&entry.filename);
        let source_content = std::fs::read_to_string(&source_path).map_err(|e| {
            format!(
                "Failed to read article source '{}': {}",
                source_path.display(),
                e
            )
        })?;

        let content_format = match entry.format.as_str() {
            "typst" => ContentFormat::Typst,
            _ => ContentFormat::Markdown,
        };

        let (html_content, raw_content, frontmatter, reading_time) = match content_format {
            ContentFormat::Markdown => {
                let parsed = parse_markdown(&source_content)
                    .map_err(|e| format!("Failed to parse markdown for '{}': {}", entry.slug, e))?;
                (
                    parsed.html_content,
                    parsed.markdown_content,
                    parsed.frontmatter,
                    parsed.reading_time_minutes,
                )
            }
            ContentFormat::Typst => {
                let html_file = entry.html_filename.as_ref().ok_or_else(|| {
                    format!(
                        "Typst article '{}' requires pre-compiled HTML (html_filename in manifest)",
                        entry.slug
                    )
                })?;
                let html_path = articles_dir.join(html_file);
                let html = std::fs::read_to_string(&html_path).map_err(|e| {
                    format!(
                        "Failed to read pre-compiled HTML for '{}': {}",
                        entry.slug, e
                    )
                })?;
                let reading_time = BlogPost::calculate_reading_time(&source_content);
                (html, source_content.clone(), None, reading_time)
            }
        };

        let title = frontmatter
            .as_ref()
            .and_then(|fm| fm.title.clone())
            .unwrap_or_else(|| plinth_shared::humanize_slug(&entry.slug));
        let tags = frontmatter
            .as_ref()
            .and_then(|fm| fm.tags.clone())
            .unwrap_or_default();
        let author = frontmatter
            .as_ref()
            .and_then(|fm| fm.author.clone())
            .unwrap_or_else(|| config.site.author.name.clone());
        let description = frontmatter
            .as_ref()
            .and_then(|fm| fm.description.clone())
            .unwrap_or_default();
        let published = frontmatter
            .as_ref()
            .and_then(|fm| fm.published)
            .unwrap_or(entry.published);
        let featured = frontmatter
            .as_ref()
            .and_then(|fm| fm.featured)
            .unwrap_or(false);
        let series_slug = frontmatter.as_ref().and_then(|fm| fm.series.clone());

        let (series_slug, series_title, series_position) = if let Some(ref s_slug) = series_slug {
            let s_title = frontmatter
                .as_ref()
                .and_then(|fm| fm.series_title.clone())
                .unwrap_or_else(|| plinth_shared::humanize_slug(s_slug));
            let s_position = match frontmatter.as_ref().and_then(|fm| fm.series_position) {
                Some(pos) => pos,
                None => {
                    let max_pos: i32 = sqlx::query_scalar(
                        "SELECT COALESCE(MAX(series_position), 0)::integer FROM blog_posts WHERE series_slug = $1",
                    )
                    .bind(s_slug)
                    .fetch_one(db)
                    .await?;
                    (max_pos.max(0) as u32) + 1
                }
            };
            (Some(s_slug.clone()), Some(s_title), Some(s_position))
        } else {
            (None, None, None)
        };

        let mut tx = db.begin().await?;

        if existing.is_some() {
            sqlx::query(
                r#"
                UPDATE blog_posts
                SET title = $1,
                    content = $2,
                    html_content = $3,
                    description = $4,
                    author = $5,
                    tags = $6,
                    published = $7,
                    featured = $8,
                    reading_time_minutes = $9,
                    content_format = $10,
                    content_hash = $11,
                    series_slug = $12,
                    series_title = $13,
                    series_position = $14,
                    updated_at = now()
                WHERE slug = $15
                "#,
            )
            .bind(&title)
            .bind(&raw_content)
            .bind(&html_content)
            .bind(&description)
            .bind(&author)
            .bind(&tags)
            .bind(published)
            .bind(featured)
            .bind(reading_time as i32)
            .bind(content_format.as_str())
            .bind(&entry.content_hash)
            .bind(&series_slug)
            .bind(&series_title)
            .bind(series_position.map(|p| p as i32))
            .bind(&entry.slug)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                r#"
                DELETE FROM blog_post_tags bpt
                USING blog_posts bp
                WHERE bpt.post_id = bp.id AND bp.slug = $1
                "#,
            )
            .bind(&entry.slug)
            .execute(&mut *tx)
            .await?;

            create_tags_for_post_tx(&mut tx, &entry.slug, &tags).await?;
            sync_post_tags_cache_tx(&mut tx, &entry.slug).await?;
            tx.commit().await?;

            info!(slug = %entry.slug, "Updated declarative article");
            stats.updated += 1;
        } else {
            sqlx::query(
                r#"
                INSERT INTO blog_posts (
                    slug,
                    title,
                    content,
                    html_content,
                    description,
                    author,
                    tags,
                    published,
                    featured,
                    reading_time_minutes,
                    content_format,
                    content_hash,
                    source,
                    published_at,
                    embedding,
                    series_slug,
                    series_title,
                    series_position
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'declarative', now(), NULL, $13, $14, $15)
                "#,
            )
            .bind(&entry.slug)
            .bind(&title)
            .bind(&raw_content)
            .bind(&html_content)
            .bind(&description)
            .bind(&author)
            .bind(&tags)
            .bind(published)
            .bind(featured)
            .bind(reading_time as i32)
            .bind(content_format.as_str())
            .bind(&entry.content_hash)
            .bind(&series_slug)
            .bind(&series_title)
            .bind(series_position.map(|p| p as i32))
            .execute(&mut *tx)
            .await?;

            create_tags_for_post_tx(&mut tx, &entry.slug, &tags).await?;
            sync_post_tags_cache_tx(&mut tx, &entry.slug).await?;
            tx.commit().await?;

            info!(slug = %entry.slug, "Inserted declarative article");
            stats.inserted += 1;
        }
    }

    let stale_slugs: Vec<String> = sqlx::query_scalar(
        "SELECT slug FROM blog_posts WHERE source = 'declarative' AND NOT (slug = ANY($1))",
    )
    .bind(&active_slugs)
    .fetch_all(db)
    .await?;

    for slug in stale_slugs {
        sqlx::query("DELETE FROM blog_posts WHERE slug = $1")
            .bind(&slug)
            .execute(db)
            .await?;
        info!(slug = %slug, "Deleted stale declarative article");
        stats.deleted += 1;
    }

    Ok(stats)
}

/// Backfill embeddings for declarative articles.
pub async fn backfill_embeddings(
    db: PlinthDb,
    vector_search: kameo::actor::ActorRef<crate::actors::vector_search::VectorSearch>,
    vector_truncation: usize,
) {
    use crate::actors::vector_search::GenerateEmbedding;

    let rows = match sqlx::query(
        "SELECT slug, content FROM blog_posts WHERE source = 'declarative' AND embedding IS NULL",
    )
    .fetch_all(&db)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            warn!("Failed to query articles for embedding backfill: {}", e);
            return;
        }
    };

    if rows.is_empty() {
        return;
    }

    info!(
        count = rows.len(),
        "Backfilling embeddings for declarative articles"
    );

    for row in rows {
        let slug = match row.try_get::<String, _>("slug") {
            Ok(slug) => slug,
            Err(e) => {
                warn!("Missing slug in embedding backfill row: {}", e);
                continue;
            }
        };
        let content = match row.try_get::<String, _>("content") {
            Ok(content) => content,
            Err(e) => {
                warn!(slug = %slug, "Missing content in embedding backfill row: {}", e);
                continue;
            }
        };

        let text = if content.len() > vector_truncation {
            &content[..vector_truncation]
        } else {
            content.as_str()
        };

        match vector_search
            .ask(GenerateEmbedding {
                text: text.to_string(),
            })
            .await
        {
            Ok(embedding) => {
                if let Err(e) = sqlx::query("UPDATE blog_posts SET embedding = $1 WHERE slug = $2")
                    .bind(Vector::from(embedding))
                    .bind(&slug)
                    .execute(&db)
                    .await
                {
                    warn!(slug = %slug, "Failed to store backfilled embedding: {}", e);
                }
            }
            Err(e) => {
                warn!(slug = %slug, "VectorSearch actor error during backfill: {}", e);
            }
        }
    }

    info!("Embedding backfill complete");
}

async fn existing_article(
    db: &PlinthDb,
    slug: &str,
) -> Result<Option<ExistingArticle>, sqlx::Error> {
    sqlx::query("SELECT source, content_hash FROM blog_posts WHERE slug = $1 LIMIT 1")
        .bind(slug)
        .fetch_optional(db)
        .await?
        .map(|row| {
            Ok(ExistingArticle {
                source: row.try_get("source")?,
                content_hash: row.try_get("content_hash")?,
            })
        })
        .transpose()
}
