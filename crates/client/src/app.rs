use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;
use plinth_shared::SiteConfig;

use crate::api;
use crate::components::{Footer, Header};
use crate::pages::*;

/// Helper to get site config from context (falls back to defaults)
pub fn use_site_config() -> SiteConfig {
    use_context::<SiteConfig>().unwrap_or_default()
}

#[component]
pub fn App() -> impl IntoView {
    // Provide meta context for SEO
    provide_meta_context();

    // Fetch site config once at app startup
    let site_config = Resource::new(|| (), |_| api::get_site_config());

    view! {
        // Meta tags
        <Stylesheet id="leptos" href="/pkg/plinth.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        // htmx script
        <Script src="https://unpkg.com/htmx.org@2.0.0/dist/htmx.min.js"/>

        <Router>
            <Suspense fallback=|| ()>
                {move || {
                    site_config.get().map(|result| {
                        let config = result.unwrap_or_default();
                        view! { <AppShell config/> }
                    })
                }}
            </Suspense>
        </Router>
    }
}

/// Inner shell — receives the resolved config as a prop and provides it as context.
#[component]
fn AppShell(config: SiteConfig) -> impl IntoView {
    provide_context(config);

    view! {
        <div class="flex flex-col min-h-screen bg-gray-50 dark:bg-black text-gray-900 dark:text-amber-100">
            // Header with navigation
            <Header/>

            // Main content
            <main class="flex-grow">
                <Routes fallback=|| view! { <NotFound/> }>
                    // Home page
                    <Route path=path!("/") view=HomePage/>

                    // About page
                    <Route path=path!("/about") view=AboutPage/>

                    // Project routes
                    <Route path=path!("/projects") view=PortfolioPage/>
                    <Route path=path!("/projects/:slug") view=PortfolioDetailPage/>

                    // Post routes
                    <Route path=path!("/posts") view=BlogListPage/>
                    <Route path=path!("/posts/:slug") view=BlogPostPage/>
                    <Route path=path!("/posts/tag/:tag") view=BlogTagPage/>

                    // Bucket list routes
                    <Route path=path!("/todos") view=TodoListPage/>
                    <Route path=path!("/todos/tag/:tag") view=TodoTagPage/>
                    <Route path=path!("/todos/:slug") view=TodoDetailPage/>
                </Routes>
            </main>

            // Footer
            <Footer/>
        </div>
    }
}
