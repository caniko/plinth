#![recursion_limit = "256"]

// Imports used by integration tests (cfg(test) module below). Binary-only
// builds warn on these; the allow silences that without cfg-gating each one.
#[cfg_attr(not(test), allow(unused_imports))]
use axum::{
    Router,
    http::{Request, header},
    middleware as axum_middleware,
    routing::{get, post},
};
#[cfg_attr(not(test), allow(unused_imports))]
use plinth_server::api::admin::auth_middleware;
use tokio::runtime::LocalRuntime;
#[cfg_attr(not(test), allow(unused_imports))]
use tower_http::limit::RequestBodyLimitLayer;
#[cfg_attr(not(test), allow(unused_imports))]
use tower_http::set_header::SetResponseHeaderLayer;

mod middleware;
pub use middleware::*;

#[allow(dead_code)]
mod router;
mod setup;
mod shell;
pub use setup::*;

fn main() {
    LocalRuntime::new()
        .expect("Failed to create LocalRuntime")
        .block_on(async_main());
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::StatusCode;
    use tower::ServiceExt;

    // --- Auth middleware helpers & tests ---

    fn test_app(api_key: Option<String>) -> Router {
        Router::new()
            .route("/protected", get(|| async { "ok" }))
            .layer(axum_middleware::from_fn_with_state(
                api_key,
                auth_middleware,
            ))
    }

    #[tokio::test]
    async fn test_auth_middleware_valid_bearer() {
        let app = test_app(Some("test_secret_key".to_string()));
        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", "Bearer test_secret_key")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_auth_middleware_wrong_key() {
        let app = test_app(Some("test_secret_key".to_string()));
        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", "Bearer wrong_key")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_no_header() {
        let app = test_app(Some("test_secret_key".to_string()));
        let req = Request::builder()
            .uri("/protected")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_non_bearer_scheme() {
        let app = test_app(Some("test_secret_key".to_string()));
        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", "Basic dXNlcjpwYXNz")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_auth_middleware_no_key_configured() {
        let app = test_app(None);
        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", "Bearer anything")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // --- is_static_file tests ---

    #[test]
    fn test_static_file_svg() {
        assert!(is_static_file("/favicon.svg"));
    }

    #[test]
    fn test_static_file_png() {
        assert!(is_static_file("/images/photo.png"));
    }

    #[test]
    fn test_static_file_json() {
        assert!(is_static_file("/manifest.json"));
    }

    #[test]
    fn test_static_file_webmanifest() {
        assert!(is_static_file("/site.webmanifest"));
    }

    #[test]
    fn test_static_file_excludes_api_path() {
        assert!(!is_static_file("/api/images/foo.png"));
    }

    #[test]
    fn test_static_file_excludes_pkg_path() {
        assert!(!is_static_file("/pkg/plinth.js"));
    }

    #[test]
    fn test_static_file_no_extension() {
        assert!(!is_static_file("/posts/my-article"));
    }

    #[test]
    fn test_static_file_html_not_included() {
        assert!(!is_static_file("/page.html"));
    }

    #[test]
    fn test_static_file_js_not_included() {
        assert!(!is_static_file("/script.js"));
    }

    #[test]
    fn test_publish_static_html_routes() {
        assert!(is_publish_static_html_route("/about"));
        assert!(is_publish_static_html_route("/support"));
        assert!(is_publish_static_html_route("/posts"));
        assert!(is_publish_static_html_route("/posts/my-article"));
        assert!(is_publish_static_html_route("/posts/tag/rust"));
        assert!(is_publish_static_html_route("/series"));
        assert!(is_publish_static_html_route("/series/rust-series"));
        assert!(is_publish_static_html_route("/projects"));
        assert!(is_publish_static_html_route("/projects/plinth"));
    }

    #[test]
    fn test_dynamic_html_routes_not_publish_static() {
        assert!(!is_publish_static_html_route("/"));
        assert!(!is_publish_static_html_route("/activity"));
        assert!(!is_publish_static_html_route("/activity/1"));
        assert!(!is_publish_static_html_route("/todos"));
        assert!(!is_publish_static_html_route("/todos/tag/rust"));
        assert!(!is_publish_static_html_route("/todos/learn-rust"));
    }

    // --- cache_control_middleware tests ---

    fn cache_app() -> Router {
        Router::new()
            .route("/{*path}", get(|| async { "ok" }))
            .route("/", get(|| async { "ok" }))
            .layer(axum::middleware::from_fn(cache_control_middleware))
    }

    async fn get_cache_control(app: Router, uri: &str) -> String {
        let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        resp.headers()
            .get(header::CACHE_CONTROL)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }

    #[tokio::test]
    async fn test_cache_control_pkg_assets() {
        let val = get_cache_control(cache_app(), "/pkg/plinth-abc123.js").await;
        assert_eq!(val, "public, max-age=31536000, immutable");
    }

    #[tokio::test]
    async fn test_cache_control_admin_api() {
        let val = get_cache_control(cache_app(), "/api/admin/articles").await;
        assert_eq!(val, "private, no-store");
    }

    #[tokio::test]
    async fn test_cache_control_health() {
        let val = get_cache_control(cache_app(), "/api/health").await;
        assert_eq!(val, "no-cache");
    }

    #[tokio::test]
    async fn test_cache_control_search() {
        let val = get_cache_control(cache_app(), "/api/search?q=test").await;
        assert_eq!(val, "private, no-store");
    }

    #[tokio::test]
    async fn test_cache_control_opinion() {
        let val = get_cache_control(cache_app(), "/api/opinion").await;
        assert_eq!(val, "private, no-store");
    }

    #[tokio::test]
    async fn test_cache_control_related_articles() {
        let val = get_cache_control(cache_app(), "/api/articles/my-post/related").await;
        assert_eq!(val, "public, s-maxage=3600");
    }

    #[tokio::test]
    async fn test_cache_control_static_file() {
        let val = get_cache_control(cache_app(), "/favicon.svg").await;
        assert_eq!(val, "public, max-age=86400");
    }

    #[tokio::test]
    async fn test_cache_control_ssr_page() {
        let val = get_cache_control(cache_app(), "/posts/my-article").await;
        assert_eq!(val, "public, max-age=0, s-maxage=0, must-revalidate");
    }

    #[tokio::test]
    async fn test_cache_control_dynamic_ssr_page() {
        let val = get_cache_control(cache_app(), "/activity/1").await;
        assert_eq!(val, "public, max-age=0, s-maxage=300");
    }

    #[tokio::test]
    async fn test_cache_control_vary_header_set() {
        let app = cache_app();
        let req = Request::builder()
            .uri("/posts/foo")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(
            resp.headers().get(header::VARY).unwrap().to_str().unwrap(),
            "Accept-Encoding"
        );
    }

    #[tokio::test]
    async fn test_cache_control_does_not_override_handler() {
        let app = Router::new()
            .route(
                "/custom",
                get(|| async { ([(header::CACHE_CONTROL, "custom-value")], "ok") }),
            )
            .layer(axum::middleware::from_fn(cache_control_middleware));

        let req = Request::builder()
            .uri("/custom")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(
            resp.headers()
                .get(header::CACHE_CONTROL)
                .unwrap()
                .to_str()
                .unwrap(),
            "custom-value"
        );
    }

    // --- Security header tests ---

    fn security_headers_app() -> Router {
        Router::new()
            .route("/test", get(|| async { "ok" }))
            .layer(SetResponseHeaderLayer::overriding(
                header::X_FRAME_OPTIONS,
                axum::http::HeaderValue::from_static("DENY"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                header::X_CONTENT_TYPE_OPTIONS,
                axum::http::HeaderValue::from_static("nosniff"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                axum::http::HeaderName::from_static("referrer-policy"),
                axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                axum::http::HeaderName::from_static("permissions-policy"),
                axum::http::HeaderValue::from_static(
                    "camera=(), microphone=(), geolocation=()",
                ),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                axum::http::HeaderName::from_static("content-security-policy"),
                axum::http::HeaderValue::from_static(
                    "default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'",
                ),
            ))
            .layer(SetResponseHeaderLayer::overriding(
                axum::http::HeaderName::from_static("strict-transport-security"),
                axum::http::HeaderValue::from_static(
                    "max-age=63072000; includeSubDomains; preload",
                ),
            ))
    }

    #[tokio::test]
    async fn test_security_headers_present() {
        let app = security_headers_app();
        let req = Request::builder().uri("/test").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let h = resp.headers();

        assert_eq!(h.get("x-frame-options").unwrap(), "DENY");
        assert_eq!(h.get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(
            h.get("referrer-policy").unwrap(),
            "strict-origin-when-cross-origin"
        );
        assert_eq!(
            h.get("permissions-policy").unwrap(),
            "camera=(), microphone=(), geolocation=()"
        );
        assert!(
            h.get("content-security-policy")
                .unwrap()
                .to_str()
                .unwrap()
                .contains("default-src 'self'")
        );
        assert!(
            h.get("strict-transport-security")
                .unwrap()
                .to_str()
                .unwrap()
                .contains("max-age=63072000")
        );
    }

    // --- Request body limit tests ---

    #[tokio::test]
    async fn test_body_limit_rejects_oversized_request() {
        // Handler must extract the body for the limit to trigger
        let app = Router::new()
            .route("/upload", post(|_body: axum::body::Bytes| async { "ok" }))
            .layer(RequestBodyLimitLayer::new(1024)); // 1KB limit

        let big_body = vec![0u8; 2048]; // 2KB
        let req = Request::builder()
            .method("POST")
            .uri("/upload")
            .body(Body::from(big_body))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn test_body_limit_allows_small_request() {
        let app = Router::new()
            .route("/upload", post(|_body: axum::body::Bytes| async { "ok" }))
            .layer(RequestBodyLimitLayer::new(1024));

        let small_body = vec![0u8; 512]; // 512 bytes
        let req = Request::builder()
            .method("POST")
            .uri("/upload")
            .body(Body::from(small_body))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
