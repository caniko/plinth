#![allow(clippy::result_large_err)]

use axum::{
    extract::FromRef,
    http::{header, Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use kameo::actor::{ActorRef, Spawn};
use leptos::config::get_configuration;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use surrealdb::{engine::local::Db, Surreal};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::{error, info, warn};

// Import the App component from the client package
use client::App;

mod actors;
mod api;
mod observability;
mod server_fns;
mod services;

use actors::content_cache::ContentCache;
use actors::vector_search::VectorSearch;
use services::db;

/// Application state that will be accessible in handlers
#[derive(Clone, FromRef)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub content_cache: ActorRef<ContentCache>,
    pub vector_search: ActorRef<VectorSearch>,
    pub db: Surreal<Db>,
}

/// Authentication middleware to verify API key
async fn auth_middleware(
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get API key from environment variable
    let api_key = std::env::var("BLOG_API_KEY")
        .unwrap_or_else(|_| "dev_api_key_change_in_production".to_string());

    // Check Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    match auth_header {
        Some(header_value) if header_value.starts_with("Bearer ") => {
            let token = &header_value[7..]; // Remove "Bearer " prefix
            if token == api_key {
                Ok(next.run(req).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

#[tokio::main]
async fn main() {
    // Initialize observability (tracing + optional OTLP export)
    let observability_config = observability::ObservabilityConfig::default();
    if let Err(e) = observability::init_observability(observability_config) {
        eprintln!("Failed to initialize observability: {}", e);
        std::process::exit(1);
    }

    info!("🚀 Starting personal website server...");

    // Initialize SurrealDB
    let db = match db::init_db().await {
        Ok(db) => {
            info!("✅ SurrealDB initialized");
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
    info!("✅ Database schema initialized");

    // Seed sample data for development
    if let Err(e) = db::seed_sample_data(&db).await {
        warn!("Failed to seed sample data: {}", e);
    } else {
        info!("✅ Sample data seeded");
    }

    // Spawn Kameo actors
    info!("🎭 Spawning actors...");

    let content_cache = ContentCache::spawn(ContentCache::new(db.clone()));
    info!("   ✅ ContentCache actor spawned");

    let vector_search = match VectorSearch::new(db.clone()) {
        Ok(vs) => {
            let actor_ref = VectorSearch::spawn(vs);
            info!("   ✅ VectorSearch actor spawned");
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
    if let Ok(addr_str) = std::env::var("LEPTOS_SITE_ADDR") {
        if let Ok(addr) = addr_str.parse() {
            leptos_options.site_addr = addr;
            info!("Using LEPTOS_SITE_ADDR from environment: {}", addr);
        }
    }

    if let Ok(site_root) = std::env::var("LEPTOS_SITE_ROOT") {
        leptos_options.site_root = site_root.clone().into();
        info!("Using LEPTOS_SITE_ROOT from environment: {}", site_root);
    }

    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Build application state
    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        content_cache,
        vector_search,
        db: db.clone(),
    };

    // Build admin API router with authentication
    let admin_router = Router::new()
        .route("/admin/articles", post(api::admin::publish_article))
        .layer(middleware::from_fn(auth_middleware))
        .with_state(app_state.clone());

    // Build public API router (search endpoints)
    let public_api_router = Router::new()
        .route("/search", get(api::search::search_articles))
        .route(
            "/articles/{slug}/related",
            get(api::search::related_articles),
        )
        .route("/opinion", get(api::search::track_opinion))
        .with_state(app_state.clone());

    // Build Axum router with HTTP tracing
    let app = Router::new()
        // API routes
        .nest("/api", admin_router.merge(public_api_router))
        // Serve Leptos routes with SSR
        .leptos_routes(&app_state, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        // Handle server functions
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
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

    info!("🚀 Server listening on http://{}", addr);
    info!("   Open your browser and visit the site!");

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
    info!("Goodbye! 👋");
}

/// Shell function for Leptos SSR rendering
fn shell(options: LeptosOptions) -> impl IntoView {
    use leptos::prelude::*;

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}
