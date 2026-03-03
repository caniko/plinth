use leptos::prelude::*;

use super::theme_toggle::ThemeToggle;
use crate::app::use_site_config;

/// Site header with navigation and theme toggle
#[component]
pub fn Header() -> impl IntoView {
    let (menu_open, set_menu_open) = signal(false);
    let config = use_site_config();
    let site_name = config.name.clone();
    let nav_items = config.nav.clone();

    view! {
        <header class="sticky top-0 z-50 bg-white dark:bg-black shadow-md">
            <nav class="container mx-auto px-4 py-4">
                <div class="flex items-center justify-between">
                    // Logo/Brand
                    <a href="/" class="flex items-center gap-2 text-2xl font-bold text-gray-900 dark:text-amber-100 hover:text-blue-600 dark:hover:text-amber-200 transition-colors">
                        <img src="/plinth-logo.svg" alt="" class="h-8 w-auto"/>
                        {site_name}
                    </a>

                    // Desktop Navigation
                    <div class="hidden md:flex items-center space-x-8">
                        {nav_items.iter().map(|item| {
                            let href = item.path.clone();
                            let label = item.label.clone();
                            view! {
                                <a href={href} class="text-gray-700 dark:text-amber-200 hover:text-blue-600 dark:hover:text-amber-200 transition-colors">
                                    {label}
                                </a>
                            }
                        }).collect::<Vec<_>>()}

                        // Theme toggle
                        <ThemeToggle/>
                    </div>

                    // Mobile menu button
                    <button
                        class="md:hidden p-2 text-gray-700 dark:text-amber-200"
                        on:click=move |_| set_menu_open.update(|open| *open = !*open)
                        aria-label="Toggle menu"
                    >
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <Show
                                when=move || menu_open.get()
                                fallback=|| view! {
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
                                }
                            >
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                            </Show>
                        </svg>
                    </button>
                </div>

                // Mobile Navigation
                <Show when=move || menu_open.get()>
                    <div class="md:hidden mt-4 pb-4 space-y-4">
                        {nav_items.iter().map(|item| {
                            let href = item.path.clone();
                            let label = item.label.clone();
                            view! {
                                <a
                                    href={href}
                                    class="block text-gray-700 dark:text-amber-200 hover:text-blue-600 dark:hover:text-amber-200 transition-colors"
                                    on:click=move |_| set_menu_open.set(false)
                                >
                                    {label}
                                </a>
                            }
                        }).collect::<Vec<_>>()}

                        // Theme toggle for mobile
                        <div class="pt-4 flex justify-start">
                            <ThemeToggle/>
                        </div>
                    </div>
                </Show>
            </nav>
        </header>
    }
}
