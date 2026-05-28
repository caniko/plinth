//! Portfolio-specific admin API handlers.

use axum::{Json, extract::State};
use plinth_shared::{ContentFormat, PortfolioItem, PublishPortfolioRequest};
use serde::Serialize;
use tracing::warn;

use super::cache::InvalidateCache as PortfolioInvalidateCache;
use crate::{
    AppState,
    api::admin::ErrorResponse,
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
) -> Result<Json<PublishPortfolioResponse>, ErrorResponse> {
    let content_format = request
        .content_format
        .clone()
        .unwrap_or(ContentFormat::Markdown);

    if content_format != ContentFormat::Markdown {
        return Err(ErrorResponse {
            error: "Unsupported portfolio content format".to_string(),
            details: Some("Only markdown portfolio manifests are supported".to_string()),
        });
    }

    let title = required_text("title", request.title)?;
    let description = required_text("description", request.description)?;
    let slug = request
        .slug
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| PortfolioItem::slugify(&title));

    if slug.is_empty() {
        return Err(ErrorResponse {
            error: "Slug is required".to_string(),
            details: Some("Provide slug or a title that can be slugified".to_string()),
        });
    }

    if request.tech_stack.is_empty() || request.tech_stack.iter().any(|s| s.trim().is_empty()) {
        return Err(ErrorResponse {
            error: "tech_stack is required".to_string(),
            details: Some("Provide at least one non-empty technology name".to_string()),
        });
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
        link: request.link,
        demo: request.demo,
        image_url: request.image_url,
        date: request.date,
        featured: request.featured,
        order: request.order,
    };

    let id = upsert_portfolio_item(&state.db, &item)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to upsert portfolio item".to_string(),
            details: Some(e.to_string()),
        })?;

    if let Err(e) = state.portfolio_cache.ask(PortfolioInvalidateCache).await {
        warn!("Portfolio cache invalidation failed: {e}");
    }

    Ok(Json(PublishPortfolioResponse {
        success: true,
        slug,
        id: Some(format!("portfolio_items:{id}")),
        message: format!("Portfolio item '{title}' published successfully"),
    }))
}

fn required_text(field: &str, value: String) -> Result<String, ErrorResponse> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(ErrorResponse {
            error: format!("{field} is required"),
            details: None,
        })
    } else {
        Ok(trimmed.to_string())
    }
}
