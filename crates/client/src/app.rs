use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::components::{Footer, Header};
use crate::pages::*;

#[component]
pub fn App() -> impl IntoView {
    // Provide meta context for SEO
    provide_meta_context();

    view! {
        // Meta tags
        <Stylesheet id="leptos" href="/pkg/personal-website.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        // htmx script
        <Script src="https://unpkg.com/htmx.org@2.0.0/dist/htmx.min.js"/>

        <Router>
            <div class="flex flex-col min-h-screen">
                // Header with navigation
                <Header/>

                // Main content
                <main class="flex-grow">
                    <Routes fallback=|| view! { <NotFound/> }>
                        // Home page
                        <Route path=path!("/") view=HomePage/>

                        // About page
                        <Route path=path!("/about") view=AboutPage/>

                        // Portfolio routes
                        <Route path=path!("/portfolio") view=PortfolioPage/>
                        <Route path=path!("/portfolio/:slug") view=PortfolioDetailPage/>

                        // Blog routes
                        <Route path=path!("/blog") view=BlogListPage/>
                        <Route path=path!("/blog/:slug") view=BlogPostPage/>
                        <Route path=path!("/blog/tag/:tag") view=BlogTagPage/>
                    </Routes>
                </main>

                // Footer
                <Footer/>
            </div>
        </Router>
    }
}
