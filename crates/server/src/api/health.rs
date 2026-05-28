use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use tracing::warn;

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub api_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_search: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub immich: Option<&'static str>,
}

/// Health check endpoint — public, unauthenticated.
///
/// Probes database connectivity and Immich reachability, reports component status.
/// Returns 200 if DB is reachable, 503 otherwise.
pub async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let db_ok = match state.db.acquire().await {
        Ok(_) => true,
        Err(e) => {
            warn!(error = %e, "Health check: database acquire failed");
            false
        }
    };

    #[cfg(feature = "brick-blog")]
    let vs_status = state.vector_search.as_ref().map(|_| "available");
    #[cfg(not(feature = "brick-blog"))]
    let vs_status: Option<&str> = None;

    let immich_status = if let Some(ref immich) = state.immich_config {
        let url = format!("{}/api/server/ping", immich.base_url);
        let ok = match state
            .http_client
            .get(&url)
            .header("x-api-key", &immich.api_key)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(r) => r.status().is_success(),
            Err(e) => {
                warn!(error = %e, "Health check: Immich ping failed");
                false
            }
        };
        Some(if ok { "ok" } else { "unreachable" })
    } else {
        None
    };

    let (status_code, status_str) = if db_ok {
        (StatusCode::OK, "ok")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "degraded")
    };

    (
        status_code,
        Json(HealthResponse {
            status: status_str,
            version: env!("CARGO_PKG_VERSION"),
            api_version: plinth_shared::API_VERSION,
            db: Some(if db_ok { "ok" } else { "unreachable" }),
            vector_search: vs_status,
            immich: immich_status,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_all_fields() {
        let resp = HealthResponse {
            status: "ok",
            version: "0.1.0",
            api_version: 1,
            db: Some("ok"),
            vector_search: Some("available"),
            immich: Some("ok"),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["version"], "0.1.0");
        assert_eq!(json["api_version"], 1);
        assert_eq!(json["db"], "ok");
        assert_eq!(json["vector_search"], "available");
        assert_eq!(json["immich"], "ok");
    }

    #[test]
    fn test_health_response_omits_none_fields() {
        let resp = HealthResponse {
            status: "degraded",
            version: "0.1.0",
            api_version: 1,
            db: Some("unreachable"),
            vector_search: None,
            immich: None,
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["status"], "degraded");
        assert_eq!(json["db"], "unreachable");
        assert!(json.get("vector_search").is_none());
        assert!(json.get("immich").is_none());
    }

    #[test]
    fn test_health_response_with_immich_without_vector_search() {
        let resp = HealthResponse {
            status: "ok",
            version: "0.1.0",
            api_version: 1,
            db: Some("ok"),
            vector_search: None,
            immich: Some("unreachable"),
        };
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["immich"], "unreachable");
        assert!(json.get("vector_search").is_none());
    }
}
