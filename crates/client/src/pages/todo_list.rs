use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;

use crate::api;
use crate::app::use_site_config;

#[component]
pub fn TodoListPage() -> impl IntoView {
    let config = use_site_config();

    let title_text = format!("{} - {}", config.pages.todos.title, config.name);
    let page_title = config.pages.todos.title.clone();
    let subtitle = config.pages.todos.subtitle.clone();
    let description = if config.pages.todos.description.is_empty() {
        "My public bucket list".to_string()
    } else {
        config.pages.todos.description.clone()
    };

    let todos = Resource::new(|| (), |_| async move { api::get_todos().await });

    view! {
        <Title text={title_text}/>
        <Meta name="description" content={description}/>

        <div class="min-h-screen bg-gray-50 dark:bg-black">
            <div class="container mx-auto px-4 py-16 max-w-5xl">
                <div class="mb-12">
                    <h1 class="text-5xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                        {page_title}
                    </h1>
                    {if !subtitle.is_empty() {
                        Some(view! {
                            <p class="text-xl text-gray-600 dark:text-amber-400">
                                {subtitle}
                            </p>
                        })
                    } else {
                        None
                    }}
                    <div class="h-1 w-20 bg-blue-600 rounded mt-4"></div>
                </div>

                <Suspense fallback=move || view! {
                    <div class="text-center py-12">
                        <p class="text-gray-600 dark:text-amber-400">"Loading..."</p>
                    </div>
                }>
                    {move || {
                        todos.get().map(|result| {
                            match result {
                                Ok(items) => {
                                    if items.is_empty() {
                                        EitherOf3::A(view! {
                                            <div class="text-center py-12">
                                                <p class="text-gray-600 dark:text-amber-400">
                                                    "Nothing here yet. Check back soon!"
                                                </p>
                                            </div>
                                        })
                                    } else {
                                        EitherOf3::B(view! {
                                            <div class="space-y-4">
                                                {items.into_iter().map(|item| {
                                                    let slug = item.slug.clone();
                                                    let completed = item.completed;
                                                    view! {
                                                        <a
                                                            href={format!("/todos/{}", slug)}
                                                            class={if completed {
                                                                "block bg-white dark:bg-black rounded-lg shadow-md hover:shadow-lg transition-all p-6 opacity-60"
                                                            } else {
                                                                "block bg-white dark:bg-black rounded-lg shadow-md hover:shadow-lg transition-all p-6"
                                                            }}
                                                        >
                                                            <div class="flex items-start gap-4">
                                                                // Status indicator
                                                                <div class="mt-1 flex-shrink-0">
                                                                    {if completed {
                                                                        view! {
                                                                            <span class="text-green-500 dark:text-green-400 text-xl">{"\u{2713}"}</span>
                                                                        }.into_any()
                                                                    } else {
                                                                        view! {
                                                                            <span class="text-gray-300 dark:text-amber-800 text-xl">{"\u{25CB}"}</span>
                                                                        }.into_any()
                                                                    }}
                                                                </div>

                                                                <div class="flex-grow">
                                                                    <h3 class={if completed {
                                                                        "text-xl font-bold text-gray-500 dark:text-amber-600 line-through"
                                                                    } else {
                                                                        "text-xl font-bold text-gray-900 dark:text-amber-100"
                                                                    }}>
                                                                        {item.title}
                                                                    </h3>
                                                                    <p class="text-gray-600 dark:text-amber-400 mt-1 text-sm">
                                                                        {item.description}
                                                                    </p>
                                                                    <div class="flex flex-wrap gap-2 mt-3">
                                                                        {item.tags.iter().map(|tag| view! {
                                                                            <span class="px-2 py-1 bg-gray-100 dark:bg-gray-900 text-gray-600 dark:text-amber-400 rounded text-xs">
                                                                                {tag.clone()}
                                                                            </span>
                                                                        }).collect::<Vec<_>>()}
                                                                    </div>
                                                                </div>

                                                                // Completion date for done items
                                                                {item.completed_at.map(|at| view! {
                                                                    <div class="text-xs text-gray-400 dark:text-amber-700 flex-shrink-0">
                                                                        {at.format("%b %Y").to_string()}
                                                                    </div>
                                                                })}
                                                            </div>
                                                        </a>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        })
                                    }
                                },
                                Err(e) => EitherOf3::C(view! {
                                    <div class="text-center py-12">
                                        <p class="text-red-600 dark:text-red-400">
                                            "Error: " {e.to_string()}
                                        </p>
                                    </div>
                                }),
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
