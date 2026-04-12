use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use plinth_shared::{SiteContent, Tag, UpdateSiteContentRequest};
use serde::Serialize;
use tracing::{error, warn};

use crate::{
    AppState,
    actors::core_cache::{GetAllTags, GetSiteContent, InvalidateCache},
};

/// Error response for admin API failures.
///
/// The `details` field is logged server-side but never sent to the client,
/// preventing internal error messages (DB errors, stack traces) from leaking.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing)]
    pub details: Option<String>,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        if let Some(ref details) = self.details {
            error!(error = %self.error, details = %details, "Admin API error");
        }
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

/// List all tags with post counts
pub async fn list_tags(State(state): State<AppState>) -> Result<Json<Vec<Tag>>, ErrorResponse> {
    let tags = state
        .core_cache
        .ask(GetAllTags)
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to query tags".to_string(),
            details: Some(e.to_string()),
        })?;

    Ok(Json(tags))
}

/// Update site content by key (upsert)
pub async fn update_site_content(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(request): Json<UpdateSiteContentRequest>,
) -> Result<Json<serde_json::Value>, ErrorResponse> {
    let db = &state.db;

    // Upsert: delete existing then create in a transaction
    db.query(
        r##"
        BEGIN TRANSACTION;
        DELETE FROM site_content WHERE key = $key;
        CREATE site_content CONTENT {
            key: $key,
            title: $title,
            content: $content,
            html_content: $html_content,
            updated_at: time::now()
        };
        COMMIT TRANSACTION;
        "##,
    )
    .bind(("key", key.clone()))
    .bind(("title", request.title))
    .bind(("content", request.content))
    .bind(("html_content", request.html_content))
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to update site content".to_string(),
        details: Some(e.to_string()),
    })?;

    // Invalidate cache
    if let Err(e) = state.core_cache.ask(InvalidateCache).await {
        warn!("Cache invalidation failed: {e}");
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "key": key,
        "message": format!("Site content '{}' updated successfully", key)
    })))
}

/// Get site content by key
pub async fn get_admin_site_content(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Json<Option<SiteContent>>, ErrorResponse> {
    let content = state
        .core_cache
        .ask(GetSiteContent(key))
        .await
        .map_err(|e| ErrorResponse {
            error: "Failed to query site content".to_string(),
            details: Some(e.to_string()),
        })?;

    Ok(Json(content))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[test]
    fn test_error_response_status_code() {
        let error = ErrorResponse {
            error: "Something went wrong".to_string(),
            details: Some("More info".to_string()),
        };
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_error_response_without_details() {
        let error = ErrorResponse {
            error: "Failure".to_string(),
            details: None,
        };
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_error_response_does_not_leak_details() {
        let error = ErrorResponse {
            error: "Something went wrong".to_string(),
            details: Some("SENSITIVE: connection string leaked".to_string()),
        };
        let response = error.into_response();
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        // details must NOT appear in the JSON response body
        assert!(json.get("details").is_none());
        // error message should still be present
        assert_eq!(json["error"], "Something went wrong");
    }
}
