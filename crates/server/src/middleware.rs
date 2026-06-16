use axum::{
    body::Body,
    http::{HeaderValue, Request, header},
    middleware::Next,
    response::Response,
};

use plinth_shared::API_VERSION;

/// Middleware that adds the `X-Plinth-API-Version` header to all responses.
pub async fn api_version_header(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert("x-plinth-api-version", HeaderValue::from(API_VERSION));
    response
}

/// Middleware that sets `Cache-Control` and `Vary` headers based on request path.
/// Handlers that already set `Cache-Control` (e.g. image proxy, feeds) are not overridden.
pub async fn cache_control_middleware(req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path().to_owned();
    let mut response = next.run(req).await;

    // Don't override Cache-Control if the handler already set it
    if response.headers().contains_key(header::CACHE_CONTROL) {
        return response;
    }

    let cache_value = if path.starts_with("/pkg/") {
        // Leptos hashed assets: immutable forever
        "public, max-age=31536000, immutable"
    } else if path.starts_with("/api/admin/") {
        "private, no-store"
    } else if path.starts_with("/api/health") {
        "no-cache"
    } else if path.starts_with("/api/search") || path.starts_with("/api/opinion") {
        "private, no-store"
    } else if path.starts_with("/api/articles/") && path.contains("/related") {
        "public, s-maxage=3600"
    } else if is_static_file(&path) {
        "public, max-age=86400"
    } else if is_publish_static_html_route(&path) {
        "public, max-age=0, s-maxage=0, must-revalidate"
    } else {
        // SSR HTML pages: no browser cache, 5 min CDN cache
        "public, max-age=0, s-maxage=300"
    };

    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(cache_value));
    headers.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));

    response
}

/// Check if a path refers to a static public file (not /pkg/, not /api/).
pub fn is_static_file(path: &str) -> bool {
    const STATIC_EXTENSIONS: &[&str] = &[
        ".svg",
        ".png",
        ".ico",
        ".jpg",
        ".jpeg",
        ".webp",
        ".woff",
        ".woff2",
        ".ttf",
        ".txt",
        ".xml",
        ".json",
        ".webmanifest",
    ];
    STATIC_EXTENSIONS.iter().any(|ext| path.ends_with(ext))
        && !path.starts_with("/api/")
        && !path.starts_with("/pkg/")
}

/// Publish-cadence HTML routes rendered with `SsrMode::Static`.
pub fn is_publish_static_html_route(path: &str) -> bool {
    if matches!(
        path,
        "/about" | "/support" | "/posts" | "/series" | "/projects"
    ) {
        return true;
    }

    let parts = path.trim_start_matches('/').split('/').collect::<Vec<_>>();

    matches!(
        parts.as_slice(),
        ["posts", _] | ["posts", "tag", _] | ["series", _] | ["projects", _]
    )
}
