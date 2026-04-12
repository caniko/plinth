//! Brick system — modular, optional feature modules for Plinth.
//!
//! Each "brick" is a self-contained content type (blog, portfolio, todo) that
//! provides its own migrations, routes, cache actors, and admin endpoints.
//! Bricks are gated by Cargo feature flags (`brick-blog`, `brick-portfolio`,
//! `brick-todo`) and composed at startup.

use std::future::Future;
use std::pin::Pin;

use axum::Router;
use surrealdb::Surreal;
use surrealdb::engine::local::Db;

use crate::AppState;

/// A database migration owned by a specific brick.
///
/// Migrations are namespaced by `(brick, version)` so each brick manages its
/// own schema independently. The migration runner tracks applied migrations
/// per brick.
pub struct BrickMigration {
    pub brick: &'static str,
    pub version: u32,
    pub name: &'static str,
    pub up: &'static str,
}

/// A sitemap entry contributed by a brick.
pub struct SitemapEntry {
    pub loc: String,
    pub lastmod: Option<String>,
    pub changefreq: Option<String>,
    pub priority: Option<f32>,
}

/// Trait that every brick implements on the server side.
///
/// Called during startup to register migrations, routes, and seed data.
/// All methods except `name()` have default no-op implementations.
pub trait Brick: Send + Sync + 'static {
    /// Unique identifier for this brick (e.g., "blog", "todo", "portfolio").
    fn name(&self) -> &'static str;

    /// SQL migrations specific to this brick.
    /// Each migration is namespaced: `(brick_name, version)`.
    fn migrations(&self) -> Vec<BrickMigration> {
        vec![]
    }

    /// Server-side Axum routes to merge into the public API router.
    fn public_routes(&self, _state: &AppState) -> Option<Router<AppState>> {
        None
    }

    /// Server-side Axum routes to merge into the admin API router (behind auth).
    fn admin_routes(&self, _state: &AppState) -> Option<Router<AppState>> {
        None
    }

    /// Feed routes (e.g., /feeds/blog.xml). Separate because they live outside /api.
    fn feed_routes(&self, _state: &AppState) -> Option<Router<AppState>> {
        None
    }

    /// Sitemap URL entries this brick contributes.
    fn sitemap_entries(
        &self,
        _db: &Surreal<Db>,
    ) -> Pin<Box<dyn Future<Output = Vec<SitemapEntry>> + Send + '_>> {
        Box::pin(async { vec![] })
    }

    /// Seed sample data for development.
    fn seed_data(
        &self,
        _db: &Surreal<Db>,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        Box::pin(async { Ok(()) })
    }
}

// Brick implementations, gated by feature flags

#[cfg(feature = "brick-blog")]
pub mod blog;

#[cfg(feature = "brick-portfolio")]
pub mod portfolio;

#[cfg(feature = "brick-todo")]
pub mod todo;

/// Collect all enabled bricks into a Vec for startup composition.
#[allow(clippy::vec_init_then_push)]
pub fn enabled_bricks() -> Vec<Box<dyn Brick>> {
    let mut bricks: Vec<Box<dyn Brick>> = Vec::new();

    #[cfg(feature = "brick-blog")]
    bricks.push(Box::new(blog::BlogBrick));

    #[cfg(feature = "brick-portfolio")]
    bricks.push(Box::new(portfolio::PortfolioBrick));

    #[cfg(feature = "brick-todo")]
    bricks.push(Box::new(todo::TodoBrick));

    bricks
}
