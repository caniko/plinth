use leptos::*;
use wasm_bindgen::JsCast;

/// Dark mode toggle component
#[component]
pub fn ThemeToggle() -> impl IntoView {
    // Initialize theme from localStorage or system preference
    let (theme, set_theme) = create_signal(get_initial_theme());

    // Effect to apply theme changes
    create_effect(move |_| {
        apply_theme(theme.get());
    });

    let toggle_theme = move |_| {
        let new_theme = if theme.get() == "dark" { "light" } else { "dark" };
        set_theme.set(new_theme);
    };

    view! {
        <button
            on:click=toggle_theme
            class="p-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
            aria-label="Toggle dark mode"
        >
            {move || if theme.get() == "dark" {
                view! {
                    <svg class="w-6 h-6 text-yellow-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"></path>
                    </svg>
                }.into_view()
            } else {
                view! {
                    <svg class="w-6 h-6 text-gray-700" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"></path>
                    </svg>
                }.into_view()
            }}
        </button>
    }
}

/// Get initial theme from localStorage or system preference
fn get_initial_theme() -> &'static str {
    #[cfg(feature = "hydrate")]
    {
        use web_sys::window;

        if let Some(window) = window() {
            // Check localStorage first
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(stored_theme)) = storage.get_item("theme") {
                    return if stored_theme == "dark" { "dark" } else { "light" };
                }
            }

            // Fallback to system preference
            if let Ok(media) = window.match_media("(prefers-color-scheme: dark)") {
                if let Some(media_query_list) = media {
                    if media_query_list.matches() {
                        return "dark";
                    }
                }
            }
        }
    }

    "light"
}

/// Apply theme by toggling 'dark' class on html element
fn apply_theme(theme: &str) {
    #[cfg(feature = "hydrate")]
    {
        use web_sys::{window, HtmlElement};

        if let Some(window) = window() {
            if let Some(document) = window.document() {
                // Store in localStorage
                if let Ok(Some(storage)) = window.local_storage() {
                    let _ = storage.set_item("theme", theme);
                }

                // Apply to HTML element
                if let Some(html) = document.document_element() {
                    let html_element = html.dyn_into::<HtmlElement>().unwrap();
                    let class_list = html_element.class_list();

                    if theme == "dark" {
                        let _ = class_list.add_1("dark");
                    } else {
                        let _ = class_list.remove_1("dark");
                    }
                }
            }
        }
    }
}
