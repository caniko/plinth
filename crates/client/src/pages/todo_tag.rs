use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;

use crate::api;
use crate::app::use_site_config;
use crate::components::ErrorMessage;

#[component]
pub fn TodoTagPage() -> impl IntoView {
    let config = use_site_config();
    let site_name = config.name.clone();

    let params = use_params_map();
    let tag = move || params.with(|p| p.get("tag").unwrap_or_default());

    let todos = Resource::new(tag, |tag| async move { api::get_todos_by_tag(tag).await });

    let site_name_for_title = site_name.clone();

    view! {
        <Title text={move || format!("Bucket list tagged '{}' - {}", tag(), site_name_for_title)}/>
        <Meta name="description" content={move || format!("Bucket list items tagged with {}", tag())}/>

        <div class="min-h-screen bg-gray-50 dark:bg-black">
            <div class="container mx-auto px-4 py-16 max-w-5xl">
                // Back link
                <a href="/todos" class="inline-flex items-center text-blue-600 dark:text-amber-300 hover:underline mb-8">
                    "\u{2190} Back to Bucket List"
                </a>

                // Header
                <div class="mb-12">
                    <h1 class="text-5xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                        "Tagged: "
                        <span class="text-blue-600 dark:text-amber-300">{tag}</span>
                    </h1>
                    <div class="h-1 w-20 bg-blue-600 rounded"></div>
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
                                                <p class="text-gray-600 dark:text-amber-400 mb-4">
                                                    {format!("No items found with tag '{}'", tag())}
                                                </p>
                                                <a href="/todos" class="btn-primary">
                                                    "View All Items"
                                                </a>
                                            </div>
                                        })
                                    } else {
                                        EitherOf3::B(view! {
                                            <div>
                                                <p class="text-gray-600 dark:text-amber-400 mb-8">
                                                    {format!("Found {} item{}", items.len(), if items.len() == 1 { "" } else { "s" })}
                                                </p>
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
                                                                            {item.tags.iter().map(|t| view! {
                                                                                <span class="px-2 py-1 bg-gray-100 dark:bg-gray-900 text-gray-600 dark:text-amber-400 rounded text-xs">
                                                                                    {t.clone()}
                                                                                </span>
                                                                            }).collect::<Vec<_>>()}
                                                                        </div>
                                                                    </div>
                                                                </div>
                                                            </a>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            </div>
                                        })
                                    }
                                },
                                Err(_) => EitherOf3::C(view! {
                                    <ErrorMessage/>
                                }),
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}
