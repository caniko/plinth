pub mod api;
mod app;
pub mod components;
pub mod pages;

// Re-export App component
pub use app::App;

// Hydration entry point for WASM
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    // Better panic messages in the browser console
    console_error_panic_hook::set_once();

    // Mount the App component to the body
    leptos::mount::hydrate_body(App);
}
