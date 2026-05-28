use axum::{
    Json,
    extract::{Path, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use plinth_shared::{SiteContent, Tag, UpdateSiteContentRequest};
use tracing::warn;

use crate::{
    AppState,
    actors::core_cache::{GetAllTags, GetSiteContent, InvalidateCache},
    error::PlinthError,
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
                // Audit trail for probing/brute-force; never log the token value.
                warn!("Rejected admin request: invalid bearer token");
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => {
            warn!("Rejected admin request: missing or malformed Authorization header");
            Err(StatusCode::UNAUTHORIZED)
        }
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

/// List all tags with post counts
pub async fn list_tags(State(state): State<AppState>) -> Result<Json<Vec<Tag>>, PlinthError> {
    let tags = state
        .core_cache
        .ask(GetAllTags)
        .await
        .map_err(PlinthError::actor)?;

    Ok(Json(tags))
}

/// Update site content by key (upsert)
pub async fn update_site_content(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(request): Json<UpdateSiteContentRequest>,
) -> Result<Json<serde_json::Value>, PlinthError> {
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
    .await?;

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
) -> Result<Json<Option<SiteContent>>, PlinthError> {
    let content = state
        .core_cache
        .ask(GetSiteContent(key))
        .await
        .map_err(PlinthError::actor)?;

    Ok(Json(content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"secret-token", b"secret-token"));
        assert!(!constant_time_eq(b"secret-token", b"secret-toaken"));
        assert!(!constant_time_eq(b"secret", b"secret-token"));
        assert!(!constant_time_eq(b"", b"x"));
        assert!(constant_time_eq(b"", b""));
    }
}
