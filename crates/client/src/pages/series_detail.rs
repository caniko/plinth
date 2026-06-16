use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;

use crate::api;
use crate::app::use_site_config;
use crate::components::ErrorMessage;

/// Series detail page — ordered list of posts in a blog series with
/// position numbers and total reading time. Reads `slug` from URL params.
#[component]
pub fn SeriesDetailPage() -> impl IntoView {
    let params = use_params_map();
    let series_slug = params.with_untracked(|p| p.get("slug").unwrap_or_default());

    let series_slug_for_resource = series_slug.clone();
    let series_posts = Resource::new(
        move || series_slug_for_resource.clone(),
        |slug| async move { api::get_series_posts(slug).await },
    );

    view! {
        <Suspense fallback=move || view! {
            <div class="min-h-screen bg-gray-50 dark:bg-black">
                <div class="container mx-auto px-4 py-16 max-w-4xl text-center">
                    <p class="text-gray-600 dark:text-amber-400">"Loading series..."</p>
                </div>
            </div>
        }>
            {move || {
                series_posts.get().map(|result| {
                    match result {
                        Ok(posts) if !posts.is_empty() => {
                            let config = use_site_config();
                            let series_slug = series_slug.clone();
                            let series_title = posts[0].series_title.clone().unwrap_or_else(|| series_slug.clone());
                            let total_reading_time: u32 = posts.iter().map(|p| p.reading_time_minutes).sum();

                            let canonical_url = if config.base_url.is_empty() {
                                format!("/series/{}", series_slug)
                            } else {
                                format!("{}/series/{}", config.base_url, series_slug)
                            };
                            let series_slug_for_feed = series_slug.clone();

                            EitherOf3::A(view! {
                                <Title text={format!("{} - {}", series_title, config.name)}/>
                                <Meta name="description" content={format!("A series of {} posts", posts.len())}/>
                                <Link rel="canonical" href={canonical_url}/>

                                <div class="min-h-screen bg-gray-50 dark:bg-black">
                                    <div class="container mx-auto px-4 py-16 max-w-4xl">
                                        <a href="/series" class="inline-flex items-center text-blue-600 dark:text-amber-300 hover:underline mb-8">
                                            "\u{2190} All Series"
                                        </a>

                                        <header class="mb-12">
                                            <h1 class="text-5xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                                                {series_title}
                                            </h1>
                                            <div class="flex items-center gap-4 text-gray-600 dark:text-amber-400">
                                                <span>{posts.len()} " parts"</span>
                                                <span>"\u{2022}"</span>
                                                <span>{total_reading_time} " min total"</span>
                                            </div>
                                            <div class="mt-4">
                                                <a
                                                    href={format!("/feeds/series/{}.xml", series_slug_for_feed)}
                                                    class="inline-flex items-center gap-1 text-sm text-gray-500 dark:text-amber-600 hover:text-blue-600 dark:hover:text-amber-300"
                                                >
                                                    "RSS Feed"
                                                </a>
                                            </div>
                                            <div class="h-1 w-20 bg-blue-600 rounded mt-4"></div>
                                        </header>

                                        <ol class="space-y-4">
                                            {posts.into_iter().map(|post| {
                                                let position = post.series_position.unwrap_or(0);
                                                view! {
                                                    <li>
                                                        <a
                                                            href={format!("/posts/{}", post.slug)}
                                                            class="flex items-start gap-4 bg-white dark:bg-black rounded-lg shadow-md hover:shadow-lg transition-all p-5"
                                                        >
                                                            <span class="flex-shrink-0 w-8 h-8 rounded-full bg-blue-100 dark:bg-amber-900/30 text-blue-800 dark:text-amber-200 flex items-center justify-center font-bold text-sm">
                                                                {position}
                                                            </span>
                                                            <div class="flex-1 min-w-0">
                                                                <h3 class="text-xl font-bold text-gray-900 dark:text-amber-100 mb-1">
                                                                    {post.title}
                                                                </h3>
                                                                <p class="text-gray-600 dark:text-amber-400 text-sm mb-2 line-clamp-2">
                                                                    {post.description}
                                                                </p>
                                                                <div class="flex items-center gap-3 text-xs text-gray-500 dark:text-amber-600">
                                                                    <time>{post.published_at.format("%b %d, %Y").to_string()}</time>
                                                                    <span>{post.reading_time_minutes} " min read"</span>
                                                                </div>
                                                            </div>
                                                        </a>
                                                    </li>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </ol>
                                    </div>
                                </div>
                            })
                        },
                        Ok(_) => EitherOf3::B(view! {
                            <Title text="Series Not Found"/>
                            <div class="min-h-screen bg-gray-50 dark:bg-black">
                                <div class="container mx-auto px-4 py-16 max-w-4xl text-center">
                                    <h1 class="text-4xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                                        "Series Not Found"
                                    </h1>
                                    <p class="text-gray-600 dark:text-amber-400 mb-8">
                                        "This series doesn't exist or has no published posts."
                                    </p>
                                    <a href="/series" class="btn-primary">"View All Series"</a>
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
