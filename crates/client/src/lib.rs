//! Plinth Leptos client — a WASM/SSR web frontend.
//!
//! This crate compiles to both WASM (for client-side rendering and hydration)
//! and server-side rendered HTML (via the `ssr` feature). It provides the
//! reactive UI components and page definitions for the Plinth blog, portfolio,
//! TODO, and activity views, along with server function wrappers in [`api`].

pub mod api;
mod app;
pub mod components;
pub mod pages;

// Re-export App component
pub use app::App;

#[cfg(feature = "ssr")]
pub use app::{
    invalidate_blog_static_routes, invalidate_portfolio_static_routes,
    invalidate_site_content_static_routes,
};

// Hydration entry point for WASM
#[cfg(all(target_arch = "wasm32", feature = "hydrate", not(feature = "csr")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    // Better panic messages in the browser console
    console_error_panic_hook::set_once();

    // Islands mode hydrates each interactive island independently.
    leptos::mount::hydrate_islands();
}

// CSR entry point for static/client-only builds.
#[cfg(all(target_arch = "wasm32", feature = "csr"))]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn mount() {
    // Better panic messages in the browser console
    console_error_panic_hook::set_once();

    // A pure CSR bundle owns rendering from an empty static shell.
    leptos::mount::mount_to_body(App);
}
