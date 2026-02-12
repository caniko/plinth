use leptos::*;
use crate::AppState;
use crate::actors::content_cache::{GetAllPortfolioItems, GetPortfolioItem};
use shared::PortfolioItem;

/// Server function to get all portfolio items
#[server(GetPortfolioItems, "/api")]
pub async fn get_portfolio_items() -> Result<Vec<PortfolioItem>, ServerFnError> {
    use axum::extract::State;

    // Get the app state from Leptos context
    let app_state = expect_context::<AppState>();

    // Query the ContentCache actor
    let result = app_state
        .content_cache
        .ask(GetAllPortfolioItems)
        .send()
        .await
        .map_err(|e| ServerFnError::ServerError(format!("Actor error: {}", e)))?;

    result.map_err(|e| ServerFnError::ServerError(e))
}

/// Server function to get a single portfolio item by slug
#[server(GetPortfolioItemBySlug, "/api")]
pub async fn get_portfolio_item_by_slug(slug: String) -> Result<Option<PortfolioItem>, ServerFnError> {
    let app_state = expect_context::<AppState>();

    let result = app_state
        .content_cache
        .ask(GetPortfolioItem(slug))
        .send()
        .await
        .map_err(|e| ServerFnError::ServerError(format!("Actor error: {}", e)))?;

    result.map_err(|e| ServerFnError::ServerError(e))
}
