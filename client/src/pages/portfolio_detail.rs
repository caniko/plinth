use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[cfg(feature = "ssr")]
use server::server_fns::get_portfolio_item_by_slug;

#[cfg(not(feature = "ssr"))]
use leptos::*;

#[component]
pub fn PortfolioDetailPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.with(|p| p.get("slug").cloned().unwrap_or_default());

    // Load portfolio item via server function
    let portfolio_item = create_resource(
        slug,
        |slug| async move {
            #[cfg(feature = "ssr")]
            { get_portfolio_item_by_slug(slug).await }

            #[cfg(not(feature = "ssr"))]
            { server::server_fns::get_portfolio_item_by_slug(slug).await }
        },
    );

    view! {
        <Suspense fallback=move || view! {
            <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
                <div class="container mx-auto px-4 py-16 max-w-4xl">
                    <div class="text-center">
                        <p class="text-gray-600 dark:text-gray-400">"Loading project..."</p>
                    </div>
                </div>
            </div>
        }>
            {move || {
                portfolio_item.get().map(|result| {
                    match result {
                        Ok(Some(item)) => {
                            view! {
                                <Title text={format!("{} - Portfolio", item.title)}/>
                                <Meta name="description" content={item.description.clone()}/>

                                <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
                                    <article class="container mx-auto px-4 py-16 max-w-4xl">
                                        // Back button
                                        <a href="/portfolio" class="inline-flex items-center text-blue-600 dark:text-blue-400 hover:underline mb-8">
                                            "← Back to Portfolio"
                                        </a>

                                        // Header
                                        <header class="mb-12">
                                            <h1 class="text-5xl font-bold mb-6 text-gray-900 dark:text-white leading-tight">
                                                {item.title.clone()}
                                            </h1>

                                            <p class="text-xl text-gray-600 dark:text-gray-400 mb-6">
                                                {item.description.clone()}
                                            </p>

                                            <div class="flex flex-wrap items-center gap-4 mb-6">
                                                {item.date.as_ref().map(|date| view! {
                                                    <div class="text-gray-600 dark:text-gray-400">
                                                        <span class="font-semibold">"Date: "</span>
                                                        {date.format("%B %Y").to_string()}
                                                    </div>
                                                })}
                                            </div>

                                            <div class="flex flex-wrap gap-2 mb-6">
                                                {item.tech_stack.iter().map(|tech| view! {
                                                    <span class="px-4 py-2 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-full font-medium">
                                                        {tech.clone()}
                                                    </span>
                                                }).collect_view()}
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

                                        // Long description (if exists)
                                        {item.long_description.as_ref().map(|desc| view! {
                                            <div class="prose prose-lg dark:prose-invert max-w-none bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8 md:p-12 mb-12">
                                                <div inner_html={desc.clone()}></div>
                                            </div>
                                        })}

                                        // Footer
                                        <footer class="mt-12 pt-8 border-t border-gray-200 dark:border-gray-700">
                                            <div class="flex justify-between items-center">
                                                <a href="/portfolio" class="btn-secondary">
                                                    "← All Projects"
                                                </a>
                                                {item.featured.then(|| view! {
                                                    <span class="px-4 py-2 bg-yellow-100 dark:bg-yellow-900 text-yellow-800 dark:text-yellow-200 rounded-full font-medium">
                                                        "⭐ Featured Project"
                                                    </span>
                                                })}
                                            </div>
                                        </footer>
                                    </article>
                                </div>
                            }.into_view()
                        }
                        Ok(None) => view! {
                            <Title text="Project Not Found"/>
                            <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
                                <div class="container mx-auto px-4 py-16 max-w-4xl text-center">
                                    <h1 class="text-4xl font-bold mb-4 text-gray-900 dark:text-white">
                                        "Project Not Found"
                                    </h1>
                                    <p class="text-gray-600 dark:text-gray-400 mb-8">
                                        "The portfolio item you're looking for doesn't exist."
                                    </p>
                                    <a href="/portfolio" class="btn-primary">
                                        "View All Projects"
                                    </a>
                                </div>
                            </div>
                        }.into_view()
                        Err(e) => view! {
                            <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
                                <div class="container mx-auto px-4 py-16 max-w-4xl text-center">
                                    <p class="text-red-600 dark:text-red-400">
                                        "Error: " {e.to_string()}
                                    </p>
                                </div>
                            </div>
                        }.into_view()
                    }
                })
            }}
        </Suspense>
    }
}
