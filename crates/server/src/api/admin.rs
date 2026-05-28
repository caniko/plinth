use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use plinth_shared::{SiteContent, Tag, UpdateSiteContentRequest};
use serde::Serialize;
use tracing::{error, warn};

use crate::{
    AppState,
    actors::core_cache::{GetAllTags, GetSiteContent, InvalidateCache},
};

/// Authentication middleware to verify the admin API key.
///
/// The API key is read by the caller and passed as middleware state. If no key
/// is configured, all admin endpoints reject requests.
pub async fn auth_middleware(
    State(api_key): State<Option<String>>,
    req: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let Some(ref expected_key) = api_key else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    match auth_header {
        Some(header_value) if header_value.starts_with("Bearer ") => {
            let token = &header_value[7..];
            if constant_time_eq(token.as_bytes(), expected_key.as_bytes()) {
                Ok(next.run(req).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Compare two byte slices in constant time to avoid leaking the API key via a
/// timing side-channel. The length is allowed to leak (token lengths are not
/// secret), but byte-by-byte comparison does not short-circuit on the first
/// mismatch.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

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
    sqlx::query(
        r#"
        INSERT INTO site_content (key, title, content, html_content, updated_at)
        VALUES ($1, $2, $3, $4, now())
        ON CONFLICT (key) DO UPDATE SET
            title = EXCLUDED.title,
            content = EXCLUDED.content,
            html_content = EXCLUDED.html_content,
            updated_at = now()
        "#,
    )
    .bind(&key)
    .bind(request.title)
    .bind(request.content)
    .bind(request.html_content)
    .execute(&state.db)
    .await
    .map_err(|e| ErrorResponse {
        error: "Failed to update site content".to_string(),
        details: Some(e.to_string()),
    })?;

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
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"secret-token", b"secret-token"));
        assert!(!constant_time_eq(b"secret-token", b"secret-toaken"));
        assert!(!constant_time_eq(b"secret", b"secret-token"));
        assert!(!constant_time_eq(b"", b"x"));
        assert!(constant_time_eq(b"", b""));
    }

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
