use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use plinth_shared::SiteConfig;

use crate::components::{Footer, Header};

#[cfg(any(feature = "csr", not(feature = "islands")))]
use crate::api;

mod invalidation;
mod routes;

// Re-export invalidation functions so lib.rs can use app::invalidate_*
#[cfg(feature = "ssr")]
pub use invalidation::{
    invalidate_blog_static_routes, invalidate_portfolio_static_routes,
    invalidate_site_content_static_routes,
};

/// Rendering-mode source of truth for the default all-bricks route table.
///
/// Static routes are publish-cadence content regenerated through Leptos
/// `StaticRoute::regenerate` streams when their admin publish/update path runs.
/// Request-time routes intentionally remain dynamic.
#[allow(dead_code)]
pub const ROUTE_RENDERING_MODES: &[(&str, &str)] = &[
    (
        "/",
        "SsrMode::OutOfOrder (streaming SSR; Phase 03 owns home)",
    ),
    ("/about", "SsrMode::Static (site content key: about)"),
    ("/support", "SsrMode::Static (site content key: support)"),
    ("/posts", "SsrMode::Static (blog index)"),
    ("/posts/:slug", "SsrMode::Static (blog post slug)"),
    ("/posts/tag/:tag", "SsrMode::Static (blog tag name/slug)"),
    ("/series", "SsrMode::Static (series index)"),
    ("/series/:slug", "SsrMode::Static (series slug)"),
    ("/projects", "SsrMode::Static (portfolio index)"),
    ("/projects/:slug", "SsrMode::Static (portfolio slug)"),
    ("/activity", "SsrMode::OutOfOrder (request-time SSR)"),
    ("/activity/:id", "SsrMode::OutOfOrder (request-time SSR)"),
    ("/todos", "SsrMode::OutOfOrder (request-time SSR)"),
    ("/todos/tag/:tag", "SsrMode::OutOfOrder (request-time SSR)"),
    ("/todos/:slug", "SsrMode::OutOfOrder (request-time SSR)"),
];

/// Helper to get site config from context (falls back to defaults).
///
/// On SSR, the server provides `SiteConfig` via `provide_context`.
/// On the client, the App component provides it after hydrating from the
/// serialized Resource data.
pub fn use_site_config() -> SiteConfig {
    use_context::<SiteConfig>().unwrap_or_default()
}

/// Root application component.
///
/// Sets up meta context, site config provider, favicon links, stylesheet,
/// htmx script, and the [`Router`] with [`Header`] / [`Footer`] layout.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let initial_config = use_context::<SiteConfig>().unwrap_or_default();
    let favicon = initial_config.favicon.clone();
    provide_context(initial_config);

    #[cfg(any(feature = "csr", not(feature = "islands")))]
    let config_provider = {
        let site_config = Resource::new(|| (), |_| api::get_site_config());

        view! {
            <Suspense fallback=|| ()>
                {move || site_config.get().map(|result| {
                    let config = result.unwrap_or_default();
                    provide_context(config);
                })}
            </Suspense>
        }
    };

    #[cfg(all(not(feature = "csr"), feature = "islands"))]
    let config_provider = ();

    view! {
        // Meta tags
        <Stylesheet id="leptos" href="/pkg/plinth.css"/>
        {if let Some(path) = &favicon {
            view! { <Link rel="icon" type_="image/svg+xml" href={path.clone()}/> }.into_any()
        } else {
            view! {
                <Link rel="icon" type_="image/svg+xml" href="/favicon.svg"/>
                <Link rel="icon" type_="image/png" sizes="32x32" href="/favicon-32x32.png"/>
                <Link rel="icon" type_="image/png" sizes="16x16" href="/favicon-16x16.png"/>
                <Link rel="apple-touch-icon" sizes="180x180" href="/favicon-180x180.png"/>
            }.into_any()
        }}
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        // htmx script (vendored locally)
        <Script src="/htmx.min.js"/>

        {config_provider}

        <Router>
            <div class="flex flex-col min-h-screen bg-gray-50 dark:bg-black text-gray-900 dark:text-amber-100">
                // Header with navigation
                <Header/>

                // Main content
                <main class="flex-grow">
                    {routes::app_routes()}
                </main>

                // Footer
                <Footer/>
            </div>
        </Router>
    }
}
