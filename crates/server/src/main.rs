#![recursion_limit = "256"]

use std::net::SocketAddr;

use axum::{
    Router,
    http::{Request, header},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post, put},
};
use kameo::actor::Spawn;
use leptos::config::get_configuration;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_http::LatencyUnit;
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{error, info, warn};

use plinth_client::App;
use plinth_server::actors::core_cache::CoreCache;
use plinth_server::api::admin::auth_middleware;
use plinth_server::config::PlinthConfig;
use plinth_server::{AppState, ImmichConfig, api, observability, services::db};

/// Middleware that adds the `X-Plinth-API-Version` header to all responses.
async fn api_version_header(mut response: Response) -> Response {
    response.headers_mut().insert(
        "x-plinth-api-version",
        axum::http::HeaderValue::from(plinth_shared::API_VERSION),
    );
    response
}

/// Middleware that sets `Cache-Control` and `Vary` headers based on request path.
/// Handlers that already set `Cache-Control` (e.g. image proxy, feeds) are not overridden.
async fn cache_control_middleware(req: Request<axum::body::Body>, next: Next) -> Response {
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
    } else {
        // SSR HTML pages: no browser cache, 5 min CDN cache
        "public, max-age=0, s-maxage=300"
    };

    let headers = response.headers_mut();
    headers.insert(
        header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static(cache_value),
    );
    headers.insert(
        header::VARY,
        axum::http::HeaderValue::from_static("Accept-Encoding"),
    );

    response
}

