use leptos::prelude::*;
use leptos_meta::*;

/// 404 page — shown when no route matches the requested URL.
#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <Title text="404 - Page Not Found"/>
        <Meta name="description" content="Page not found"/>

        <div class="container mx-auto px-4 py-8 text-center">
            <h1 class="text-6xl font-bold mb-4">"404"</h1>
            <p class="text-2xl mb-4">"Page Not Found"</p>
            <p class="text-lg mb-8">"The page you're looking for doesn't exist."</p>
            <a href="/" class="btn-primary">"Go Home"</a>
        </div>
    }
}
