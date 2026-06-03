use axum::{
    Json,
    extract::{Path, State},
};
use plinth_shared::{PublishActivityRequest, validate_activity_fields};
use tracing::warn;

use super::cache::ActivityInvalidateCache;
use crate::AppState;
use crate::error::PlinthError;
use crate::services::db::{delete_activity_item, patch_activity_item, upsert_activity_item};

/// POST /api/admin/activity — upsert by natural key.
pub async fn publish_activity_item(
    State(state): State<AppState>,
    Json(request): Json<PublishActivityRequest>,
) -> Result<Json<serde_json::Value>, PlinthError> {
    validate_activity_fields(
        request.impact,
        &request.repo_owner,
        &request.repo_name,
        request.number,
    )
    .map_err(|e| PlinthError::validation(e.to_string()))?;

    let fetched_at = chrono::Utc::now();
    let title = request.title.clone();
    let id = upsert_activity_item(&state.db, &request, fetched_at).await?;

    if let Err(e) = state.activity_cache.ask(ActivityInvalidateCache).await {
        warn!("Activity cache invalidation failed: {e}");
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "id": id,
        "message": format!("Activity '{title}' published"),
    })))
}

/// DELETE /api/admin/activity/{id}
pub async fn delete_activity_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, PlinthError> {
    let deleted = delete_activity_item(&state.db, id).await?;
    if deleted == 0 {
        return Err(PlinthError::not_found(format!(
            "activity item {id} not found"
        )));
    }

    if let Err(e) = state.activity_cache.ask(ActivityInvalidateCache).await {
        warn!("Activity cache invalidation failed: {e}");
    }

    Ok(Json(serde_json::json!({ "success": true, "deleted": id })))
}

/// PATCH /api/admin/activity/{id}
#[derive(serde::Deserialize)]
pub struct PatchActivityBody {
    pub impact: Option<i16>,
    pub featured: Option<bool>,
    pub published: Option<bool>,
}

pub async fn patch_activity_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<PatchActivityBody>,
) -> Result<Json<serde_json::Value>, PlinthError> {
    if let Some(impact) = body.impact
        && !(1..=10).contains(&impact)
    {
        return Err(PlinthError::validation("impact must be between 1 and 10"));
    }

    let updated =
        patch_activity_item(&state.db, id, body.impact, body.featured, body.published).await?;
    if !updated {
        return Err(PlinthError::not_found(format!(
            "activity item {id} not found"
        )));
    }

    if let Err(e) = state.activity_cache.ask(ActivityInvalidateCache).await {
        warn!("Activity cache invalidation failed: {e}");
    }

    Ok(Json(serde_json::json!({ "success": true, "updated": id })))
}
