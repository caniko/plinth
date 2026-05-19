//! Blog-specific admin API handlers.
//!
//! Extracted from the monolithic `api::admin` module. These handlers manage
//! blog post CRUD and tag operations.

use axum::{
    Json,
    extract::{Path, State},
};
use chrono::Utc;
use plinth_shared::{AddTagRequest, BlogPost, ContentFormat, PublishArticleRequest, humanize_slug};
use serde::Serialize;
use tracing::warn;

use super::cache::InvalidateCache as BlogInvalidateCache;
use crate::{
    AppState,
    actors::core_cache::InvalidateCache as CoreInvalidateCache,
    services::{
        db::{create_tags_for_post_tx, sync_post_tags_cache_tx, vector_or_none},
        markdown_processor::{generate_slug, parse_markdown},
    },
};

// Re-export shared error type from the original admin module.
pub use crate::api::admin::ErrorResponse;

/// Response for successful article publication
#[derive(Debug, Serialize)]
pub struct PublishArticleResponse {
    pub success: bool,
    pub slug: String,
    pub id: Option<String>,
    pub message: String,
}

/// Publish a new blog article
///
/// This endpoint accepts markdown content with optional frontmatter,
/// processes it into HTML, generates an embedding, and stores it in the database.
pub async fn publish_article(
    State(state): State<AppState>,
    Json(request): Json<PublishArticleRequest>,
) -> Result<Json<PublishArticleResponse>, ErrorResponse> {
    let content_format = request
        .content_format
        .clone()
        .unwrap_or(ContentFormat::Markdown);

    // Process content based on format
    let (html_content, markdown_content, frontmatter, reading_time) = match &content_format {
        ContentFormat::Markdown => {
            let parsed = parse_markdown(&request.content).map_err(|e| ErrorResponse {
                error: "Failed to parse markdown".to_string(),
                details: Some(e),
            })?;
            (
                parsed.html_content,
                parsed.markdown_content,
                parsed.frontmatter,
                parsed.reading_time_minutes,
            )
        }
        ContentFormat::Typst => {
            // For Typst, the CLI pre-renders HTML
            let html = request.html_content.clone().ok_or_else(|| ErrorResponse {
                error: "Typst posts require pre-rendered HTML".to_string(),
                details: Some(
                    "Set html_content in the request (CLI compiles Typst to HTML)".to_string(),
                ),
            })?;
            let reading_time = BlogPost::calculate_reading_time(&request.content);
            (html, request.content.clone(), None, reading_time)
        }
    };

    // Determine title (from frontmatter or request)
    let title = request
        .title
        .or_else(|| frontmatter.as_ref().and_then(|fm| fm.title.clone()))
        .ok_or_else(|| ErrorResponse {
            error: "Title is required".to_string(),
            details: Some("Provide title in request or frontmatter".to_string()),
        })?;

    // Generate slug (from request, frontmatter, or title)
    let slug = request.slug.unwrap_or_else(|| generate_slug(&title));

    let tags = request
        .tags
        .or_else(|| frontmatter.as_ref().and_then(|fm| fm.tags.clone()))
        .unwrap_or_default();

    let default_author = state.config.site.author.name.clone();
    let author = request
        .author
        .or_else(|| frontmatter.as_ref().and_then(|fm| fm.author.clone()))
        .unwrap_or(default_author);

    let description = request
        .description
        .or_else(|| frontmatter.as_ref().and_then(|fm| fm.description.clone()))
        .unwrap_or_default();

    let published = request
        .published
        .or_else(|| frontmatter.as_ref().and_then(|fm| fm.published))
        .unwrap_or(true);

    let featured = request
        .featured
        .or_else(|| frontmatter.as_ref().and_then(|fm| fm.featured))
        .unwrap_or(false);

    let db = &state.db;

    // Resolve series fields from request or frontmatter
    let series_slug = request
        .series
        .or_else(|| frontmatter.as_ref().and_then(|fm| fm.series.clone()));

    let (series_slug, series_title, series_position) = if let Some(ref s_slug) = series_slug {
        let s_title = request
            .series_title
            .or_else(|| frontmatter.as_ref().and_then(|fm| fm.series_title.clone()))
            .unwrap_or_else(|| humanize_slug(s_slug));

        let s_position = request
            .series_position
            .or_else(|| frontmatter.as_ref().and_then(|fm| fm.series_position));

        // Auto-assign position if not provided.
        // Note: this is not fully atomic — concurrent publishes to the same
        // series could get duplicate positions. Acceptable for single-author use.
        let s_position = match s_position {
            Some(pos) => pos,
            None => {
                let max_pos: i32 = sqlx::query_scalar(
                    "SELECT COALESCE(MAX(series_position), 0)::integer FROM blog_posts WHERE series_slug = $1",
                )
                .bind(s_slug)
                .fetch_one(db)
                .await
                .map_err(|e| ErrorResponse {
                    error: "Failed to query series position".to_string(),
                    details: Some(e.to_string()),
                })?;
                (max_pos.max(0) as u32) + 1
            }
        };

        (Some(s_slug.clone()), Some(s_title), Some(s_position))
    } else {
        (None, None, None)
    };

    // Generate vector embedding if provided in request
    // (CLI tool will generate embeddings locally and send them)
    let embedding = request.embedding;

    // Create BlogPost record
    let blog_post = BlogPost {
        id: None,
        slug: slug.clone(),
        title: title.clone(),
        description,
        content: markdown_content,
        html_content,
        published_at: Utc::now(),
        updated_at: None,
        author,
        tags: tags.clone(),
        featured,
        published,
        reading_time_minutes: reading_time,
        embedding,
        content_format,
        source: "api".to_string(),
        content_hash: None,
        series_slug,
        series_title,
        series_position,
    };

    let embedding = vector_or_none(blog_post.embedding.clone());
    let mut tx = db.begin().await.map_err(|e| ErrorResponse {
        error: "Failed to start transaction".to_string(),
        details: Some(e.to_string()),
    })?;

    let created_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO blog_posts (
            slug,
            title,
            description,
            content,
            html_content,
            published_at,
            author,
            tags,
            featured,
            published,
            reading_time_minutes,
            embedding,
            content_format,
            source,
            content_hash,
            series_slug,
            series_title,
            series_position
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NULL, $15, $16, $17)
        RETURNING id
        "#,
    )
    .bind(&blog_post.slug)
    .bind(&blog_post.title)
    .bind(&blog_post.description)
    .bind(&blog_post.content)
    .bind(&blog_post.html_content)
    .bind(blog_post.published_at)
    .bind(&blog_post.author)
    .bind(&blog_post.tags)
    .bind(blog_post.featured)
    .bind(blog_post.published)
    .bind(blog_post.reading_time_minutes as i32)
    .bind(embedding)
    .bind(blog_post.content_format.as_str())
    .bind(&blog_post.source)
    .bind(&blog_post.series_slug)
    .bind(&blog_post.series_title)
    .bind(blog_post.series_position.map(|p| p as i32))
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to create blog post in database".to_string(),
        details: Some(e.to_string()),
    })?;

    create_tags_for_post_tx(&mut tx, &slug, &tags)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to create tag relations".to_string(),
            details: Some(e.to_string()),
        })?;
    sync_post_tags_cache_tx(&mut tx, &slug)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to sync tag cache".to_string(),
            details: Some(e.to_string()),
        })?;

    tx.commit().await.map_err(|e| ErrorResponse {
        error: "Failed to commit transaction".to_string(),
        details: Some(e.to_string()),
    })?;

    if let Err(e) = state.core_cache.ask(CoreInvalidateCache).await {
        warn!("Core cache invalidation failed: {e}");
    }
    if let Err(e) = state.blog_cache.ask(BlogInvalidateCache).await {
        warn!("Blog cache invalidation failed: {e}");
    }

    Ok(Json(PublishArticleResponse {
        success: true,
        slug: slug.clone(),
        id: Some(format!("blog_posts:{created_id}")),
        message: format!("Article '{}' published successfully", title),
    }))
}

