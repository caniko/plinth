#![allow(clippy::result_large_err)]

pub mod actors;
pub mod api;
pub mod config;
pub mod db_helpers;
pub mod observability;
pub mod server_fns;
pub mod services;

use axum::extract::FromRef;
use kameo::actor::ActorRef;
use leptos::prelude::*;
use plinth_shared::SiteConfig;
use surrealdb::{Surreal, engine::local::Db};

use actors::content_cache::ContentCache;
use actors::vector_search::VectorSearch;
use config::PlinthConfig;

/// Configuration for connecting to an Immich instance (image proxy)
#[derive(Clone)]
pub struct ImmichConfig {
    pub base_url: String,
    pub api_key: String,
}

/// Application state that will be accessible in handlers
#[derive(Clone, FromRef)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub content_cache: ActorRef<ContentCache>,
    pub vector_search: ActorRef<VectorSearch>,
    pub db: Surreal<Db>,
    pub immich_config: Option<ImmichConfig>,
    pub http_client: reqwest::Client,
    pub config: PlinthConfig,
    pub site_config: SiteConfig,
}
