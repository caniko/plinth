//! Declarative content loader for Nix-managed blog articles.
//!
//! Reads a content directory (built by the NixOS module) containing a
//! `manifest.json` and article source files. Upserts articles into SurrealDB
//! at server startup, keyed by slug, with change detection via content hashes.

use std::collections::HashMap;
use std::path::Path;

use plinth_shared::{BlogPost, ContentFormat};
use serde::Deserialize;
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use tracing::{info, warn};

use crate::config::PlinthConfig;
use crate::db_helpers::take_as_opt;
use crate::services::db::create_tags_for_post;
use crate::services::markdown_processor::parse_markdown;

/// A single entry in the declarative content manifest.
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestEntry {
    pub slug: String,
    pub filename: String,
    /// Pre-compiled HTML filename (for Typst articles compiled at Nix build time)
    pub html_filename: Option<String>,
    /// "markdown" or "typst"
    pub format: String,
    pub published: bool,
    /// SHA-256 hash of the source file content
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

/// Row returned when checking existing articles in the database.
#[derive(Debug, Deserialize)]
struct ExistingArticle {
    #[serde(default = "default_api_source")]
    source: String,
    content_hash: Option<String>,
}

fn default_api_source() -> String {
    "api".to_string()
}

/// Load declarative articles from a content directory into SurrealDB.
///
/// The content directory must contain a `manifest.json` and an `articles/`
/// subdirectory with the source files referenced in the manifest.
///
/// # Errors
///
/// Returns an error if:
/// - The manifest cannot be read or parsed
/// - A declarative article's slug conflicts with an API-published article
/// - Database operations fail
pub async fn load_declarative_articles(
    db: &Surreal<Db>,
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

    // Phase 1: Check for conflicts with API-published articles
    for entry in manifest.values() {
        let mut result = db
            .query("SELECT source, content_hash FROM blog_posts WHERE slug = $slug LIMIT 1")
            .bind(("slug", entry.slug.clone()))
            .await?;

        let existing: Option<ExistingArticle> =
            take_as_opt(&mut result, 0).map_err(Box::<dyn std::error::Error>::from)?;

        if let Some(ref existing) = existing
            && existing.source != "declarative"
        {
            return Err(format!(
                "Slug conflict: declarative article '{}' collides with an existing \
                 API-published article. Either rename the declarative slug or delete \
                 the API-published article via the admin API.",
                entry.slug
            )
            .into());
        }
    }

    // Phase 2: Upsert articles
    for entry in manifest.values() {
        // Re-query for existing state (already validated no API conflicts above)
        let mut result = db
            .query("SELECT source, content_hash FROM blog_posts WHERE slug = $slug LIMIT 1")
            .bind(("slug", entry.slug.clone()))
            .await?;

        let existing: Option<ExistingArticle> =
            take_as_opt(&mut result, 0).map_err(Box::<dyn std::error::Error>::from)?;

        // Skip if content hasn't changed
        if let Some(ref existing) = existing
            && existing.content_hash.as_deref() == Some(&entry.content_hash)
        {
            stats.skipped += 1;
            continue;
        }

        // Read source file
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

        // Process content based on format
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
                // Read pre-compiled HTML from the content directory
                let html = if let Some(ref html_file) = entry.html_filename {
                    let html_path = articles_dir.join(html_file);
                    std::fs::read_to_string(&html_path).map_err(|e| {
                        format!(
                            "Failed to read pre-compiled HTML for '{}': {}",
                            entry.slug, e
                        )
                    })?
                } else {
                    return Err(format!(
                        "Typst article '{}' requires pre-compiled HTML (html_filename in manifest)",
                        entry.slug
                    )
                    .into());
                };
                let reading_time = BlogPost::calculate_reading_time(&source_content);
                (html, source_content.clone(), None, reading_time)
            }
        };

        // Extract metadata from frontmatter
        let title = frontmatter
            .as_ref()
            .and_then(|fm| fm.title.clone())
            .unwrap_or_else(|| plinth_shared::humanize_slug(&entry.slug));

        let slug = &entry.slug;

        let tags = frontmatter
            .as_ref()
            .and_then(|fm| fm.tags.clone())
            .unwrap_or_default();

        let default_author = config.site.author.name.clone();
        let author = frontmatter
            .as_ref()
            .and_then(|fm| fm.author.clone())
            .unwrap_or(default_author);

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

        // Series fields
        let series_slug = frontmatter.as_ref().and_then(|fm| fm.series.clone());

        let (series_slug, series_title, series_position) = if let Some(ref s_slug) = series_slug {
            let s_title = frontmatter
                .as_ref()
                .and_then(|fm| fm.series_title.clone())
                .unwrap_or_else(|| plinth_shared::humanize_slug(s_slug));

            let s_position = frontmatter.as_ref().and_then(|fm| fm.series_position);

            // Auto-assign position if not provided
            let s_position = match s_position {
                Some(pos) => pos,
                None => {
                    let mut result = db
                        .query("SELECT VALUE math::max(series_position) FROM blog_posts WHERE series_slug = $slug")
                        .bind(("slug", s_slug.to_string()))
                        .await?;
                    let max_pos: Option<u32> = result.take(0).unwrap_or(None);
                    max_pos.unwrap_or(0) + 1
                }
            };

            (Some(s_slug.clone()), Some(s_title), Some(s_position))
        } else {
            (None, None, None)
        };

        let format_str = match content_format {
            ContentFormat::Markdown => "markdown",
            ContentFormat::Typst => "typst",
        };

        if existing.is_some() {
            // UPDATE existing declarative article (preserve published_at)
            db.query(
                r#"
                UPDATE blog_posts SET
                    title = $title,
                    content = $content,
                    html_content = $html_content,
                    description = $description,
                    author = $author,
                    tags = $tags,
                    published = $published,
                    featured = $featured,
                    reading_time_minutes = $reading_time,
                    content_format = $format,
                    content_hash = $content_hash,
                    series_slug = $series_slug,
                    series_title = $series_title,
                    series_position = $series_position,
                    updated_at = time::now()
                WHERE slug = $slug
                "#,
            )
            .bind(("slug", slug.to_string()))
            .bind(("title", title.clone()))
            .bind(("content", raw_content))
            .bind(("html_content", html_content))
            .bind(("description", description))
            .bind(("author", author))
            .bind(("tags", tags.clone()))
            .bind(("published", published))
            .bind(("featured", featured))
            .bind(("reading_time", reading_time as i64))
            .bind(("format", format_str.to_string()))
            .bind(("content_hash", entry.content_hash.clone()))
            .bind(("series_slug", series_slug))
            .bind(("series_title", series_title))
            .bind(("series_position", series_position.map(|p| p as i64)))
            .await?;

            // Re-create tag relations: delete old, create new
            db.query(
                r#"
                LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = $slug LIMIT 1)[0];
                DELETE tagged WHERE in = $post;
                "#,
            )
            .bind(("slug", slug.to_string()))
            .await?;

            create_tags_for_post(db, slug, &tags).await?;

            info!(slug = %slug, "Updated declarative article");
            stats.updated += 1;
        } else {
            // INSERT new declarative article
            db.query(
                r#"
                CREATE blog_posts CONTENT {
                    slug: $slug,
                    title: $title,
                    content: $content,
                    html_content: $html_content,
                    description: $description,
                    author: $author,
                    tags: $tags,
                    published: $published,
                    featured: $featured,
                    reading_time_minutes: $reading_time,
                    content_format: $format,
                    content_hash: $content_hash,
                    source: "declarative",
                    published_at: time::now(),
                    embedding: NONE,
                    series_slug: $series_slug,
                    series_title: $series_title,
                    series_position: $series_position
                }
                "#,
            )
            .bind(("slug", slug.to_string()))
            .bind(("title", title.clone()))
            .bind(("content", raw_content))
            .bind(("html_content", html_content))
            .bind(("description", description))
            .bind(("author", author))
            .bind(("tags", tags.clone()))
            .bind(("published", published))
            .bind(("featured", featured))
            .bind(("reading_time", reading_time as i64))
            .bind(("format", format_str.to_string()))
            .bind(("content_hash", entry.content_hash.clone()))
            .bind(("series_slug", series_slug))
            .bind(("series_title", series_title))
            .bind(("series_position", series_position.map(|p| p as i64)))
            .await?;

            create_tags_for_post(db, slug, &tags).await?;

            info!(slug = %slug, "Inserted declarative article");
            stats.inserted += 1;
        }
    }

    // Phase 3: Delete declarative articles no longer in the manifest
    let mut result = db
        .query(
            "SELECT slug FROM blog_posts WHERE source = 'declarative' AND slug NOT IN $active_slugs",
        )
        .bind(("active_slugs", active_slugs))
        .await?;

    let stale_rows: Vec<HashMap<String, String>> = result.take(0).unwrap_or_default();
    for row in &stale_rows {
        if let Some(slug) = row.get("slug") {
            // Delete tag relations and the article
            db.query(
                r#"
                LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = $slug LIMIT 1)[0];
                DELETE tagged WHERE in = $post;
                DELETE FROM blog_posts WHERE slug = $slug;
                "#,
            )
            .bind(("slug", slug.to_string()))
            .await?;

            info!(slug = %slug, "Deleted stale declarative article");
            stats.deleted += 1;
        }
    }

    Ok(stats)
}

