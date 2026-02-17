#![recursion_limit = "256"]

use axum::{
    Router,
    extract::State,
    http::{Request, StatusCode, header},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post, put},
};
use kameo::actor::Spawn;
use leptos::config::get_configuration;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{error, info, warn};

use plinth_client::App;
use plinth_server::actors::content_cache::ContentCache;
#[allow(unused_imports)]
use plinth_server::actors::content_cache::{GetAllBlogPosts, GetAllPortfolioItems, GetAllTodos};
use plinth_server::actors::vector_search::VectorSearch;
use plinth_server::config::PlinthConfig;
use plinth_server::{AppState, ImmichConfig, api, observability, services::db};

/// Authentication middleware to verify API key.
/// The API key is read exclusively from the `PLINTH_API_KEY` environment variable.
async fn auth_middleware(
    State(api_key): State<Option<String>>,
    req: Request<axum::body::Body>,
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
            if token == expected_key {
                Ok(next.run(req).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
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

    // Initialize SurrealDB from config
    let db = match db::init_db(&config.database).await {
        Ok(db) => {
            info!("SurrealDB initialized");
            db
        }
        Err(e) => {
            error!("Failed to initialize SurrealDB: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize database schema
    if let Err(e) = db::init_schema(&db).await {
        error!("Failed to initialize database schema: {}", e);
        std::process::exit(1);
    }
    info!("Database schema initialized");

    // Seed sample data for development
    if let Err(e) = db::seed_sample_data(&db).await {
        warn!("Failed to seed sample data: {}", e);
    } else {
        info!("Sample data seeded");
    }

    // Spawn Kameo actors
    info!("Spawning actors...");

    let content_cache = ContentCache::spawn(ContentCache::new(db.clone()));
    info!("ContentCache actor spawned");

    let vector_search = match VectorSearch::new(db.clone(), config.content.vector_truncation) {
        Ok(vs) => {
            let actor_ref = VectorSearch::spawn(vs);
            info!("VectorSearch actor spawned");
            actor_ref
        }
        Err(e) => {
            error!("Failed to initialize VectorSearch actor: {}", e);
            std::process::exit(1);
        }
    };

    // Get Leptos configuration
    let conf = get_configuration(None).unwrap();
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

    let http_client = reqwest::Client::builder()
        .build()
        .expect("Failed to build HTTP client");

    // Extract client-safe site config
    let site_config = config.to_site_config();
    let api_key = std::env::var("PLINTH_API_KEY").ok();
    if api_key.is_none() {
        warn!("PLINTH_API_KEY not set — admin API endpoints will reject all requests");
    }
    let site_lang = config.site.lang.clone();
    let site_theme = config.site.default_theme.clone();

    // Clone site_config before moving into AppState (shell closure needs it)
    let site_config_for_ssr = site_config.clone();

    // Build application state
    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        content_cache,
        vector_search,
        db: db.clone(),
        immich_config,
        http_client,
        config,
        site_config,
    };

    // Build admin API router with authentication
    let admin_router = Router::new()
        .route("/admin/articles", post(api::admin::publish_article))
        .route("/admin/tags", get(api::admin::list_tags))
        .route(
            "/admin/posts/{post_slug}/tags",
            post(api::admin::add_tag_to_post),
        )
        .route(
            "/admin/posts/{post_slug}/tags/{tag_slug}",
            delete(api::admin::remove_tag_from_post),
        )
        .route(
            "/admin/content/{key}",
            put(api::admin::update_site_content).get(api::admin::get_admin_site_content),
        )
        .route("/admin/todos", post(api::admin::create_todo))
        .route(
            "/admin/todos/{slug}",
            put(api::admin::update_todo).delete(api::admin::delete_todo),
        )
        .route(
            "/admin/todos/{todo_slug}/tags",
            post(api::admin::add_tag_to_todo),
        )
        .route(
            "/admin/todos/{todo_slug}/tags/{tag_slug}",
            delete(api::admin::remove_tag_from_todo),
        )
        .layer(middleware::from_fn_with_state(api_key, auth_middleware))
        .with_state(app_state.clone());

    // Build public API router (search + image proxy)
    let public_api_router = Router::new()
        .route("/search", get(api::search::search_articles))
        .route(
            "/articles/{slug}/related",
            get(api::search::related_articles),
        )
        .route("/opinion", get(api::search::track_opinion))
        .route("/images/{asset_id}", get(api::images::serve_image))
        .with_state(app_state.clone());

    // Build Axum router with HTTP tracing
    let app = Router::new()
        // API routes
        .nest("/api", admin_router.merge(public_api_router))
        // RSS feed routes
        .route("/feeds/blog.xml", get(api::feeds::blog_feed))
        .route("/feeds/projects.xml", get(api::feeds::projects_feed))
        .route("/feed.xml", get(api::feeds::blog_feed))
        // Serve Leptos routes with SSR
        // additional_context provides Surreal<Db> and SiteConfig for both
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
                move || {
                    shell(
                        leptos_options.clone(),
                        site_lang.clone(),
                        site_theme.clone(),
                    )
                }
            },
        )
        // Handle static files and error pages
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>({
            let site_lang = site_lang.clone();
            let site_theme = site_theme.clone();
            let site_config = site_config_for_ssr;
            move |options| {
                provide_context(site_config.clone());
                shell(options, site_lang.clone(), site_theme.clone())
            }
        }))
        .with_state(app_state)
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
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        })
        .await
        .unwrap();

    // Shutdown observability and flush telemetry
    info!("Server stopped, cleaning up...");
    observability::shutdown_observability();
    info!("Goodbye!");
}

/// Shell function for Leptos SSR rendering.
/// Context (SiteConfig, Surreal<Db>) is provided by leptos_routes_with_context.
fn shell(options: LeptosOptions, lang: String, default_theme: String) -> impl IntoView {
    use leptos::prelude::*;
    use leptos_meta::MetaTags;

    let theme_class = if default_theme == "light" { "" } else { "dark" };
    let theme_script = format!(
        "var t=localStorage.getItem('theme');if(t==='light'){{document.documentElement.classList.remove('dark')}}else if(!t&&'{}' === 'light'){{document.documentElement.classList.remove('dark')}};",
        default_theme
    );

    view! {
        <!DOCTYPE html>
        <html lang={lang} class={theme_class}>
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="color-scheme" content="light dark"/>
                <meta name="darkreader-lock"/>
                <script>{theme_script}</script>
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
    use tower::ServiceExt;

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
}
