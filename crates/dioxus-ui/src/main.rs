// wasm-bindgen's externref lowering needs one explicit wasm-bindgen item in
// otherwise-valid applications. Without it, recent nightly Rust toolchains
// can emit an incomplete externref intrinsic set and the release packaging
// step fails with `failed to find intrinsics to enable clone_ref function`.
#[cfg(all(feature = "web", target_arch = "wasm32"))]
#[wasm_bindgen::prelude::wasm_bindgen]
unsafe extern "C" {}

#[cfg(not(feature = "server"))]
fn main() {
    dioxus::launch(plinth_web::App);
}

#[cfg(feature = "server")]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    tokio::task::LocalSet::new()
        .run_until(async {
            if let Err(error) = serve().await {
                eprintln!("Plinth Dioxus server failed: {error}");
                std::process::exit(1);
            }
        })
        .await;
}

/// Asset requests carry a file name or live under Dioxus' generated bundles.
#[cfg(feature = "server")]
fn is_static_asset_path(path: &str) -> bool {
    path.starts_with("/pkg/")
        || path.starts_with("/assets/")
        || path
            .rsplit('/')
            .next()
            .is_some_and(|name| name.contains('.'))
}

#[cfg(feature = "server")]
async fn serve() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use axum::{Extension, Router, body::Body, http::header, response::Response, routing::get};
    use dioxus::server::{DioxusRouterExt, FullstackState, ServeConfig};
    use plinth_server::{bootstrap, middleware};
    use plinth_web::page_cache;
    use tower_http::compression::CompressionLayer;
    use tower_http::limit::RequestBodyLimitLayer;
    use tower_http::set_header::SetResponseHeaderLayer;
    use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};

    async fn page_cache_middleware(
        Extension(cache): Extension<page_cache::PageCache>,
        request: axum::extract::Request<Body>,
        next: axum::middleware::Next,
    ) -> Response {
        let path = request.uri().path().to_string();
        let query = request.uri().query();
        let key = page_cache::PageKey::from_request(request.method().as_str(), &path, query);
        let cacheable =
            key.is_some() && page_cache::policy(&path) == page_cache::PagePolicy::CachedContent;

        // Static bundle assets are not rendered pages. Claiming a render for
        // them leaks the in-flight claim when the browser cancels the request
        // (a favicon or image cancelled by reload), and every later request
        // for that path then waits forever.
        if cacheable && is_static_asset_path(&path) {
            return next.run(request).await;
        }

        if let Some(ref key) = key
            && cacheable
            && let Some(body) = cache.get(key)
        {
            return Response::builder()
                .status(axum::http::StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
                .body(Body::from(body.to_vec()))
                .unwrap_or_else(|_| Response::new(Body::empty()));
        }

        let render_generation = if cacheable {
            match cache
                .claim_render(key.as_ref().expect("cacheable requests have keys"))
                .await
            {
                Ok(generation) => Some(generation),
                Err(body) => {
                    return Response::builder()
                        .status(axum::http::StatusCode::OK)
                        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
                        .body(Body::from(body.to_vec()))
                        .unwrap_or_else(|_| Response::new(Body::empty()));
                }
            }
        } else {
            None
        };

        let response = next.run(request).await;
        if !cacheable
            || response.status() != axum::http::StatusCode::OK
            || !response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.starts_with("text/html"))
        {
            if render_generation.is_some() {
                cache
                    .release_render(key.as_ref().expect("cacheable requests have keys"))
                    .await;
            }
            return response;
        }

        let (parts, body) = response.into_parts();
        let body = match axum::body::to_bytes(body, 4 * 1024 * 1024).await {
            Ok(body) => body,
            Err(_) => {
                if render_generation.is_some() {
                    cache
                        .release_render(key.as_ref().expect("cacheable requests have keys"))
                        .await;
                }
                return Response::from_parts(parts, Body::empty());
            }
        };
        if let (Some(cache_key), Some(generation)) = (key.clone(), render_generation) {
            cache.put_if_generation(
                generation,
                cache_key,
                std::sync::Arc::<[u8]>::from(body.to_vec()),
                vec![path],
            );
        }
        if render_generation.is_some() {
            cache
                .release_render(key.as_ref().expect("cacheable requests have keys"))
                .await;
        }
        Response::from_parts(parts, Body::from(body))
    }

    let backend = bootstrap::initialize().await;
    let addr = backend.site_addr;
    let state = FullstackState::new(
        // Streaming stays off (the default): the server waits for every loader,
        // so pages carry their content and status code without JavaScript, and
        // the route-level suspense fallback is only seen during client
        // navigation. Loaders read `AppState` from the request extension.
        ServeConfig::new(),
        plinth_web::App,
    );
    // Keep the stable API state and Dioxus' fullstack state in separate routers;
    // each is resolved before merging so the final service has no unresolved
    // Axum state parameter.
    let api =
        plinth_server::router::build_api_router(backend.api_key).with_state(backend.state.clone());
    let pages = Router::<FullstackState>::new()
        // Clients that ask for the legacy icon blindly get the PNG favicon.
        .route(
            "/favicon.ico",
            get(|| async { axum::response::Redirect::permanent("/favicon-32x32.png") }),
        )
        .register_server_functions()
        .serve_static_assets()
        .fallback(get(FullstackState::render_handler))
        .with_state(state);

    let page_cache = page_cache::PageCache::from_env(std::time::Duration::from_secs(300), 512);
    let invalidation_cache = page_cache.clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(std::time::Duration::from_millis(250));
        loop {
            tick.tick().await;
            if !plinth_server::page_cache::drain().is_empty() {
                // The event payload is intentionally narrow at the write
                // boundary; clearing the complete rendered-page set here keeps
                // correctness while the external tag index is introduced.
                invalidation_cache.clear();
            }
        }
    });

    let app = api
        .merge(pages)
        .layer(axum::middleware::from_fn(page_cache_middleware))
        // Axum applies the last layer outermost. The cache middleware extracts
        // PageCache, so its Extension provider must be added after it.
        .layer(Extension(page_cache))
        // Server functions called over HTTP run outside the render runtime,
        // so loaders read the backend from the request, not Dioxus context.
        .layer(Extension(backend.state.clone()))
        .layer(axum::middleware::from_fn(middleware::cache_control_middleware))
        .layer(axum::middleware::map_response(middleware::api_version_header))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_FRAME_OPTIONS,
            axum::http::HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            axum::http::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::HeaderName::from_static("referrer-policy"),
            axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::HeaderName::from_static("permissions-policy"),
            axum::http::HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::HeaderName::from_static("content-security-policy"),
            axum::http::HeaderValue::from_static(
                "default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'",
            ),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::HeaderName::from_static("strict-transport-security"),
            axum::http::HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
        ))
        .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024))
        .layer(CompressionLayer::new())
        // Request headers are intentionally excluded: the admin Authorization
        // token must never enter logs or OTLP exports.
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(false))
                .on_response(
                    DefaultOnResponse::new()
                        .latency_unit(tower_http::LatencyUnit::Millis)
                        .include_headers(false),
                ),
        );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "Plinth Dioxus server listening");
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            let _ = shutdown_tx.send(());
        }
    });

    let result = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let _ = shutdown_rx.await;
    })
    .await;

    plinth_server::observability::shutdown_observability();
    result?;
    Ok(())
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::is_static_asset_path;

    #[test]
    fn static_asset_paths_never_enter_the_page_cache() {
        assert!(is_static_asset_path("/favicon.svg"));
        assert!(is_static_asset_path("/plinth.css"));
        assert!(is_static_asset_path("/pkg/plinth_bg.wasm"));
        assert!(is_static_asset_path("/assets/tartan-ui-dxh1.css"));
        assert!(!is_static_asset_path("/"));
        assert!(!is_static_asset_path("/posts/hello-world"));
    }
}
