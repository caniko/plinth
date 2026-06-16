use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::*;

use crate::api;
use crate::app::use_site_config;

/// About page — displays editable `about` site content with a default
/// fallback bio when no custom content is configured.
#[component]
pub fn AboutPage() -> impl IntoView {
    let config = use_site_config();

    let page_title = config.pages.about.title.clone();
    let title_text = format!("{} - {}", config.pages.about.title, config.name);
    let description = if config.pages.about.description.is_empty() {
        "Learn more about me, my background, and what I do".to_string()
    } else {
        config.pages.about.description.clone()
    };

    let about_content = Resource::new(
        || (),
        |_| async move { api::get_site_content("about".to_string()).await },
    );

    view! {
        <Title text={title_text}/>
        <Meta name="description" content={description}/>

        <div class="min-h-screen bg-gray-50 dark:bg-black">
            <div class="container mx-auto px-4 py-16 max-w-4xl">
                // Header
                <div class="mb-12">
                    <h1 class="text-5xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                        {page_title}
                    </h1>
                    <div class="h-1 w-20 bg-blue-600 rounded"></div>
                </div>

                // Content
                <Suspense fallback=move || view! {
                    <p class="text-gray-500 dark:text-amber-400">"Loading..."</p>
                }>
                    {move || {
                        about_content.get().map(|result| {
                            match result {
                                Ok(Some(content)) => Either::Left(view! {
                                    <div class="prose prose-lg dark:prose-invert max-w-none"
                                         inner_html={content.html_content}>
                                    </div>
                                }),
                                _ => Either::Right(view! {
                                    <DefaultAboutContent/>
                                }),
                            }
                        })
                    }}
                </Suspense>

                // Navigation
                <div class="mt-12 flex gap-4">
                    <a href="/projects" class="btn-primary">
                        "View My Work \u{2192}"
                    </a>
                    <a href="/posts" class="btn-secondary">
                        "Read My Posts \u{2192}"
                    </a>
                </div>
            </div>
        </div>
    }
}

#[component]
fn DefaultAboutContent() -> impl IntoView {
    view! {
        <div class="prose prose-lg dark:prose-invert max-w-none">
            <div class="bg-white dark:bg-black rounded-lg shadow-lg p-8 mb-8">
                <h2 class="text-3xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                    "Hello!"
                </h2>
                <p class="text-gray-700 dark:text-amber-200 mb-4">
                    "Welcome to my personal website. This is a space where I share my "
                    "thoughts, projects, and experiences in software development."
                </p>
                <p class="text-gray-700 dark:text-amber-200 mb-4">
                    "I'm passionate about building fast, reliable, and maintainable software "
                    "using modern technologies like Rust, WebAssembly, and server-side rendering."
                </p>
            </div>

            <div class="bg-white dark:bg-black rounded-lg shadow-lg p-8 mb-8">
                <h2 class="text-3xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                    "What I Do"
                </h2>
                <ul class="space-y-3 text-gray-700 dark:text-amber-200">
                    <li class="flex items-start">
                        <span class="text-blue-600 mr-2">{"\u{25b8}"}</span>
                        <span>"Full-stack development with a focus on performance and developer experience"</span>
                    </li>
                    <li class="flex items-start">
                        <span class="text-blue-600 mr-2">{"\u{25b8}"}</span>
                        <span>"Building web applications with Rust and WebAssembly"</span>
                    </li>
                    <li class="flex items-start">
                        <span class="text-blue-600 mr-2">{"\u{25b8}"}</span>
                        <span>"Exploring semantic search and AI-powered features"</span>
                    </li>
                    <li class="flex items-start">
                        <span class="text-blue-600 mr-2">{"\u{25b8}"}</span>
                        <span>"Writing about software architecture and best practices"</span>
                    </li>
                </ul>
            </div>

            <div class="bg-white dark:bg-black rounded-lg shadow-lg p-8">
                <h2 class="text-3xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                    "This Website"
                </h2>
                <p class="text-gray-700 dark:text-amber-200 mb-4">
                    "This site is built with cutting-edge technologies:"
                </p>
                <div class="grid grid-cols-2 md:grid-cols-3 gap-4 mb-4">
                    <div class="bg-gray-100 dark:bg-gray-900 rounded p-3 text-center">
                        <span class="font-semibold text-gray-900 dark:text-amber-100">"Leptos 0.7"</span>
                    </div>
                    <div class="bg-gray-100 dark:bg-gray-900 rounded p-3 text-center">
                        <span class="font-semibold text-gray-900 dark:text-amber-100">"Rust + WASM"</span>
                    </div>
                    <div class="bg-gray-100 dark:bg-gray-900 rounded p-3 text-center">
                        <span class="font-semibold text-gray-900 dark:text-amber-100">"Postgres"</span>
                    </div>
                    <div class="bg-gray-100 dark:bg-gray-900 rounded p-3 text-center">
                        <span class="font-semibold text-gray-900 dark:text-amber-100">"Tailwind CSS"</span>
                    </div>
                    <div class="bg-gray-100 dark:bg-gray-900 rounded p-3 text-center">
                        <span class="font-semibold text-gray-900 dark:text-amber-100">"htmx"</span>
                    </div>
                    <div class="bg-gray-100 dark:bg-gray-900 rounded p-3 text-center">
                        <span class="font-semibold text-gray-900 dark:text-amber-100">"Kameo"</span>
                    </div>
                </div>
                <p class="text-gray-700 dark:text-amber-200">
                    "Features server-side rendering for SEO, vector embeddings for semantic search, "
                    "and a type-safe actor system for state management."
                </p>
            </div>
        </div>
    }
}
