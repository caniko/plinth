//! Plinth Leptos client — a WASM/SSR web frontend.
//!
//! This crate compiles to both WASM (for client-side rendering and hydration)
//! and server-side rendered HTML (via the `ssr` feature). It provides the
//! reactive UI components and page definitions for the Plinth blog, portfolio,
//! TODO, and activity views, along with server function wrappers in [`api`].

/// Server function wrappers for fetching data from the Plinth backend.
pub mod api;
mod app;
/// Reusable UI components: header, footer, theme toggle, error display, etc.
pub mod components;
/// Page-level Leptos components (home, about, blog, portfolio, etc.).
pub mod pages;

/// Root [`App`] component — the top-level router and layout.
pub use app::App;

/// Invalidation helpers for SSR static route regeneration.
#[cfg(feature = "ssr")]
pub use app::{
    invalidate_blog_static_routes, invalidate_portfolio_static_routes,
    invalidate_site_content_static_routes,
};

/// WASM hydration entry point (islands mode).
///
/// Called by the Leptos framework after the server-rendered HTML is loaded
/// in the browser. Hydrates interactive islands independently.
#[cfg(all(target_arch = "wasm32", feature = "hydrate", not(feature = "csr")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    // Better panic messages in the browser console
    console_error_panic_hook::set_once();

    // Islands mode hydrates each interactive island independently.
    leptos::mount::hydrate_islands();
}

/// WASM client-side rendering entry point.
///
/// Mounts the [`App`] component into the DOM body from an empty static shell.
/// Used in pure CSR builds (no SSR).
#[cfg(all(target_arch = "wasm32", feature = "csr"))]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn mount() {
    // Better panic messages in the browser console
    console_error_panic_hook::set_once();

    // A pure CSR bundle owns rendering from an empty static shell.
    leptos::mount::mount_to_body(App);
}
