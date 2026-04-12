use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;

use crate::{AppState, ImmichConfig};

#[derive(Debug, Deserialize)]
pub struct ImageQuery {
    /// Image size: "original", "preview", "thumbnail"
    #[serde(default = "default_size")]
    pub size: String,
}

fn default_size() -> String {
    "original".to_string()
}

/// Proxy an image from Immich to the reader.
///
/// `GET /api/images/{asset_id}?size=original|preview|thumbnail`
///
/// Immich is not publicly exposed; Plinth fetches the image with an API key
/// and streams it to the client with aggressive caching headers.
pub async fn serve_image(
    State(state): State<AppState>,
    Path(asset_id): Path<String>,
    Query(params): Query<ImageQuery>,
    request_headers: HeaderMap,
) -> Result<Response, StatusCode> {
    // Validate asset_id is a UUID to prevent path traversal / injection
    uuid::Uuid::parse_str(&asset_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // ETag is derived from asset_id + size (content-addressed by design)
    let etag = generate_etag(&asset_id, &params.size);

    // Return 304 Not Modified if the client already has this version
    if let Some(if_none_match) = request_headers.get(header::IF_NONE_MATCH)
        && if_none_match.as_bytes() == etag.as_bytes()
    {
        let mut headers = HeaderMap::new();
        if let Ok(val) = header::HeaderValue::from_str(&etag) {
            headers.insert(header::ETAG, val);
        }
        return Ok((StatusCode::NOT_MODIFIED, headers).into_response());
    }

    let immich = state
        .immich_config
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let url = immich_asset_url(immich, &asset_id, &params.size);

    let response = state
        .http_client
        .get(&url)
        .header("x-api-key", &immich.api_key)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Forward content-type from Immich, but only allow known image types
    // to prevent a compromised upstream from serving HTML/JS via the proxy.
    const ALLOWED_IMAGE_TYPES: &[&str] = &[
        "image/jpeg",
        "image/png",
        "image/webp",
        "image/gif",
        "image/svg+xml",
        "image/avif",
    ];
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|ct| {
            let ct_str = ct.to_str().unwrap_or("");
            if ALLOWED_IMAGE_TYPES.iter().any(|t| ct_str.starts_with(t)) {
                Some(ct.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| header::HeaderValue::from_static("application/octet-stream"));

    let cache_max_age = state.config.images.cache_max_age;
    let cache_value = format!("public, max-age={cache_max_age}, immutable");

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, content_type);
    headers.insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_str(&cache_value).unwrap_or_else(|_| {
            header::HeaderValue::from_static("public, max-age=31536000, immutable")
        }),
    );
    if let Ok(val) = header::HeaderValue::from_str(&etag) {
        headers.insert(header::ETAG, val);
    }

    // Stream the body without buffering
    let body = Body::from_stream(response.bytes_stream());

    Ok((headers, body).into_response())
}

/// Generate an ETag for the given asset and size variant.
///
/// Since Immich asset IDs are content-addressed (the CLI uses SHA-256 hashing
/// for deduplication), the same UUID always returns the same image data.
fn generate_etag(asset_id: &str, size: &str) -> String {
    format!("\"{}:{}\"", asset_id, size)
}

/// Build the Immich URL for the requested asset and size variant.
fn immich_asset_url(config: &ImmichConfig, asset_id: &str, size: &str) -> String {
    match size {
        "thumbnail" => format!(
            "{}/api/assets/{}/thumbnail?size=thumbnail",
            config.base_url, asset_id
        ),
        "preview" => format!(
            "{}/api/assets/{}/thumbnail?size=preview",
            config.base_url, asset_id
        ),
        _ => format!("{}/api/assets/{}/original", config.base_url, asset_id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immich_asset_url_original() {
        let config = ImmichConfig {
            base_url: "http://immich:2283".to_string(),
            api_key: "key".to_string(),
        };
        let url = immich_asset_url(&config, "abc-123", "original");
        assert_eq!(url, "http://immich:2283/api/assets/abc-123/original");
    }

    #[test]
    fn test_immich_asset_url_thumbnail() {
        let config = ImmichConfig {
            base_url: "http://immich:2283".to_string(),
            api_key: "key".to_string(),
        };
        let url = immich_asset_url(&config, "abc-123", "thumbnail");
        assert_eq!(
            url,
            "http://immich:2283/api/assets/abc-123/thumbnail?size=thumbnail"
        );
    }

    #[test]
    fn test_immich_asset_url_preview() {
        let config = ImmichConfig {
            base_url: "http://immich:2283".to_string(),
            api_key: "key".to_string(),
        };
        let url = immich_asset_url(&config, "abc-123", "preview");
        assert_eq!(
            url,
            "http://immich:2283/api/assets/abc-123/thumbnail?size=preview"
        );
    }

    #[test]
    fn test_immich_asset_url_unknown_defaults_to_original() {
        let config = ImmichConfig {
            base_url: "http://immich:2283".to_string(),
            api_key: "key".to_string(),
        };
        let url = immich_asset_url(&config, "abc-123", "bogus");
        assert_eq!(url, "http://immich:2283/api/assets/abc-123/original");
    }

    #[test]
    fn test_image_query_default_size() {
        let query: ImageQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(query.size, "original");
    }

    #[test]
    fn test_generate_etag() {
        let etag = generate_etag("550e8400-e29b-41d4-a716-446655440000", "original");
        assert_eq!(etag, "\"550e8400-e29b-41d4-a716-446655440000:original\"");
    }

    #[test]
    fn test_generate_etag_different_sizes() {
        let original = generate_etag("abc-123", "original");
        let preview = generate_etag("abc-123", "preview");
        let thumbnail = generate_etag("abc-123", "thumbnail");
        assert_ne!(original, preview);
        assert_ne!(preview, thumbnail);
        assert_ne!(original, thumbnail);
    }

    #[test]
    fn test_generate_etag_empty_strings() {
        let etag = generate_etag("", "");
        assert_eq!(etag, "\":\"");
    }

    #[test]
    fn test_image_query_explicit_sizes() {
        let query: ImageQuery = serde_json::from_str(r#"{"size": "preview"}"#).unwrap();
        assert_eq!(query.size, "preview");

        let query: ImageQuery = serde_json::from_str(r#"{"size": "thumbnail"}"#).unwrap();
        assert_eq!(query.size, "thumbnail");
    }

    #[test]
    fn test_image_query_ignores_unknown_fields() {
        // CLI encodes dimensions as ?w=X&h=Y — these must not break deserialization
        let query: ImageQuery =
            serde_json::from_str(r#"{"size": "original", "w": 1920, "h": 1080}"#).unwrap();
        assert_eq!(query.size, "original");
    }
}
