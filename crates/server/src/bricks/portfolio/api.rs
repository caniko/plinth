//! Portfolio public API handlers.

use axum::{
    Json,
    extract::{Path, State},
};
use plinth_shared::PortfolioItem;

use super::cache::{GetAllPortfolioItems, GetPortfolioItem};
use crate::{AppState, error::PlinthError};

/// GET /api/portfolio
pub async fn list_portfolio_items(
    State(state): State<AppState>,
) -> Result<Json<Vec<PortfolioItem>>, PlinthError> {
    let items = state
        .portfolio_cache
        .ask(GetAllPortfolioItems)
        .await
        .map_err(PlinthError::actor)?;

    Ok(Json(items))
}

/// GET /api/portfolio/{slug}
pub async fn get_portfolio_item(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Option<PortfolioItem>>, PlinthError> {
    let item = state
        .portfolio_cache
        .ask(GetPortfolioItem(slug))
        .await
        .map_err(PlinthError::actor)?;

    Ok(Json(item))
}