/// Check if a path refers to a static public file (not /pkg/, not /api/).
fn is_static_file(path: &str) -> bool {
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

// Leptos SSR uses tokio::task::spawn_local for reactive effects/resources.
// LocalRuntime (unlike the standard Runtime) allows spawn_local from any spawned task.
fn main() {
    tokio::runtime::LocalRuntime::new()
        .expect("Failed to create LocalRuntime")
        .block_on(async_main());
}

async fn async_main() {
    // Load unified configuration (TOML file + env var overrides)
    let config = match PlinthConfig::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize observability from config
    let observability_config =
        observability::ObservabilityConfig::from_config(&config.observability);
    if let Err(e) = observability::init_observability(observability_config) {
        eprintln!("Failed to initialize observability: {}", e);
        std::process::exit(1);
    }

    info!("Starting Plinth server...");

    // Initialize Postgres from config
    let db = match db::init_db(&config.database).await {
        Ok(db) => {
            info!("Postgres initialized");
            db
        }
        Err(e) => {
            error!("Failed to initialize Postgres: {}", e);
            std::process::exit(1);
        }
    };

    // Run database migrations (core + all enabled bricks)
    use plinth_server::services::migrations;
    match migrations::run_migrations(&db).await {
        Ok(applied) => {
            if applied > 0 {
                info!(applied, "Database migrations applied");
            } else {
                info!("Database schema is up to date");
            }
        }
        Err(e) => {
            error!("Failed to run database migrations: {}", e);
            std::process::exit(1);
        }
    }

    // Seed sample data for development
    if let Err(e) = db::seed_sample_data(&db).await {
        warn!("Failed to seed sample data: {}", e);
    } else {
        info!("Sample data seeded");
    }

    // Load declarative articles from Nix store (if configured)
    #[cfg(feature = "brick-blog")]
    if let Some(ref content_dir) = config.content.content_dir {
        use plinth_server::services::declarative_content;
        match declarative_content::load_declarative_articles(&db, content_dir, &config).await {
            Ok(stats) => {
                info!(
                    inserted = stats.inserted,
                    updated = stats.updated,
                    deleted = stats.deleted,
                    skipped = stats.skipped,
                    "Declarative articles loaded"
                );
            }
            Err(e) => {
                error!("Failed to load declarative articles: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Spawn core cache actor (tags, site content)
    info!("Spawning actors...");
    let core_cache = CoreCache::spawn(CoreCache::new(db.clone()));
    let core_cache_ref = core_cache.clone();
    info!("CoreCache actor spawned");

    // Spawn brick-specific cache actors
    #[cfg(feature = "brick-blog")]
    let blog_cache = {
        use plinth_server::bricks::blog::cache::BlogCache;
        let cache = BlogCache::spawn(BlogCache::new(db.clone()));
        info!("BlogCache actor spawned");
        cache
    };

    #[cfg(feature = "brick-blog")]
    let vector_search = {
        use plinth_server::actors::vector_search::VectorSearch;
        match VectorSearch::new(db.clone(), config.content.vector_truncation) {
            Ok(vs) => {
                let actor_ref = VectorSearch::spawn(vs);
                info!("VectorSearch actor spawned");
                Some(actor_ref)
            }
            Err(e) => {
                warn!("VectorSearch disabled (semantic search unavailable): {}", e);
                None
            }
        }
    };

    // Backfill embeddings for declarative articles that lack them (background task)
    #[cfg(feature = "brick-blog")]
    if config.content.content_dir.is_some()
        && let Some(ref vs) = vector_search
    {
        let db_clone = db.clone();
        let vs_clone = vs.clone();
        let truncation = config.content.vector_truncation;
        tokio::task::spawn_local(async move {
            plinth_server::services::declarative_content::backfill_embeddings(
                db_clone, vs_clone, truncation,
            )
            .await;
        });
    }

    #[cfg(feature = "brick-portfolio")]
    let portfolio_cache = {
        use plinth_server::bricks::portfolio::cache::PortfolioCache;
        let cache = PortfolioCache::spawn(PortfolioCache::new(db.clone()));
        info!("PortfolioCache actor spawned");
        cache
    };

    #[cfg(feature = "brick-todo")]
    let todo_cache = {
        use plinth_server::bricks::todo::cache::TodoCache;
        let cache = TodoCache::spawn(TodoCache::new(db.clone()));
        info!("TodoCache actor spawned");
        cache
    };

    // Get Leptos configuration
    let conf = match get_configuration(None) {
        Ok(conf) => conf,
        Err(e) => {
            error!("Failed to load Leptos configuration: {}", e);
            std::process::exit(1);
        }
    };
    let mut leptos_options = conf.leptos_options;

    // Override with environment variables if set
    if let Ok(addr_str) = std::env::var("LEPTOS_SITE_ADDR")
        && let Ok(addr) = addr_str.parse()
    {
        leptos_options.site_addr = addr;
        info!("Using LEPTOS_SITE_ADDR from environment: {}", addr);
    }

    if let Ok(site_root) = std::env::var("LEPTOS_SITE_ROOT") {
        leptos_options.site_root = site_root.clone().into();
        info!("Using LEPTOS_SITE_ROOT from environment: {}", site_root);
    }

    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Build Immich config from unified config + env var for the secret
    let immich_api_key = std::env::var("IMMICH_API_KEY").ok();
    let immich_config = if !config.immich.api_url.is_empty() {
        if let Some(key) = immich_api_key {
            info!("Immich integration enabled: {}", config.immich.api_url);
            Some(ImmichConfig {
                base_url: config.immich.api_url.trim_end_matches('/').to_string(),
                api_key: key,
            })
        } else {
            warn!(
                "IMMICH_API_KEY not set — Immich integration disabled despite api_url being configured"
            );
            None
        }
    } else {
        info!("Immich integration disabled (api_url not configured)");
        None
    };

    let http_client = match reqwest::Client::builder().build() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to build HTTP client: {e}");
            std::process::exit(1);
        }
    };

    // Extract client-safe site config
    let site_config = config.to_site_config();
    let api_key = std::env::var("PLINTH_API_KEY").ok();
    if api_key.is_none() {
        warn!("PLINTH_API_KEY not set — admin API endpoints will reject all requests");
    }
    let site_lang = config.site.lang.clone();
    let site_theme = config.site.default_theme.clone();
    let plausible_domain = config.analytics.plausible_domain.clone();
    let plausible_script_url = config.analytics.plausible_script_url.clone();

    // Clone site_config before moving into AppState (shell closure needs it)
    let site_config_for_ssr = site_config.clone();

    // Build application state
    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        core_cache,
        db: db.clone(),
        immich_config,
        http_client,
        config,
        site_config,
        #[cfg(feature = "brick-blog")]
        blog_cache,
        #[cfg(feature = "brick-blog")]
        vector_search,
        #[cfg(feature = "brick-portfolio")]
        portfolio_cache,
        #[cfg(feature = "brick-todo")]
        todo_cache,
    };

    // Build admin API router — core routes (tags, site content)
    let mut admin_router = Router::new()
        .route("/admin/tags", get(api::admin::list_tags))
        .route(
            "/admin/content/{key}",
            put(api::admin::update_site_content).get(api::admin::get_admin_site_content),
        );

    // Merge brick-specific admin routes
    #[cfg(feature = "brick-blog")]
    {
        admin_router = admin_router
            .route(
                "/admin/articles",
                post(plinth_server::bricks::blog::admin::publish_article),
            )
            .route(
                "/admin/articles/{slug}",
                delete(plinth_server::bricks::blog::admin::delete_article),
            )
            .route(
                "/admin/posts/{post_slug}/tags",
                post(plinth_server::bricks::blog::admin::add_tag_to_post),
            )
            .route(
                "/admin/posts/{post_slug}/tags/{tag_slug}",
                delete(plinth_server::bricks::blog::admin::remove_tag_from_post),
            );
    }

    #[cfg(feature = "brick-todo")]
    {
        admin_router = admin_router
            .route(
                "/admin/todos",
                post(plinth_server::bricks::todo::admin::create_todo),
            )
            .route(
                "/admin/todos/{slug}",
                put(plinth_server::bricks::todo::admin::update_todo)
                    .delete(plinth_server::bricks::todo::admin::delete_todo),
            )
            .route(
                "/admin/todos/{todo_slug}/tags",
                post(plinth_server::bricks::todo::admin::add_tag_to_todo),
            )
            .route(
                "/admin/todos/{todo_slug}/tags/{tag_slug}",
                delete(plinth_server::bricks::todo::admin::remove_tag_from_todo),
            );
    }

    #[cfg(feature = "brick-portfolio")]
    {
        admin_router = admin_router.route(
            "/admin/portfolio",
            post(plinth_server::bricks::portfolio::admin::publish_portfolio_item),
        );
    }

    admin_router = admin_router.layer(middleware::from_fn_with_state(api_key, auth_middleware));

    // Rate limiter: ~60 requests per minute per IP for public API endpoints
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(60)
        .finish()
        .unwrap_or_else(|| {
            eprintln!("Failed to build rate limiter config");
            std::process::exit(1);
        });

    // Stricter rate limiter for admin endpoints: ~10 requests per minute per IP
    let admin_governor_conf = GovernorConfigBuilder::default()
        .per_second(6)
        .burst_size(10)
        .finish()
        .unwrap_or_else(|| {
            eprintln!("Failed to build admin rate limiter config");
            std::process::exit(1);
        });

    // Apply admin rate limiter after auth middleware
    let admin_router = admin_router
        .layer(GovernorLayer::new(admin_governor_conf))
        .with_state(app_state.clone());

    // Build public API router (health + image proxy + conditional search)
    let mut public_api_router = Router::new()
        .route("/health", get(api::health::health_check))
        .route("/images/{asset_id}", get(api::images::serve_image));

    #[cfg(feature = "brick-blog")]
    {
        public_api_router = public_api_router
            .route("/search", get(api::search::search_articles))
            .route(
                "/articles/{slug}/related",
                get(api::search::related_articles),
            )
            .route("/opinion", get(api::search::track_opinion));
    }

    #[cfg(feature = "brick-portfolio")]
    {
        public_api_router = public_api_router
            .route(
                "/portfolio",
                get(plinth_server::bricks::portfolio::api::list_portfolio_items),
            )
            .route(
                "/portfolio/{slug}",
                get(plinth_server::bricks::portfolio::api::get_portfolio_item),
            );
    }

    let public_api_router = public_api_router
        .layer(GovernorLayer::new(governor_conf))
        .with_state(app_state.clone());

    // Build feed routes (conditional on bricks)
    let mut feed_app = Router::new().route("/sitemap.xml", get(api::feeds::sitemap_xml));

    #[cfg(feature = "brick-blog")]
    {
        feed_app = feed_app
            .route("/feeds/blog.xml", get(api::feeds::blog_feed))
            .route("/feeds/series/{slug}", get(api::feeds::series_feed))
            .route("/feed.xml", get(api::feeds::blog_feed));
    }

    #[cfg(feature = "brick-portfolio")]
    {
        feed_app = feed_app.route("/feeds/projects.xml", get(api::feeds::projects_feed));
    }

    // Build Axum router with HTTP tracing
    let app = Router::new()
        // API routes
        .nest("/api", admin_router.merge(public_api_router))
        // RSS feed routes
        .merge(feed_app.with_state(app_state.clone()))
        // Serve Leptos routes with SSR
        // additional_context provides the database pool and SiteConfig for both
        // SSR page renders AND server function HTTP calls
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let db = db.clone();
                let site_config = site_config_for_ssr.clone();
                move || {
                    provide_context(db.clone());
                    provide_context(site_config.clone());
                }
            },
            {
                let leptos_options = leptos_options.clone();
                let site_lang = site_lang.clone();
                let site_theme = site_theme.clone();
                let plausible_domain = plausible_domain.clone();
                let plausible_script_url = plausible_script_url.clone();
                move || {
                    shell(
                        leptos_options.clone(),
                        site_lang.clone(),
                        site_theme.clone(),
                        plausible_domain.clone(),
                        plausible_script_url.clone(),
                    )
                }
            },
        )
        // Handle static files and error pages
        // Note: leptos_routes_with_context may not discover routes behind a <Suspense>,
        // so the fallback also needs to provide all SSR context (db + site_config).
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>({
            let site_lang = site_lang.clone();
            let site_theme = site_theme.clone();
            let plausible_domain = plausible_domain.clone();
            let plausible_script_url = plausible_script_url.clone();
            let site_config = site_config_for_ssr;
            let db = db.clone();
            move |options| {
                provide_context(db.clone());
                provide_context(site_config.clone());
                shell(
                    options,
                    site_lang.clone(),
                    site_theme.clone(),
                    plausible_domain.clone(),
                    plausible_script_url.clone(),
                )
            }
        }))
        .with_state(app_state)
        // Cache-Control headers based on request path
        .layer(axum::middleware::from_fn(cache_control_middleware))
        // Add API version header to all responses
        .layer(axum::middleware::map_response(api_version_header))
        // Security headers
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
            axum::http::HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::HeaderName::from_static("content-security-policy"),
            axum::http::HeaderValue::from_static(
                "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'",
            ),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::HeaderName::from_static("strict-transport-security"),
            axum::http::HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
        ))
        // Request body size limit (2MB)
        .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024))
        // HTTP compression (gzip + brotli)
        .layer(CompressionLayer::new())
        // Add HTTP request/response tracing
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(
                    DefaultOnResponse::new()
                        .latency_unit(LatencyUnit::Millis)
                        .include_headers(true),
                ),
        );

    info!("Server listening on http://{}", addr);

    // Setup graceful shutdown signal
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    // Spawn signal handler task
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C signal, initiating graceful shutdown...");
                let _ = shutdown_tx.send(());
            }
            Err(e) => {
                error!("Failed to listen for Ctrl+C signal: {}", e);
            }
        }
    });

    // Run the server with graceful shutdown
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };
    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        shutdown_rx.await.ok();
    });

    match tokio::time::timeout(std::time::Duration::from_secs(30), server).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            error!("Server error: {}", e);
            std::process::exit(1);
        }
        Err(_) => {
            warn!("Graceful shutdown timed out after 30s, forcing exit");
        }
    }

    // Stop actors gracefully before exiting
    info!("Server stopped, shutting down actors...");
    drop(core_cache_ref);
    observability::shutdown_observability();
    info!("Goodbye!");
}