/// Delete a blog article by slug (admin only)
pub async fn delete_article(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let mut tx = state.db.begin().await.map_err(|e| ErrorResponse {
        error: "Failed to start transaction".to_string(),
        details: Some(e.to_string()),
    })?;

    let deleted = sqlx::query("DELETE FROM blog_posts WHERE slug = $1")
        .bind(&slug)
        .execute(&mut *tx)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to delete article".to_string(),
            details: Some(e.to_string()),
        })?
        .rows_affected();

    if deleted == 0 {
        return Err(ErrorResponse {
            error: "Article not found".to_string(),
            details: None,
        });
    }

    tx.commit().await.map_err(|e| ErrorResponse {
        error: "Failed to commit transaction".to_string(),
        details: Some(e.to_string()),
    })?;

    if let Err(e) = state.core_cache.ask(CoreInvalidateCache).await {
        warn!("Core cache invalidation failed: {e}");
    }
    if let Err(e) = state.blog_cache.ask(BlogInvalidateCache).await {
        warn!("Blog cache invalidation failed: {e}");
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Article '{}' deleted successfully", slug)
    })))
}

/// Add a tag to a post
pub async fn add_tag_to_post(
    State(state): State<AppState>,
    Path(post_slug): Path<String>,
    Json(request): Json<AddTagRequest>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    crate::services::db::create_tags_for_post(
        &state.db,
        &post_slug,
        std::slice::from_ref(&request.tag),
    )
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to add tag to post".to_string(),
        details: Some(e.to_string()),
    })?;

    if let Err(e) = state.core_cache.ask(CoreInvalidateCache).await {
        warn!("Core cache invalidation failed: {e}");
    }
    if let Err(e) = state.blog_cache.ask(BlogInvalidateCache).await {
        warn!("Blog cache invalidation failed: {e}");
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Tag '{}' added to post '{}'", request.tag, post_slug)
    })))
}

/// Remove a tag from a post
pub async fn remove_tag_from_post(
    State(state): State<AppState>,
    Path((post_slug, tag_slug)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let mut tx = state.db.begin().await.map_err(|e| ErrorResponse {
        error: "Failed to start transaction".to_string(),
        details: Some(e.to_string()),
    })?;

    sqlx::query(
        r#"
        DELETE FROM blog_post_tags bpt
        USING blog_posts bp, tags t
        WHERE bpt.post_id = bp.id
          AND bpt.tag_id = t.id
          AND bp.slug = $1
          AND t.slug = $2
        "#,
    )
    .bind(&post_slug)
    .bind(&tag_slug)
    .execute(&mut *tx)
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to remove tag from post".to_string(),
        details: Some(e.to_string()),
    })?;

    sync_post_tags_cache_tx(&mut tx, &post_slug)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to sync tag cache".to_string(),
            details: Some(e.to_string()),
        })?;

    tx.commit().await.map_err(|e| ErrorResponse {
        error: "Failed to commit transaction".to_string(),
        details: Some(e.to_string()),
    })?;

    if let Err(e) = state.core_cache.ask(CoreInvalidateCache).await {
        warn!("Core cache invalidation failed: {e}");
    }
    if let Err(e) = state.blog_cache.ask(BlogInvalidateCache).await {
        warn!("Blog cache invalidation failed: {e}");
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Tag '{}' removed from post '{}'", tag_slug, post_slug)
    })))
}
