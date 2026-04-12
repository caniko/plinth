use leptos::prelude::*;

/// User-friendly error display that hides internal error details.
#[component]
pub fn ErrorMessage() -> impl IntoView {
    view! {
        <div class="text-center py-12" role="alert">
            <p class="text-red-600 dark:text-red-400 mb-4">
                "Something went wrong. Please try again later."
            </p>
            <button
                on:click=move |_| {
                    if let Some(window) = leptos::web_sys::window() {
                        let _ = window.location().reload();
                    }
                }
                class="text-blue-600 dark:text-amber-300 hover:underline cursor-pointer"
            >
                "Reload page"
            </button>
        </div>
    }
}