/// Shell function for Leptos SSR rendering.
/// Context (SiteConfig, database pool) is provided by leptos_routes_with_context.
fn shell(
    options: LeptosOptions,
    lang: String,
    default_theme: String,
    plausible_domain: String,
    plausible_script_url: String,
) -> impl IntoView {
    use leptos::prelude::*;
    use leptos_meta::MetaTags;

    let theme_class = if default_theme == "light" { "" } else { "dark" };
    let theme_script = format!(
        "var t=localStorage.getItem('theme');if(t==='light'){{document.documentElement.classList.remove('dark')}}else if(!t&&'{}' === 'light'){{document.documentElement.classList.remove('dark')}};",
        default_theme
    );

    let plausible_enabled = !plausible_domain.is_empty() && !plausible_script_url.is_empty();

    view! {
        <!DOCTYPE html>
        <html lang={lang} class={theme_class}>
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="color-scheme" content="light dark"/>
                <meta name="darkreader-lock"/>
                <script>{theme_script}</script>
                {plausible_enabled.then(|| view! {
                    <script defer data-domain=plausible_domain src=plausible_script_url></script>
                })}
                <link rel="alternate" type_="application/rss+xml" title="Blog" href="/feeds/blog.xml"/>
                <link rel="alternate" type_="application/rss+xml" title="Projects" href="/feeds/projects.xml"/>
                <MetaTags/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
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
            .layer(middleware::from_fn_with_state(api_key, auth_middleware))
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
                    "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'",
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
