use leptos::prelude::*;
use plinth_shared::config::NavItem;

use super::theme_toggle::ThemeToggle;
use crate::app::use_site_config;

/// Site header with navigation and theme toggle
#[component]
pub fn Header() -> impl IntoView {
    let config = use_site_config();
    let site_name = config.name.clone();
    let logo = config.logo.clone();
    let nav_items = config.nav.clone();
    let mobile_nav_items = nav_items.clone();
    let show_support = config.donation.enabled && !config.donation.links.is_empty();

    view! {
        <header class="sticky top-0 z-50 bg-white dark:bg-black shadow-md">
            <nav class="container mx-auto px-4 py-4">
                <div class="flex items-center justify-between">
                    // Logo/Brand
                    <a href="/" class="flex items-center gap-2 text-2xl font-bold text-gray-900 dark:text-amber-100 hover:text-blue-600 dark:hover:text-amber-200 transition-colors">
                        {logo.clone().map(|path| view! {
                            <img src={path} alt="" class="h-8 w-auto"/>
                        })}
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

                        // Support link (conditional)
                        {show_support.then(|| view! {
                            <a href="/support" class="inline-flex items-center gap-1 text-gray-700 dark:text-amber-200 hover:text-blue-600 dark:hover:text-amber-200 transition-colors">
                                <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                                    <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
                                </svg>
                                "Support"
                            </a>
                        })}

                        // Theme toggle
                        <ThemeToggle/>
                    </div>

                    <MobileMenu nav_items=mobile_nav_items show_support=show_support/>
                </div>
            </nav>
        </header>
    }
}

/// Mobile-only navigation menu.
#[cfg_attr(all(not(feature = "csr"), feature = "islands"), island)]
#[cfg_attr(any(feature = "csr", not(feature = "islands")), component)]
fn MobileMenu(nav_items: Vec<NavItem>, show_support: bool) -> impl IntoView {
    let (menu_open, set_menu_open) = signal(false);
    let close_menu = move |_| set_menu_open.set(false);

    view! {
        <div class="md:hidden">
            <button
                class="p-2 text-gray-700 dark:text-amber-200"
                on:click=move |_| set_menu_open.update(|open| *open = !*open)
                aria-label="Toggle menu"
                aria-expanded=move || menu_open.get().to_string()
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

            <Show when=move || menu_open.get()>
                <div class="mt-4 pb-4 space-y-4">
                    {nav_items.iter().map(|item| {
                        let href = item.path.clone();
                        let label = item.label.clone();
                        view! {
                            <a
                                href={href}
                                class="block text-gray-700 dark:text-amber-200 hover:text-blue-600 dark:hover:text-amber-200 transition-colors"
                                on:click=close_menu
                            >
                                {label}
                            </a>
                        }
                    }).collect::<Vec<_>>()}

                    {show_support.then(|| view! {
                        <a
                            href="/support"
                            class="flex items-center gap-1 text-gray-700 dark:text-amber-200 hover:text-blue-600 dark:hover:text-amber-200 transition-colors"
                            on:click=close_menu
                        >
                            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
                                <path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
                            </svg>
                            "Support"
                        </a>
                    })}

                    <div class="pt-4 flex justify-start">
                        <ThemeToggle/>
                    </div>
                </div>
            </Show>
        </div>
    }
}
