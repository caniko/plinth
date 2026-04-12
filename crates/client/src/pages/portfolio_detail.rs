use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;

use crate::api;
use crate::app::use_site_config;
use crate::components::ErrorMessage;

#[component]
pub fn PortfolioDetailPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.with(|p| p.get("slug").unwrap_or_default());

    let portfolio_item = Resource::new(slug, |slug| async move {
        api::get_portfolio_item_by_slug(slug).await
    });

    view! {
        <Suspense fallback=move || view! {
            <div class="min-h-screen bg-gray-50 dark:bg-black">
                <div class="container mx-auto px-4 py-16 max-w-4xl">
                    <div class="text-center">
                        <p class="text-gray-600 dark:text-amber-400">"Loading project..."</p>
                    </div>
                </div>
            </div>
        }>
            {move || {
                portfolio_item.get().map(|result| {
                    match result {
                        Ok(Some(item)) => {
                            let config = use_site_config();
                            EitherOf3::A(view! {
                                <Title text={format!("{} - {}", item.title, config.name)}/>
                                <Meta name="description" content={item.description.clone()}/>

                                <div class="min-h-screen bg-gray-50 dark:bg-black">
                                    <article class="container mx-auto px-4 py-16 max-w-4xl">
                                        // Back button
                                        <a href="/projects" class="inline-flex items-center text-blue-600 dark:text-amber-300 hover:underline mb-8">
                                            "\u{2190} Back to Projects"
                                        </a>

                                        // Header
                                        <header class="mb-12">
                                            <h1 class="text-5xl font-bold mb-6 text-gray-900 dark:text-amber-100 leading-tight">
                                                {item.title.clone()}
                                            </h1>

                                            <p class="text-xl text-gray-600 dark:text-amber-400 mb-6">
                                                {item.description.clone()}
                                            </p>

                                            <div class="flex flex-wrap items-center gap-4 mb-6">
                                                <div class="text-gray-600 dark:text-amber-400">
                                                    <span class="font-semibold">"Date: "</span>
                                                    {item.date.format("%B %Y").to_string()}
                                                </div>
                                            </div>

                                            <div class="flex flex-wrap gap-2 mb-6">
                                                {item.tech_stack.iter().map(|tech| view! {
                                                    <span class="px-4 py-2 bg-blue-100 dark:bg-amber-900/30 text-blue-800 dark:text-amber-200 rounded-full font-medium">
                                                        {tech.clone()}
                                                    </span>
                                                }).collect::<Vec<_>>()}
                                            </div>

                                            // Project image
                                            {item.image_url.as_ref().map(|url| view! {
                                                <img
                                                    src={url.clone()}
                                                    alt={item.title.clone()}
                                                    class="w-full rounded-lg shadow-xl mb-8"
                                                />
                                            })}
                                        </header>

                                        // Links
                                        <div class="mb-12 flex flex-wrap gap-4">
                                            {item.link.as_ref().map(|link| view! {
                                                <a
                                                    href={link.clone()}
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    class="btn-primary inline-flex items-center gap-2"
                                                >
                                                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"></path>
                                                    </svg>
                                                    "View Project"
                                                </a>
                                            })}

                                            {item.demo.as_ref().map(|demo| view! {
                                                <a
                                                    href={demo.clone()}
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    class="btn-secondary inline-flex items-center gap-2"
                                                >
                                                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"></path>
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                                                    </svg>
                                                    "Live Demo"
                                                </a>
                                            })}
                                        </div>

                                        // Detailed content (if exists)
                                        {item.html_content.as_ref().map(|html| view! {
                                            <div class="prose prose-lg dark:prose-invert max-w-none bg-white dark:bg-black rounded-lg shadow-lg p-8 md:p-12 mb-12">
                                                <div inner_html={html.clone()}></div>
                                            </div>
                                        })}

                                        // Footer
                                        <footer class="mt-12 pt-8 border-t border-gray-200 dark:border-amber-900/50">
                                            <div class="flex justify-between items-center">
                                                <a href="/projects" class="btn-secondary">
                                                    "\u{2190} All Projects"
                                                </a>
                                                {item.featured.then(|| view! {
                                                    <span class="px-4 py-2 bg-yellow-100 dark:bg-yellow-900 text-yellow-800 dark:text-yellow-200 rounded-full font-medium">
                                                        "\u{2b50} Featured Project"
                                                    </span>
                                                })}
                                            </div>
                                        </footer>
                                    </article>
                                </div>
                            })
                        },
                        Ok(None) => EitherOf3::B(view! {
                            <Title text="Project Not Found"/>
                            <div class="min-h-screen bg-gray-50 dark:bg-black">
                                <div class="container mx-auto px-4 py-16 max-w-4xl text-center">
                                    <h1 class="text-4xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                                        "Project Not Found"
                                    </h1>
                                    <p class="text-gray-600 dark:text-amber-400 mb-8">
                                        "The project you're looking for doesn't exist."
                                    </p>
                                    <a href="/projects" class="btn-primary">
                                        "View All Projects"
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
