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
        db::sync_post_tags_cache,
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
/// processes it into HTML, generates an embedding, and stores it in SurrealDB.
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
                let mut result = db
                    .query(
                        "SELECT VALUE math::max(series_position) FROM blog_posts WHERE series_slug = $slug",
                    )
                    .bind(("slug", s_slug.to_string()))
                    .await
                    .map_err(|e| ErrorResponse {
                        error: "Failed to query series position".to_string(),
                        details: Some(e.to_string()),
                    })?;
                let max_pos: Option<u32> = result.take(0).unwrap_or(None);
                max_pos.unwrap_or(0) + 1
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
        id: None, // Will be generated by SurrealDB
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

    // Insert into SurrealDB (convert through serde_json::Value for SurrealValue compat)
    let blog_post_value = serde_json::to_value(&blog_post).map_err(|e| ErrorResponse {
        error: "Failed to serialize blog post".to_string(),
        details: Some(e.to_string()),
    })?;
    let created_value: Option<serde_json::Value> = db
        .create("blog_posts")
        .content(blog_post_value)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to create blog post in database".to_string(),
            details: Some(e.to_string()),
        })?;

    let created_post: BlogPost = created_value
        .ok_or_else(|| ErrorResponse {
            error: "No blog post returned from database".to_string(),
            details: None,
        })
        .and_then(|v| {
            serde_json::from_value(v).map_err(|e| ErrorResponse {
                error: "Failed to deserialize created blog post".to_string(),
                details: Some(e.to_string()),
            })
        })?;

    // Create tags and graph relations
    crate::services::db::create_tags_for_post(db, &slug, &tags)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to create tag relations".to_string(),
            details: Some(e.to_string()),
        })?;

    // Invalidate caches
    if let Err(e) = state.core_cache.ask(CoreInvalidateCache).await {
        warn!("Core cache invalidation failed: {e}");
    }
    if let Err(e) = state.blog_cache.ask(BlogInvalidateCache).await {
        warn!("Blog cache invalidation failed: {e}");
    }

    Ok(Json(PublishArticleResponse {
        success: true,
        slug: slug.clone(),
        id: created_post.id,
        message: format!("Article '{}' published successfully", title),
    }))
}

/// Delete a blog article by slug (admin only)
pub async fn delete_article(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let db = &state.db;

    // Check that the article exists
    let mut result = db
        .query("SELECT count() AS c FROM blog_posts WHERE slug = $slug")
        .bind(("slug", slug.clone()))
        .await
        .map_err(|e| ErrorResponse {
            error: "Database query failed".to_string(),
            details: Some(e.to_string()),
        })?;

    let count: Option<i64> = result.take("c").unwrap_or(None);
    if count.unwrap_or(0) == 0 {
        return Err(ErrorResponse {
            error: "Article not found".to_string(),
            details: None,
        });
    }

    // Delete tag relations and the article itself in a transaction
    db.query(
        r#"
        BEGIN TRANSACTION;
        LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = $slug LIMIT 1)[0];
        DELETE tagged WHERE in = $post;
        DELETE FROM blog_posts WHERE slug = $slug;
        COMMIT TRANSACTION;
    "#,
    )
    .bind(("slug", slug.clone()))
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to delete article".to_string(),
        details: Some(e.to_string()),
    })?;

    // Invalidate caches
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
    let db = &state.db;
    let tag_slug = generate_slug(&request.tag);

    // Ensure tag exists
    db.query(
        r#"
        IF (SELECT count() FROM tags WHERE slug = $tag_slug) = 0 THEN
            CREATE tags CONTENT {
                name: $tag_name,
                slug: $tag_slug,
                created_at: time::now()
            }
        END;
    "#,
    )
    .bind(("tag_name", request.tag.clone()))
    .bind(("tag_slug", tag_slug.clone()))
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to ensure tag exists".to_string(),
        details: Some(e.to_string()),
    })?;

    // Create relation
    db.query(
        r#"
        LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = $post_slug LIMIT 1)[0];
        LET $tag = (SELECT VALUE id FROM tags WHERE slug = $tag_slug LIMIT 1)[0];
        RELATE $post->tagged->$tag CONTENT { created_at: time::now() };
    "#,
    )
    .bind(("post_slug", post_slug.clone()))
    .bind(("tag_slug", tag_slug))
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to add tag to post".to_string(),
        details: Some(e.to_string()),
    })?;

    // Sync denormalized cache
    sync_post_tags_cache(db, &post_slug)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to sync tag cache".to_string(),
            details: Some(e.to_string()),
        })?;

    // Invalidate caches
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
    let db = &state.db;

    // Delete the relation
    db.query(
        r#"
        LET $post = (SELECT VALUE id FROM blog_posts WHERE slug = $post_slug LIMIT 1)[0];
        LET $tag = (SELECT VALUE id FROM tags WHERE slug = $tag_slug LIMIT 1)[0];
        DELETE tagged WHERE in = $post AND out = $tag;
    "#,
    )
    .bind(("post_slug", post_slug.clone()))
    .bind(("tag_slug", tag_slug.clone()))
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to remove tag from post".to_string(),
        details: Some(e.to_string()),
    })?;

    // Sync denormalized cache
    sync_post_tags_cache(db, &post_slug)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to sync tag cache".to_string(),
            details: Some(e.to_string()),
        })?;

    // Invalidate caches
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
