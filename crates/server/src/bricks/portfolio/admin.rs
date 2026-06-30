//! Portfolio-specific admin API handlers.

use axum::{Json, extract::State};
use plinth_shared::{ContentFormat, PortfolioItem, PublishPortfolioRequest, normalized_links};
use serde::Serialize;
use tracing::warn;

use super::cache::InvalidateCache as PortfolioInvalidateCache;
use crate::{
    AppState,
    error::PlinthError,
    services::{db::upsert_portfolio_item, markdown_processor::markdown_to_html},
};

/// Response for successful portfolio publication.
#[derive(Debug, Serialize)]
pub struct PublishPortfolioResponse {
    pub success: bool,
    pub slug: String,
    pub id: Option<String>,
    pub message: String,
}

/// Publish or update a portfolio item.
///
/// The item is upserted by slug. Markdown content is rendered to HTML on the
/// server before persistence.
pub async fn publish_portfolio_item(
    State(state): State<AppState>,
    Json(request): Json<PublishPortfolioRequest>,
) -> Result<Json<PublishPortfolioResponse>, PlinthError> {
    let content_format = request
        .content_format
        .clone()
        .unwrap_or(ContentFormat::Markdown);

    if content_format != ContentFormat::Markdown {
        return Err(PlinthError::validation(
            "Unsupported portfolio content format",
        ));
    }

    let title = required_text("title", request.title)?;
    let description = required_text("description", request.description)?;
    let slug = request
        .slug
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| PortfolioItem::slugify(&title));

    if slug.is_empty() {
        return Err(PlinthError::validation("Slug is required"));
    }

    if request.tech_stack.is_empty() || request.tech_stack.iter().any(|s| s.trim().is_empty()) {
        return Err(PlinthError::validation("tech_stack is required"));
    }

    let html_content = request
        .content
        .as_ref()
        .map(|content| markdown_to_html(content))
        .or(request.html_content);

    let item = PortfolioItem {
        id: None,
        slug: slug.clone(),
        title: title.clone(),
        description,
        content: request.content,
        html_content,
        tech_stack: request.tech_stack,
        link: trim_optional_url(request.link),
        demo: trim_optional_url(request.demo),
        project_url: trim_optional_url(request.project_url),
        links: normalized_links(request.links),
        image_url: request.image_url,
        date: request.date,
        featured: request.featured,
        order: request.order,
    };

    let id = upsert_portfolio_item(&state.db, &item).await?;

    if let Err(e) = state.portfolio_cache.ask(PortfolioInvalidateCache).await {
        warn!("Portfolio cache invalidation failed: {e}");
    }
    plinth_client::invalidate_portfolio_static_routes(&slug);

    Ok(Json(PublishPortfolioResponse {
        success: true,
        slug,
        id: Some(format!("portfolio_items:{id}")),
        message: format!("Portfolio item '{title}' published successfully"),
    }))
}

/// Summary of results from a batch portfolio sync.
#[derive(Debug, Serialize)]
pub struct SyncPortfolioResponse {
    pub success: bool,
    pub published: usize,
    pub failed: usize,
    pub errors: Vec<String>,
    pub message: String,
}

/// Batch-publish or update portfolio items from a list of manifests.
///
/// Accepts an array of PublishPortfolioRequest items. Each is validated,
/// rendered to HTML, and upserted by slug. The cache is invalidated once
/// at the end.
pub async fn sync_portfolio_items(
    State(state): State<AppState>,
    Json(requests): Json<Vec<PublishPortfolioRequest>>,
) -> Result<Json<SyncPortfolioResponse>, PlinthError> {
    let mut published = 0usize;
    let mut errors = Vec::new();

    for request in requests {
        let content_format = request
            .content_format
            .clone()
            .unwrap_or(ContentFormat::Markdown);

        if content_format != ContentFormat::Markdown {
            errors.push(format!(
                "Item '{}': Unsupported format {:?}",
                request.title, content_format
            ));
            continue;
        }

        let title = match required_text("title", request.title.clone()) {
            Ok(t) => t,
            Err(e) => {
                errors.push(format!("Item (unnamed): {e}"));
                continue;
            }
        };

        let description = match required_text("description", request.description.clone()) {
            Ok(d) => d,
            Err(e) => {
                errors.push(format!("Item '{title}': {e}"));
                continue;
            }
        };

        let slug = request
            .slug
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| PortfolioItem::slugify(&title));

        if slug.is_empty() {
            errors.push(format!("Item '{title}': slug is required"));
            continue;
        }

        if request.tech_stack.is_empty() || request.tech_stack.iter().any(|s| s.trim().is_empty()) {
            errors.push(format!("Item '{title}': tech_stack is required"));
            continue;
        }

        let html_content = request
            .content
            .as_ref()
            .map(|content| markdown_to_html(content))
            .or(request.html_content.clone());

        let item = PortfolioItem {
            id: None,
            slug: slug.clone(),
            title,
            description,
            content: request.content.clone(),
            html_content,
            tech_stack: request.tech_stack.clone(),
            link: trim_optional_url(request.link.clone()),
            demo: trim_optional_url(request.demo.clone()),
            project_url: trim_optional_url(request.project_url.clone()),
            links: normalized_links(request.links.clone()),
            image_url: request.image_url.clone(),
            date: request.date,
            featured: request.featured,
            order: request.order,
        };

        if let Err(e) = upsert_portfolio_item(&state.db, &item).await {
            errors.push(format!("Item '{}': DB upsert failed: {e}", item.title));
            continue;
        }

        published += 1;
    }

    if let Err(e) = state.portfolio_cache.ask(PortfolioInvalidateCache).await {
        warn!("Portfolio cache invalidation after sync failed: {e}");
    }

    let failed = errors.len();
    let message = format!("Synced {published} portfolio item(s) with {failed} error(s)");

    Ok(Json(SyncPortfolioResponse {
        success: failed == 0,
        published,
        failed,
        errors,
        message,
    }))
}

fn required_text(field: &str, value: String) -> Result<String, PlinthError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(PlinthError::validation(format!("{field} is required")))
    } else {
        Ok(trimmed.to_string())
    }
}

fn trim_optional_url(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
