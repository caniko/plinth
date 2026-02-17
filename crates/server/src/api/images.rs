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
) -> Result<Response, StatusCode> {
    // Validate asset_id is a UUID to prevent path traversal / injection
    uuid::Uuid::parse_str(&asset_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let immich = state
        .immich_config
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let url = immich_asset_url(immich, &asset_id, &params.size);

    let response = state
        .http_client
        .get(&url)
        .header("x-api-key", &immich.api_key)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !response.status().is_success() {
        return Err(StatusCode::NOT_FOUND);
    }

    // Forward content-type from Immich
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .cloned()
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

    // Stream the body without buffering
    let body = Body::from_stream(response.bytes_stream());

    Ok((headers, body).into_response())
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
}
