use crate::actors::content_cache::{GetAllPortfolioItems, GetPortfolioItem};
use crate::AppState;
use leptos::prelude::*;
use shared::PortfolioItem;

/// Server function to get all portfolio items
#[server(GetPortfolioItems, "/api")]
pub async fn get_portfolio_items() -> Result<Vec<PortfolioItem>, ServerFnError> {
    let app_state = expect_context::<AppState>();

    app_state
        .content_cache
        .ask(GetAllPortfolioItems)
        .await
        .map_err(|e| ServerFnError::ServerError(format!("Actor error: {}", e)))
}

/// Server function to get a single portfolio item by slug
#[server(GetPortfolioItemBySlug, "/api")]
pub async fn get_portfolio_item_by_slug(
    slug: String,
) -> Result<Option<PortfolioItem>, ServerFnError> {
    let app_state = expect_context::<AppState>();

    app_state
        .content_cache
        .ask(GetPortfolioItem(slug))
        .await
        .map_err(|e| ServerFnError::ServerError(format!("Actor error: {}", e)))
}
