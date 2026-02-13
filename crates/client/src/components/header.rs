use leptos::either::Either;
use leptos::prelude::*;

use super::theme_toggle::ThemeToggle;

/// Site header with navigation and theme toggle
#[component]
pub fn Header() -> impl IntoView {
    let (menu_open, set_menu_open) = signal(false);

    view! {
        <header class="sticky top-0 z-50 bg-white dark:bg-gray-900 shadow-md">
            <nav class="container mx-auto px-4 py-4">
                <div class="flex items-center justify-between">
                    // Logo/Brand
                    <a href="/" class="text-2xl font-bold text-gray-900 dark:text-white hover:text-blue-600 dark:hover:text-blue-400 transition-colors">
                        "Personal Site"
                    </a>

                    // Desktop Navigation
                    <div class="hidden md:flex items-center space-x-8">
                        <a href="/" class="text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors">
                            "Home"
                        </a>
                        <a href="/about" class="text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors">
                            "About"
                        </a>
                        <a href="/portfolio" class="text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors">
                            "Portfolio"
                        </a>
                        <a href="/blog" class="text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors">
                            "Blog"
                        </a>

                        // Theme toggle
                        <ThemeToggle/>
                    </div>

                    // Mobile menu button
                    <button
                        class="md:hidden p-2 text-gray-700 dark:text-gray-300"
                        on:click=move |_| set_menu_open.update(|open| *open = !*open)
                        aria-label="Toggle menu"
                    >
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            {move || if menu_open.get() {
                                Either::Left(view! {
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                                })
                            } else {
                                Either::Right(view! {
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
                                })
                            }}
                        </svg>
                    </button>
                </div>

                // Mobile Navigation
                {move || menu_open.get().then(|| view! {
                    <div class="md:hidden mt-4 pb-4 space-y-4">
                        <a
                            href="/"
                            class="block text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
                            on:click=move |_| set_menu_open.set(false)
                        >
                            "Home"
                        </a>
                        <a
                            href="/about"
                            class="block text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
                            on:click=move |_| set_menu_open.set(false)
                        >
                            "About"
                        </a>
                        <a
                            href="/portfolio"
                            class="block text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
                            on:click=move |_| set_menu_open.set(false)
                        >
                            "Portfolio"
                        </a>
                        <a
                            href="/blog"
                            class="block text-gray-700 dark:text-gray-300 hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
                            on:click=move |_| set_menu_open.set(false)
                        >
                            "Blog"
                        </a>

                        // Theme toggle for mobile
                        <div class="pt-4 flex justify-start">
                            <ThemeToggle/>
                        </div>
                    </div>
                })}
            </nav>
        </header>
    }
}
