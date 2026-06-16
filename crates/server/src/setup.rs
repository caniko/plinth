use std::net::SocketAddr;

use axum::{Router, http::header};
use kameo::actor::Spawn;
use leptos::config::get_configuration;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list_with_exclusions_and_ssg_and_context};
use tower_http::LatencyUnit;
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{error, info, warn};

use plinth_server::actors::core_cache::CoreCache;
use plinth_server::config::PlinthConfig;
use plinth_server::{AppState, ImmichConfig, observability, services::db};

use crate::{api_version_header, cache_control_middleware};

/// Plinth server entry point — config, DB, actors, routes, and signal handling.
pub async fn async_main() {
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
            // Log on completion so a task that vanished (e.g. panicked) is
            // detectable by the absence of this line.
            info!("Embedding backfill task finished");
        });
    }

    #[cfg(feature = "brick-portfolio")]
    let portfolio_cache = {
        use plinth_server::bricks::portfolio::cache::PortfolioCache;
        let cache = PortfolioCache::spawn(PortfolioCache::new(db.clone()));
        info!("PortfolioCache actor spawned");
        cache
    };

    #[cfg(feature = "brick-activity")]
    let activity_cache = {
        use plinth_forge::{CodebergClient, ForgeClient, ForgeRouter, GitHubClient};
        use plinth_server::bricks::activity::cache::ActivityCache;
        use std::sync::Arc;
        let forge = config.forge.clone();
        let github_token = std::env::var("GITHUB_TOKEN").ok();
        let codeberg_token = std::env::var("CODEBERG_TOKEN").ok();
        let router = ForgeRouter {
            github: GitHubClient::with_base_url(forge.github_base_url.clone(), github_token),
            codeberg: CodebergClient::with_base_url(
                forge.codeberg_base_url.clone(),
                codeberg_token,
            ),
        };
        let forge_client: Arc<dyn ForgeClient + Send + Sync> = Arc::new(router);
        let cache = ActivityCache::spawn(ActivityCache::new(
            db.clone(),
            config.ranking.clone(),
            forge,
            forge_client,
        ));
        info!("ActivityCache actor spawned");
        cache
    };

    // Type-erased refresh hook handed to the SSR read path (via Leptos context) so
    // a page visit can trigger the cache actor's stale-while-revalidate refresh.
    #[cfg(feature = "brick-activity")]
    let activity_refresh_hook: std::sync::Arc<dyn plinth_shared::ActivityRefreshHook> =
        std::sync::Arc::new(
            plinth_server::bricks::activity::cache::ActivityRefreshHandle(activity_cache.clone()),
        );

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
            error!("Failed to build HTTP client: {e}");
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
    let addr = leptos_options.site_addr;
    let route_context = {
        let db = db.clone();
        let site_config = site_config_for_ssr.clone();
        #[cfg(feature = "brick-activity")]
        let activity_refresh_hook = activity_refresh_hook.clone();
        move || {
            provide_context(db.clone());
            provide_context(site_config.clone());
            #[cfg(feature = "brick-activity")]
            provide_context(activity_refresh_hook.clone());
        }
    };
    let ssr_shell = {
        let leptos_options = leptos_options.clone();
        let site_lang = site_lang.clone();
        let site_theme = site_theme.clone();
        let plausible_domain = plausible_domain.clone();
        let plausible_script_url = plausible_script_url.clone();
        move || {
            crate::shell::shell(
                leptos_options.clone(),
                site_lang.clone(),
                site_theme.clone(),
                plausible_domain.clone(),
                plausible_script_url.clone(),
            )
        }
    };
    let (routes, _static_route_generator) = generate_route_list_with_exclusions_and_ssg_and_context(
        ssr_shell.clone(),
        None,
        route_context.clone(),
    );
    for route in &routes {
        info!(
            path = route.path(),
            mode = ?route.mode(),
            "Registered Leptos route"
        );
    }
    // `StaticRouteGenerator::generate` eagerly renders every static route at
    // startup, but in Leptos 0.8.8 that path injects metadata before the shell's
    // closing `</head>` is available for this app. The Axum static route handler
    // still generates missing files on first request and subscribes to each
    // route's regeneration stream.

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
        #[cfg(feature = "brick-activity")]
        activity_cache,
        #[cfg(feature = "brick-todo")]
        todo_cache,
    };

    // Build admin API router (core + brick-specific routes)
    let admin_router = crate::router::build_admin_router(api_key);

    // Build public API router (health + image proxy + brick-specific routes)
    let public_api_router = crate::router::build_public_api_router();

    // Build feed routes (sitemap + brick-specific feeds)
    let feed_app = crate::router::build_feed_router();

    // Build Axum router with HTTP tracing
    let app = Router::<AppState>::new()
        // API routes
        .nest("/api", admin_router.merge(public_api_router))
        // RSS feed routes
        .merge(feed_app)
        // Serve Leptos routes with SSR
        // additional_context provides the database pool and SiteConfig for both
        // SSR page renders AND server function HTTP calls
        .leptos_routes_with_context(
            &app_state,
            routes,
            route_context,
            ssr_shell,
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
            #[cfg(feature = "brick-activity")]
            let activity_refresh_hook = activity_refresh_hook.clone();
            move |options| {
                provide_context(db.clone());
                provide_context(site_config.clone());
                #[cfg(feature = "brick-activity")]
                provide_context(activity_refresh_hook.clone());
                crate::shell::shell(
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
                "default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'",
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
        // Add HTTP request/response tracing.
        // Headers are intentionally NOT captured: request headers include the
        // admin `Authorization: Bearer <PLINTH_API_KEY>` token, which must not
        // be written to logs or exported via OTLP.
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(false))
                .on_response(
                    DefaultOnResponse::new()
                        .latency_unit(LatencyUnit::Millis)
                        .include_headers(false),
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

    // Serve until the shutdown signal fires and in-flight connections drain.
    // (Previously this future was wrapped in a 30s `timeout`, which force-exited
    // the server 30 seconds after startup during normal operation.)
    if let Err(e) = server.await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }

    // Stop actors gracefully before exiting
    info!("Server stopped, shutting down actors...");
    drop(core_cache_ref);
    observability::shutdown_observability();
    info!("Goodbye!");
}