/// Backfill embeddings for declarative articles that lack them.
///
/// Queries for articles with `source = 'declarative' AND embedding IS NULL`,
/// generates an embedding via the VectorSearch actor, and stores it.
/// Runs asynchronously — intended to be spawned as a background task.
pub async fn backfill_embeddings(
    db: Surreal<Db>,
    vector_search: kameo::actor::ActorRef<crate::actors::vector_search::VectorSearch>,
    vector_truncation: usize,
) {
    use crate::actors::vector_search::GenerateEmbedding;

    let mut result = match db
        .query("SELECT slug, content FROM blog_posts WHERE source = 'declarative' AND embedding IS NONE")
        .await
    {
        Ok(r) => r,
        Err(e) => {
            warn!("Failed to query articles for embedding backfill: {}", e);
            return;
        }
    };

    let rows: Vec<HashMap<String, String>> = result.take(0).unwrap_or_default();
    if rows.is_empty() {
        return;
    }

    info!(
        count = rows.len(),
        "Backfilling embeddings for declarative articles"
    );

    for row in &rows {
        let Some(slug) = row.get("slug") else {
            continue;
        };
        let Some(content) = row.get("content") else {
            continue;
        };

        // Truncate content for embedding
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
                if let Err(e) = db
                    .query("UPDATE blog_posts SET embedding = $embedding WHERE slug = $slug")
                    .bind(("slug", slug.to_string()))
                    .bind(("embedding", embedding))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_deserialization() {
        let json = r#"{
            "hello-world": {
                "slug": "hello-world",
                "filename": "hello-world.md",
                "format": "markdown",
                "published": true,
                "content_hash": "sha256:abc123"
            },
            "typst-post": {
                "slug": "typst-post",
                "filename": "typst-post.typ",
                "html_filename": "typst-post.html",
                "format": "typst",
                "published": false,
                "content_hash": "sha256:def456"
            }
        }"#;

        let manifest: HashMap<String, ManifestEntry> = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.len(), 2);

        let md = &manifest["hello-world"];
        assert_eq!(md.slug, "hello-world");
        assert_eq!(md.format, "markdown");
        assert!(md.published);
        assert!(md.html_filename.is_none());

        let typ = &manifest["typst-post"];
        assert_eq!(typ.slug, "typst-post");
        assert_eq!(typ.format, "typst");
        assert!(!typ.published);
        assert_eq!(typ.html_filename.as_deref(), Some("typst-post.html"));
    }

    #[test]
    fn test_load_stats_default() {
        let stats = LoadStats::default();
        assert_eq!(stats.inserted, 0);
        assert_eq!(stats.updated, 0);
        assert_eq!(stats.deleted, 0);
        assert_eq!(stats.skipped, 0);
    }
}
