use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;

use crate::api;

#[component]
pub fn PortfolioPage() -> impl IntoView {
    let portfolio_items = Resource::new(|| (), |_| async move { api::get_portfolio_items().await });

    view! {
        <Title text="Portfolio - Personal Website"/>
        <Meta name="description" content="My portfolio of projects and work"/>

        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            <div class="container mx-auto px-4 py-16">
                // Header
                <div class="mb-12">
                    <h1 class="text-5xl font-bold mb-4 text-gray-900 dark:text-white">
                        "Portfolio"
                    </h1>
                    <p class="text-xl text-gray-600 dark:text-gray-400">
                        "A collection of my projects and work"
                    </p>
                    <div class="h-1 w-20 bg-blue-600 rounded mt-4"></div>
                </div>

                // Portfolio Grid
                <Suspense fallback=move || view! {
                    <div class="text-center py-12">
                        <p class="text-gray-600 dark:text-gray-400">"Loading portfolio..."</p>
                    </div>
                }>
                    {move || {
                        portfolio_items.get().map(|result| {
                            match result {
                                Ok(items) => {
                                    if items.is_empty() {
                                        EitherOf3::A(view! {
                                            <div class="text-center py-12">
                                                <p class="text-gray-600 dark:text-gray-400">
                                                    "No portfolio items yet. Check back soon!"
                                                </p>
                                            </div>
                                        })
                                    } else {
                                        EitherOf3::B(view! {
                                            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
                                                {items.into_iter().map(|item| {
                                                    let slug = item.slug.clone();
                                                    view! {
                                                        <a
                                                            href={format!("/portfolio/{}", slug)}
                                                            class="card card-dark block hover:scale-105 transition-transform"
                                                        >
                                                            {item.image_url.as_ref().map(|url| view! {
                                                                <img
                                                                    src={url.clone()}
                                                                    alt={item.title.clone()}
                                                                    class="w-full h-48 object-cover rounded-t-lg"
                                                                />
                                                            })}
                                                            <div class="p-6">
                                                                <h2 class="text-2xl font-bold mb-2 text-gray-900 dark:text-white">
                                                                    {item.title.clone()}
                                                                </h2>
                                                                <p class="text-gray-600 dark:text-gray-400 mb-4">
                                                                    {item.description.clone()}
                                                                </p>
                                                                <div class="flex flex-wrap gap-2">
                                                                    {item.tech_stack.iter().map(|tech| view! {
                                                                        <span class="px-3 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-full text-sm">
                                                                            {tech.clone()}
                                                                        </span>
                                                                    }).collect::<Vec<_>>()}
                                                                </div>
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
                                            "Error loading portfolio: " {e.to_string()}
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
