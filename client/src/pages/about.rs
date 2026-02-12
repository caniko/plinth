use leptos::*;
use leptos_meta::*;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <Title text="About - Personal Website"/>
        <Meta name="description" content="Learn more about me, my background, and what I do"/>

        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            <div class="container mx-auto px-4 py-16 max-w-4xl">
                // Header
                <div class="mb-12">
                    <h1 class="text-5xl font-bold mb-4 text-gray-900 dark:text-white">
                        "About Me"
                    </h1>
                    <div class="h-1 w-20 bg-blue-600 rounded"></div>
                </div>

                // Content
                <div class="prose prose-lg dark:prose-invert max-w-none">
                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8 mb-8">
                        <h2 class="text-3xl font-bold mb-4 text-gray-900 dark:text-white">
                            "Hello! 👋"
                        </h2>
                        <p class="text-gray-700 dark:text-gray-300 mb-4">
                            "Welcome to my personal website. This is a space where I share my "
                            "thoughts, projects, and experiences in software development."
                        </p>
                        <p class="text-gray-700 dark:text-gray-300 mb-4">
                            "I'm passionate about building fast, reliable, and maintainable software "
                            "using modern technologies like Rust, WebAssembly, and server-side rendering."
                        </p>
                    </div>

                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8 mb-8">
                        <h2 class="text-3xl font-bold mb-4 text-gray-900 dark:text-white">
                            "What I Do"
                        </h2>
                        <ul class="space-y-3 text-gray-700 dark:text-gray-300">
                            <li class="flex items-start">
                                <span class="text-blue-600 mr-2">"▸"</span>
                                <span>"Full-stack development with a focus on performance and developer experience"</span>
                            </li>
                            <li class="flex items-start">
                                <span class="text-blue-600 mr-2">"▸"</span>
                                <span>"Building web applications with Rust and WebAssembly"</span>
                            </li>
                            <li class="flex items-start">
                                <span class="text-blue-600 mr-2">"▸"</span>
                                <span>"Exploring semantic search and AI-powered features"</span>
                            </li>
                            <li class="flex items-start">
                                <span class="text-blue-600 mr-2">"▸"</span>
                                <span>"Writing about software architecture and best practices"</span>
                            </li>
                        </ul>
                    </div>

                    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8">
                        <h2 class="text-3xl font-bold mb-4 text-gray-900 dark:text-white">
                            "This Website"
                        </h2>
                        <p class="text-gray-700 dark:text-gray-300 mb-4">
                            "This site is built with cutting-edge technologies:"
                        </p>
                        <div class="grid grid-cols-2 md:grid-cols-3 gap-4 mb-4">
                            <div class="bg-gray-100 dark:bg-gray-700 rounded p-3 text-center">
                                <span class="font-semibold text-gray-900 dark:text-white">"Leptos 0.7"</span>
                            </div>
                            <div class="bg-gray-100 dark:bg-gray-700 rounded p-3 text-center">
                                <span class="font-semibold text-gray-900 dark:text-white">"Rust + WASM"</span>
                            </div>
                            <div class="bg-gray-100 dark:bg-gray-700 rounded p-3 text-center">
                                <span class="font-semibold text-gray-900 dark:text-white">"SurrealDB"</span>
                            </div>
                            <div class="bg-gray-100 dark:bg-gray-700 rounded p-3 text-center">
                                <span class="font-semibold text-gray-900 dark:text-white">"Tailwind CSS"</span>
                            </div>
                            <div class="bg-gray-100 dark:bg-gray-700 rounded p-3 text-center">
                                <span class="font-semibold text-gray-900 dark:text-white">"htmx"</span>
                            </div>
                            <div class="bg-gray-100 dark:bg-gray-700 rounded p-3 text-center">
                                <span class="font-semibold text-gray-900 dark:text-white">"Kameo"</span>
                            </div>
                        </div>
                        <p class="text-gray-700 dark:text-gray-300">
                            "Features server-side rendering for SEO, vector embeddings for semantic search, "
                            "and a type-safe actor system for state management."
                        </p>
                    </div>
                </div>

                // Navigation
                <div class="mt-12 flex gap-4">
                    <a href="/portfolio" class="btn-primary">
                        "View My Work →"
                    </a>
                    <a href="/blog" class="btn-secondary">
                        "Read My Blog →"
                    </a>
                </div>
            </div>
        </div>
    }
}
