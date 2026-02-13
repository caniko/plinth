#![allow(clippy::result_large_err)]

pub mod actors;
pub mod api;
pub mod observability;
pub mod server_fns;
pub mod services;

use axum::extract::FromRef;
use kameo::actor::ActorRef;
use leptos::prelude::*;
use surrealdb::{engine::local::Db, Surreal};

use actors::content_cache::ContentCache;
use actors::vector_search::VectorSearch;

/// Application state that will be accessible in handlers
#[derive(Clone, FromRef)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub content_cache: ActorRef<ContentCache>,
    pub vector_search: ActorRef<VectorSearch>,
    pub db: Surreal<Db>,
}
