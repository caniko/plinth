use leptos::*;
use leptos_meta::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Title text="Home - Personal Website"/>
        <Meta name="description" content="Welcome to my personal website featuring my biography, portfolio, and blog"/>

        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            // Hero Section
            <section class="bg-gradient-to-br from-blue-600 to-purple-700 text-white py-20 md:py-32">
                <div class="container mx-auto px-4 text-center">
                    <h1 class="text-5xl md:text-7xl font-bold mb-6 leading-tight">
                        "Welcome to My"
                        <br/>
                        "Digital Space"
                    </h1>
                    <p class="text-xl md:text-2xl mb-8 max-w-2xl mx-auto text-blue-100">
                        "Explore my work, thoughts, and experiences in software engineering"
                    </p>
                    <div class="flex flex-wrap justify-center gap-4">
                        <a href="/about" class="px-8 py-4 bg-white text-blue-600 rounded-lg font-semibold hover:bg-blue-50 transition-colors shadow-lg">
                            "Learn About Me"
                        </a>
                        <a href="/blog" class="px-8 py-4 bg-transparent border-2 border-white text-white rounded-lg font-semibold hover:bg-white hover:text-blue-600 transition-colors">
                            "Read My Blog"
                        </a>
                    </div>
                </div>
            </section>

            // Main Sections Overview
            <section class="container mx-auto px-4 py-16">
                <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
                    // About Section Card
                    <a href="/about" class="card card-dark hover:scale-105 transition-transform group">
                        <div class="text-center">
                            <div class="w-16 h-16 bg-blue-100 dark:bg-blue-900 rounded-full flex items-center justify-center mx-auto mb-4 group-hover:bg-blue-200 dark:group-hover:bg-blue-800 transition-colors">
                                <svg class="w-8 h-8 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"></path>
                                </svg>
                            </div>
                            <h2 class="text-2xl font-bold mb-3 text-gray-900 dark:text-white group-hover:text-blue-600 dark:group-hover:text-blue-400 transition-colors">
                                "About Me"
                            </h2>
                            <p class="text-gray-600 dark:text-gray-400">
                                "Learn about my background, skills, and experience in software engineering"
                            </p>
                        </div>
                    </a>

                    // Portfolio Section Card
                    <a href="/portfolio" class="card card-dark hover:scale-105 transition-transform group">
                        <div class="text-center">
                            <div class="w-16 h-16 bg-purple-100 dark:bg-purple-900 rounded-full flex items-center justify-center mx-auto mb-4 group-hover:bg-purple-200 dark:group-hover:bg-purple-800 transition-colors">
                                <svg class="w-8 h-8 text-purple-600 dark:text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path>
                                </svg>
                            </div>
                            <h2 class="text-2xl font-bold mb-3 text-gray-900 dark:text-white group-hover:text-purple-600 dark:group-hover:text-purple-400 transition-colors">
                                "Portfolio"
                            </h2>
                            <p class="text-gray-600 dark:text-gray-400">
                                "Browse my projects and see what I've built with various technologies"
                            </p>
                        </div>
                    </a>

                    // Blog Section Card
                    <a href="/blog" class="card card-dark hover:scale-105 transition-transform group">
                        <div class="text-center">
                            <div class="w-16 h-16 bg-green-100 dark:bg-green-900 rounded-full flex items-center justify-center mx-auto mb-4 group-hover:bg-green-200 dark:group-hover:bg-green-800 transition-colors">
                                <svg class="w-8 h-8 text-green-600 dark:text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"></path>
                                </svg>
                            </div>
                            <h2 class="text-2xl font-bold mb-3 text-gray-900 dark:text-white group-hover:text-green-600 dark:group-hover:text-green-400 transition-colors">
                                "Blog"
                            </h2>
                            <p class="text-gray-600 dark:text-gray-400">
                                "Read my thoughts and insights on software development and technology"
                            </p>
                        </div>
                    </a>
                </div>
            </section>

            // Featured Section (Optional - can be populated dynamically later)
            <section class="bg-white dark:bg-gray-800 py-16">
                <div class="container mx-auto px-4">
                    <h2 class="text-4xl font-bold mb-12 text-center text-gray-900 dark:text-white">
                        "What I Do"
                    </h2>
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8">
                        <div class="text-center">
                            <div class="text-4xl mb-4">"🦀"</div>
                            <h3 class="text-xl font-semibold mb-2 text-gray-900 dark:text-white">"Rust Development"</h3>
                            <p class="text-gray-600 dark:text-gray-400">"Building performant and safe systems"</p>
                        </div>
                        <div class="text-center">
                            <div class="text-4xl mb-4">"🌐"</div>
                            <h3 class="text-xl font-semibold mb-2 text-gray-900 dark:text-white">"Web Development"</h3>
                            <p class="text-gray-600 dark:text-gray-400">"Creating modern web applications"</p>
                        </div>
                        <div class="text-center">
                            <div class="text-4xl mb-4">"🔍"</div>
                            <h3 class="text-xl font-semibold mb-2 text-gray-900 dark:text-white">"System Design"</h3>
                            <p class="text-gray-600 dark:text-gray-400">"Architecting scalable solutions"</p>
                        </div>
                        <div class="text-center">
                            <div class="text-4xl mb-4">"💡"</div>
                            <h3 class="text-xl font-semibold mb-2 text-gray-900 dark:text-white">"Problem Solving"</h3>
                            <p class="text-gray-600 dark:text-gray-400">"Finding elegant solutions"</p>
                        </div>
                    </div>
                </div>
            </section>

            // Call to Action
            <section class="container mx-auto px-4 py-16">
                <div class="bg-gradient-to-r from-blue-600 to-purple-600 rounded-2xl p-12 text-center text-white shadow-xl">
                    <h2 class="text-3xl md:text-4xl font-bold mb-4">
                        "Let's Connect"
                    </h2>
                    <p class="text-xl mb-8 text-blue-100">
                        "Interested in my work? Check out my portfolio or get in touch!"
                    </p>
                    <div class="flex flex-wrap justify-center gap-4">
                        <a href="/portfolio" class="px-8 py-4 bg-white text-blue-600 rounded-lg font-semibold hover:bg-blue-50 transition-colors">
                            "View Portfolio"
                        </a>
                        <a href="/about" class="px-8 py-4 bg-transparent border-2 border-white text-white rounded-lg font-semibold hover:bg-white hover:text-blue-600 transition-colors">
                            "Contact Info"
                        </a>
                    </div>
                </div>
            </section>
        </div>
    }
}
