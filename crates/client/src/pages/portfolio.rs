use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;

use crate::api;
use crate::app::use_site_config;
use crate::components::ErrorMessage;

#[component]
pub fn PortfolioPage() -> impl IntoView {
    let config = use_site_config();

    let title_text = format!("{} - {}", config.pages.portfolio.title, config.name);
    let page_title = config.pages.portfolio.title.clone();
    let subtitle = config.pages.portfolio.subtitle.clone();
    let description = if config.pages.portfolio.description.is_empty() {
        "A collection of my projects and work".to_string()
    } else {
        config.pages.portfolio.description.clone()
    };

    let canonical_url = if config.base_url.is_empty() {
        "/projects".to_string()
    } else {
        format!("{}/projects", config.base_url)
    };

    let portfolio_items = Resource::new(|| (), |_| async move { api::get_portfolio_items().await });

    view! {
        <Title text={title_text}/>
        <Meta name="description" content={description}/>
        <Link rel="canonical" href={canonical_url}/>

        <div class="min-h-screen bg-gray-50 dark:bg-black">
            <div class="container mx-auto px-4 py-16">
                // Header
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

                // Portfolio Grid
                <Suspense fallback=move || view! {
                    <div class="text-center py-12">
                        <p class="text-gray-600 dark:text-amber-400">"Loading projects..."</p>
                    </div>
                }>
                    {move || {
                        portfolio_items.get().map(|result| {
                            match result {
                                Ok(items) => {
                                    if items.is_empty() {
                                        EitherOf3::A(view! {
                                            <div class="text-center py-12">
                                                <p class="text-gray-600 dark:text-amber-400">
                                                    "No projects yet. Check back soon!"
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
                                                            href={format!("/projects/{}", slug)}
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
                                                                <h2 class="text-2xl font-bold mb-2 text-gray-900 dark:text-amber-100">
                                                                    {item.title.clone()}
                                                                </h2>
                                                                <p class="text-gray-600 dark:text-amber-400 mb-4">
                                                                    {item.description.clone()}
                                                                </p>
                                                                <div class="flex flex-wrap gap-2">
                                                                    {item.tech_stack.iter().map(|tech| view! {
                                                                        <span class="px-3 py-1 bg-blue-100 dark:bg-amber-900/30 text-blue-800 dark:text-amber-200 rounded-full text-sm">
                                                                            {tech.clone()}
                                                                        </span>
                                                                    }).collect::<Vec<_>>()}
                                                                </div>
                                                                {item.project_url.as_ref().map(|_| view! {
                                                                    <span class="mt-4 inline-flex text-sm font-medium text-blue-600 dark:text-amber-300">
                                                                        "Project site"
                                                                    </span>
                                                                })}
                                                            </div>
                                                        </a>
                                                    }
                                                }).collect::<Vec<_>>()}
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
