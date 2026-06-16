//! Plinth HTTP server — Axum-based web application serving the Plinth CMS.
//!
//! This crate implements the server-side of Plinth: HTTP API endpoints
//! (public, admin, feeds, search, images), a Kameo actor-based caching layer,
//! content bricks (blog, portfolio, TODO, activity), database migrations,
//! OpenTelemetry observability, and authentication middleware.

#![allow(clippy::result_large_err)]

pub mod actors;
pub mod api;
pub mod bricks;
pub mod config;
pub mod error;
pub mod observability;
pub mod services;

use kameo::actor::ActorRef;
use leptos::prelude::*;
use plinth_shared::SiteConfig;

/// Shared database pool handle.
pub type PlinthDb = services::db::Db;

use actors::core_cache::CoreCache;
use config::PlinthConfig;

/// Configuration for connecting to an Immich instance (image proxy)
#[derive(Clone)]
pub struct ImmichConfig {
    pub base_url: String,
    pub api_key: String,
}

/// Application state that will be accessible in handlers.
///
/// Brick-specific actors are feature-gated — only present when the
/// corresponding brick feature is enabled. `#[derive(FromRef)]` is not
/// used because it doesn't support `#[cfg]` attributes on fields.
#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub core_cache: ActorRef<CoreCache>,
    pub db: PlinthDb,
    pub immich_config: Option<ImmichConfig>,
    pub http_client: reqwest::Client,
    pub config: PlinthConfig,
    pub site_config: SiteConfig,

    // Brick-specific actors
    #[cfg(feature = "brick-blog")]
    pub blog_cache: ActorRef<bricks::blog::cache::BlogCache>,
    #[cfg(feature = "brick-blog")]
    pub vector_search: Option<ActorRef<actors::vector_search::VectorSearch>>,
    #[cfg(feature = "brick-portfolio")]
    pub portfolio_cache: ActorRef<bricks::portfolio::cache::PortfolioCache>,
    #[cfg(feature = "brick-activity")]
    pub activity_cache: ActorRef<bricks::activity::cache::ActivityCache>,
    #[cfg(feature = "brick-todo")]
    pub todo_cache: ActorRef<bricks::todo::cache::TodoCache>,
}

// Manual FromRef implementations for types that Axum extractors need.
// The derive macro doesn't support #[cfg] attributes on fields.

impl axum::extract::FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}
