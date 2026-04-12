use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;
use plinth_shared::SiteConfig;

use crate::api;
use crate::components::{Footer, Header};
use crate::pages::*;

/// Helper to get site config from context (falls back to defaults).
///
/// On SSR, the server provides `SiteConfig` via `provide_context`.
/// On the client, the App component provides it after hydrating from the
/// serialized Resource data.
pub fn use_site_config() -> SiteConfig {
    use_context::<SiteConfig>().unwrap_or_default()
}

#[component]
pub fn App() -> impl IntoView {
    // Provide meta context for SEO
    provide_meta_context();

    // Fetch site config once at app startup.
    // On SSR, this resolves immediately and the value is serialized into the HTML.
    // On the client, it hydrates from the serialized data.
    let site_config = Resource::new(|| (), |_| api::get_site_config());

    // Provide SiteConfig as context eagerly:
    // - On SSR: the server provides it via leptos_routes_with_context,
    //   so use_context finds the real config.
    // - On client: falls back to default until the Suspense below resolves.
    let initial_config = use_context::<SiteConfig>().unwrap_or_default();
    provide_context(initial_config);

    view! {
        // Meta tags
        <Stylesheet id="leptos" href="/pkg/plinth.css"/>
        <Link rel="icon" type_="image/svg+xml" href="/favicon.svg"/>
        <Link rel="icon" type_="image/png" sizes="32x32" href="/favicon-32x32.png"/>
        <Link rel="icon" type_="image/png" sizes="16x16" href="/favicon-16x16.png"/>
        <Link rel="apple-touch-icon" sizes="180x180" href="/favicon-180x180.png"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        // htmx script (vendored locally)
        <Script src="/htmx.min.js"/>

        // Hidden Suspense to trigger Resource serialization for client hydration.
        // On the client, this re-provides SiteConfig once the Resource resolves.
        <Suspense fallback=|| ()>
            {move || site_config.get().map(|result| {
                let config = result.unwrap_or_default();
                provide_context(config);
            })}
        </Suspense>

        <Router>
            <div class="flex flex-col min-h-screen bg-gray-50 dark:bg-black text-gray-900 dark:text-amber-100">
                // Header with navigation
                <Header/>

                // Main content
                <main class="flex-grow">
                    {app_routes()}
                </main>

                // Footer
                <Footer/>
            </div>
        </Router>
    }
}

/// Build all routes. This function exists so we can use #[cfg] to compose
/// the route tuple at compile time — Leptos Routes requires a statically
/// known tuple of Route components.
#[cfg(all(
    feature = "brick-blog",
    feature = "brick-portfolio",
    feature = "brick-todo"
))]
fn app_routes() -> impl IntoView {
    view! {
        <Routes fallback=|| view! { <NotFound/> }>
            <Route path=path!("/") view=HomePage/>
            <Route path=path!("/about") view=AboutPage/>
            <Route path=path!("/support") view=SupportPage/>
            <Route path=path!("/posts") view=BlogListPage/>
            <Route path=path!("/posts/:slug") view=BlogPostPage/>
            <Route path=path!("/posts/tag/:tag") view=BlogTagPage/>
            <Route path=path!("/series") view=SeriesListPage/>
            <Route path=path!("/series/:slug") view=SeriesDetailPage/>
            <Route path=path!("/projects") view=PortfolioPage/>
            <Route path=path!("/projects/:slug") view=PortfolioDetailPage/>
            <Route path=path!("/todos") view=TodoListPage/>
            <Route path=path!("/todos/tag/:tag") view=TodoTagPage/>
            <Route path=path!("/todos/:slug") view=TodoDetailPage/>
        </Routes>
    }
}

// When not all bricks are enabled, fall back to a minimal set.
// In practice, the default feature set enables all bricks, so this is
// only used for custom builds with specific bricks disabled.
#[cfg(not(all(
    feature = "brick-blog",
    feature = "brick-portfolio",
    feature = "brick-todo"
)))]
fn app_routes() -> impl IntoView {
    // Build a routes view with only enabled brick routes.
    // We use nested cfg to include each brick's routes.
    view! {
        <Routes fallback=|| view! { <NotFound/> }>
            <Route path=path!("/") view=HomePage/>
            <Route path=path!("/about") view=AboutPage/>
            <Route path=path!("/support") view=SupportPage/>
        </Routes>
    }
}
