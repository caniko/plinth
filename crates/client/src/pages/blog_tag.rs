use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;

use crate::api;
use crate::app::use_site_config;

#[component]
pub fn BlogTagPage() -> impl IntoView {
    let config = use_site_config();
    let site_name = config.name.clone();

    let params = use_params_map();
    let tag = move || params.with(|p| p.get("tag").unwrap_or_default());

    let blog_posts = Resource::new(
        tag,
        |tag| async move { api::get_blog_posts_by_tag(tag).await },
    );

    let site_name_for_title = site_name.clone();

    view! {
        <Title text={move || format!("Posts tagged '{}' - {}", tag(), site_name_for_title)}/>
        <Meta name="description" content={move || format!("All posts tagged with {}", tag())}/>

        <div class="min-h-screen bg-gray-50 dark:bg-black">
            <div class="container mx-auto px-4 py-16">
                // Back to posts link
                <a href="/posts" class="inline-flex items-center text-blue-600 dark:text-amber-300 hover:underline mb-8">
                    "\u{2190} Back to All Posts"
                </a>

                // Header
                <div class="mb-12">
                    <h1 class="text-5xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                        "Posts tagged: "
                        <span class="text-blue-600 dark:text-amber-300">{tag}</span>
                    </h1>
                    <div class="h-1 w-20 bg-blue-600 rounded"></div>
                </div>

                // Posts listing
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
                                                <p class="text-gray-600 dark:text-amber-400 mb-4">
                                                    {format!("No posts found with tag '{}'", tag())}
                                                </p>
                                                <a href="/posts" class="btn-primary">
                                                    "View All Posts"
                                                </a>
                                            </div>
                                        })
                                    } else {
                                        EitherOf3::B(view! {
                                            <div>
                                                <p class="text-gray-600 dark:text-amber-400 mb-8">
                                                    {format!("Found {} post{}", posts.len(), if posts.len() == 1 { "" } else { "s" })}
                                                </p>
                                                <div class="grid grid-cols-1 gap-8">
                                                    {posts.into_iter().map(|post| {
                                                        let slug = post.slug.clone();
                                                        view! {
                                                            <article class="card card-dark hover:shadow-xl transition-shadow">
                                                                <a href={format!("/posts/{}", slug)} class="block">
                                                                    <h2 class="text-3xl font-bold mb-3 text-gray-900 dark:text-amber-100 hover:text-blue-600 dark:hover:text-amber-200 transition-colors">
                                                                        {post.title.clone()}
                                                                    </h2>

                                                                    <div class="flex flex-wrap items-center gap-4 text-sm text-gray-600 dark:text-amber-400 mb-4">
                                                                        <time>{post.published_at.format("%B %d, %Y").to_string()}</time>
                                                                        <span>"\u{2022}"</span>
                                                                        <span>{post.reading_time_minutes} " min read"</span>
                                                                        <span>"\u{2022}"</span>
                                                                        <span>{post.author.clone()}</span>
                                                                    </div>

                                                                    <p class="text-gray-700 dark:text-amber-200 mb-4 line-clamp-3">
                                                                        {post.description.clone()}
                                                                    </p>

                                                                    <div class="flex flex-wrap gap-2">
                                                                        {post.tags.iter().map(|t| view! {
                                                                            <span class="px-3 py-1 bg-blue-100 dark:bg-amber-900/30 text-blue-800 dark:text-amber-200 rounded-full text-sm">
                                                                                {t.clone()}
                                                                            </span>
                                                                        }).collect::<Vec<_>>()}
                                                                    </div>
                                                                </a>
                                                            </article>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            </div>
                                        })
                                    }
                                },
                                Err(e) => EitherOf3::C(view! {
                                    <div class="text-center py-12">
                                        <p class="text-red-600 dark:text-red-400">
                                            "Error loading posts: " {e.to_string()}
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
