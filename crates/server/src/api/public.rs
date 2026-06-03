//! Public read-only API handlers shared by CSR and other clients.

use axum::{
    Json,
    extract::{Path, State},
};
use plinth_shared::{SiteConfig, SiteContent};

use crate::{AppState, actors::core_cache::GetSiteContent, error::PlinthError};

/// GET /api/config
pub async fn get_site_config(State(state): State<AppState>) -> Json<SiteConfig> {
    Json(state.site_config)
}

/// GET /api/content/{key}
pub async fn get_site_content(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<Option<SiteContent>>, PlinthError> {
    let content = state
        .core_cache
        .ask(GetSiteContent(key))
        .await
        .map_err(PlinthError::actor)?;

    Ok(Json(content))
}
