use crate::api;
use crate::app::use_site_config;
use crate::components::ErrorMessage;
use chrono::{DateTime, Utc};
use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;

/// Todo detail page — single bucket list item with completion status,
/// dates, tags, and optional long-form HTML content. Reads `slug`
/// from URL params.
#[component]
pub fn TodoDetailPage() -> impl IntoView {
    let params = use_params_map();
    let slug = params.with_untracked(|p| p.get("slug").unwrap_or_default());

    let slug_for_resource = slug.clone();
    let todo = Resource::new(
        move || slug_for_resource.clone(),
        |slug| async move { api::get_todo_by_slug(slug).await },
    );

    view! {
        <Suspense fallback=move || view! {
            <div class="min-h-screen bg-gray-50 dark:bg-black">
                <div class="container mx-auto px-4 py-16 max-w-4xl">
                    <div class="text-center">
                        <p class="text-gray-600 dark:text-amber-400">"Loading..."</p>
                    </div>
                </div>
            </div>
        }>
            {move || {
                todo.get().map(|result| {
                    match result {
                        Ok(Some(item)) => {
                            let config = use_site_config();
                            let completed = item.completed;
                            EitherOf3::A(view! {
                                <Title text={format!("{} - {}", item.title, config.name)}/>
                                <Meta name="description" content={item.description.clone()}/>

                                <div class="min-h-screen bg-gray-50 dark:bg-black">
                                    <article class="container mx-auto px-4 py-16 max-w-4xl">
                                        // Back button
                                        <a href="/todos" class="inline-flex items-center text-blue-600 dark:text-amber-300 hover:underline mb-8">
                                            "\u{2190} Back to Bucket List"
                                        </a>

                                        // Header
                                        <header class="mb-12">
                                            <div class="flex items-center gap-4 mb-4">
                                                // Completion badge
                                                {if completed {
                                                    view! {
                                                        <span class="px-3 py-1 bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300 rounded-full text-sm font-medium">
                                                            "Completed"
                                                        </span>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <span class="px-3 py-1 bg-amber-100 dark:bg-amber-900/30 text-amber-800 dark:text-amber-300 rounded-full text-sm font-medium">
                                                            "In progress"
                                                        </span>
                                                    }.into_any()
                                                }}
                                            </div>

                                            <h1 class={if completed {
                                                "text-5xl font-bold mb-6 text-gray-500 dark:text-amber-600 leading-tight line-through"
                                            } else {
                                                "text-5xl font-bold mb-6 text-gray-900 dark:text-amber-100 leading-tight"
                                            }}>
                                                {item.title.clone()}
                                            </h1>

                                            <p class="text-xl text-gray-600 dark:text-amber-400 mb-6">
                                                {item.description.clone()}
                                            </p>

                                            <div class="flex flex-wrap items-center gap-4 text-gray-600 dark:text-amber-400 mb-6">
                                                <span>"Added: " {item.created_at.format("%B %d, %Y").to_string()}</span>
                                                {item.completed_at.map(|at: DateTime<Utc>| view! {
                                                    <span>"\u{2022}"</span>
                                                    <span>"Completed: " {at.format("%B %d, %Y").to_string()}</span>
                                                })}
                                            </div>

                                            <div class="flex flex-wrap gap-2">
                                                {item.tags.iter().map(|tag: &String| view! {
                                                    <a
                                                        href={format!("/todos/tag/{}", tag)}
                                                        class="px-3 py-1 bg-blue-100 dark:bg-amber-900/30 text-blue-800 dark:text-amber-200 rounded-full text-sm hover:bg-blue-200 dark:hover:bg-amber-800/30 transition-colors"
                                                    >
                                                        {tag.clone()}
                                                    </a>
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        </header>

                                        // Optional long-form content
                                        {item.html_content.map(|html| view! {
                                            <div class="prose prose-lg dark:prose-invert max-w-none bg-white dark:bg-black rounded-lg shadow-lg p-8 md:p-12">
                                                <div inner_html={html}></div>
                                            </div>
                                        })}

                                        // Footer
                                        <footer class="mt-12 pt-8 border-t border-gray-200 dark:border-amber-900/50">
                                            <a href="/todos" class="btn-secondary">
                                                "\u{2190} All Items"
                                            </a>
                                        </footer>
                                    </article>
                                </div>
                            })
                        },
                        Ok(None) => EitherOf3::B(view! {
                            <Title text="Not Found"/>
                            <div class="min-h-screen bg-gray-50 dark:bg-black">
                                <div class="container mx-auto px-4 py-16 max-w-4xl text-center">
                                    <h1 class="text-4xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                                        "Not Found"
                                    </h1>
                                    <p class="text-gray-600 dark:text-amber-400 mb-8">
                                        "This bucket list item doesn't exist."
                                    </p>
                                    <a href="/todos" class="btn-primary">
                                        "View Bucket List"
                                    </a>
                                </div>
                            </div>
                        }),
                        Err(_) => EitherOf3::C(view! {
                            <ErrorMessage/>
                        }),
                    }
                })
            }}
        </Suspense>
    }
}
