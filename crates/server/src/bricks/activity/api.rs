use axum::{
    Json,
    extract::{Path, Query, State},
};
use plinth_shared::{ActivityItem, ActivityListItem};
use serde::Deserialize;

use super::cache::{GetActivityItem, GetRankedActivity};
use crate::{AppState, error::PlinthError};

/// Query parameters for listing activity items (limit, featured).
#[derive(Deserialize, Default)]
pub struct ActivityListQuery {
    pub limit: Option<i64>,
    #[serde(default)]
    pub featured: bool,
}

/// GET /api/activity — ranked list (query: limit, featured)
pub async fn list_activity_items(
    State(state): State<AppState>,
    Query(q): Query<ActivityListQuery>,
) -> Result<Json<Vec<ActivityListItem>>, PlinthError> {
    let items = state
        .activity_cache
        .ask(GetRankedActivity {
            limit: q.limit,
            featured_only: q.featured,
        })
        .await
        .map_err(PlinthError::actor)?;

    Ok(Json(items))
}

/// GET /api/activity/{id}
pub async fn get_activity_item(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Option<ActivityItem>>, PlinthError> {
    let item = state
        .activity_cache
        .ask(GetActivityItem(id))
        .await
        .map_err(PlinthError::actor)?;

    Ok(Json(item))
}
