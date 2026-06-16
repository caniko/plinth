use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;

use crate::api;
use crate::app::use_site_config;
use crate::components::ErrorMessage;

/// Blog listing page — all published posts ordered by date, with
/// series info, tags, and reading time.
#[component]
pub fn BlogListPage() -> impl IntoView {
    let config = use_site_config();

    let title_text = format!("{} - {}", config.pages.blog.title, config.name);
    let page_title = config.pages.blog.title.clone();
    let subtitle = config.pages.blog.subtitle.clone();
    let description = if config.pages.blog.description.is_empty() {
        "Read my latest thoughts on software development".to_string()
    } else {
        config.pages.blog.description.clone()
    };

    let blog_posts = Resource::new(|| (), |_| async move { api::get_blog_posts().await });

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
                        <p class="text-gray-600 dark:text-amber-400">"Loading posts..."</p>
                    </div>
                }>
                    {move || {
                        blog_posts.get().map(|result| {
                            match result {
                                Ok(posts) => {
                                    if posts.is_empty() {
                                        EitherOf3::A(view! {
                                            <div class="text-center py-12">
                                                <p class="text-gray-600 dark:text-amber-400">
                                                    "No posts yet. Check back soon!"
                                                </p>
                                            </div>
                                        })
                                    } else {
                                        EitherOf3::B(view! {
                                            <div class="space-y-6">
                                                {posts.into_iter().map(|post| {
                                                    let slug = post.slug.clone();
                                                    view! {
                                                        <a
                                                            href={format!("/posts/{}", slug)}
                                                            class="block bg-white dark:bg-black rounded-lg shadow-md hover:shadow-lg transition-all p-6"
                                                        >
                                                            <div class="flex items-start justify-between mb-3">
                                                                <h3 class="text-2xl font-bold text-gray-900 dark:text-amber-100">
                                                                    {post.title}
                                                                </h3>
                                                                <span class="text-sm text-gray-500 dark:text-amber-400 ml-4">
                                                                    {post.published_at.format("%b %d, %Y").to_string()}
                                                                </span>
                                                            </div>
                                                            {post.series_title.as_ref().map(|series_title| {
                                                                let pos = post.series_position.unwrap_or(0);
                                                                view! {
                                                                    <span class="inline-block px-2 py-0.5 mb-2 text-xs bg-blue-50 dark:bg-amber-900/20 text-blue-700 dark:text-amber-300 rounded">
                                                                        {format!("Part {} of {}", pos, series_title)}
                                                                    </span>
                                                                }
                                                            })}
                                                            <p class="text-gray-600 dark:text-amber-400 mb-4">
                                                                {post.description}
                                                            </p>
                                                            <div class="flex items-center justify-between">
                                                                <div class="flex flex-wrap gap-2">
                                                                    {post.tags.iter().take(3).map(|tag| view! {
                                                                        <span class="px-2 py-1 bg-gray-100 dark:bg-gray-900 text-gray-600 dark:text-amber-400 rounded text-xs">
                                                                            {tag.clone()}
                                                                        </span>
                                                                    }).collect::<Vec<_>>()}
                                                                </div>
                                                                <span class="text-xs text-gray-500">
                                                                    {post.reading_time_minutes} " min read"
                                                                </span>
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
