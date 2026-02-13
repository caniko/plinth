use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;

use crate::api;

#[component]
pub fn BlogTagPage() -> impl IntoView {
    let params = use_params_map();
    let tag = move || params.with(|p| p.get("tag").unwrap_or_default());

    let blog_posts = Resource::new(
        tag,
        |tag| async move { api::get_blog_posts_by_tag(tag).await },
    );

    view! {
        <Title text={move || format!("Posts tagged '{}' - Blog", tag())}/>
        <Meta name="description" content={move || format!("All blog posts tagged with {}", tag())}/>

        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            <div class="container mx-auto px-4 py-16">
                // Back to blog link
                <a href="/blog" class="inline-flex items-center text-blue-600 dark:text-blue-400 hover:underline mb-8">
                    "← Back to All Posts"
                </a>

                // Header
                <div class="mb-12">
                    <h1 class="text-5xl font-bold mb-4 text-gray-900 dark:text-white">
                        "Posts tagged: "
                        <span class="text-blue-600 dark:text-blue-400">{tag}</span>
                    </h1>
                    <div class="h-1 w-20 bg-blue-600 rounded"></div>
                </div>

                // Posts listing
                <Suspense fallback=move || view! {
                    <div class="text-center py-12">
                        <p class="text-gray-600 dark:text-gray-400">"Loading posts..."</p>
                    </div>
                }>
                    {move || {
                        blog_posts.get().map(|result| {
                            match result {
                                Ok(posts) => {
                                    if posts.is_empty() {
                                        EitherOf3::A(view! {
                                            <div class="text-center py-12">
                                                <p class="text-gray-600 dark:text-gray-400 mb-4">
                                                    {format!("No posts found with tag '{}'", tag())}
                                                </p>
                                                <a href="/blog" class="btn-primary">
                                                    "View All Posts"
                                                </a>
                                            </div>
                                        })
                                    } else {
                                        EitherOf3::B(view! {
                                            <div>
                                                <p class="text-gray-600 dark:text-gray-400 mb-8">
                                                    {format!("Found {} post{}", posts.len(), if posts.len() == 1 { "" } else { "s" })}
                                                </p>
                                                <div class="grid grid-cols-1 gap-8">
                                                    {posts.into_iter().map(|post| {
                                                        let slug = post.slug.clone();
                                                        view! {
                                                            <article class="card card-dark hover:shadow-xl transition-shadow">
                                                                <a href={format!("/blog/{}", slug)} class="block">
                                                                    <h2 class="text-3xl font-bold mb-3 text-gray-900 dark:text-white hover:text-blue-600 dark:hover:text-blue-400 transition-colors">
                                                                        {post.title.clone()}
                                                                    </h2>

                                                                    <div class="flex flex-wrap items-center gap-4 text-sm text-gray-600 dark:text-gray-400 mb-4">
                                                                        <time>{post.published_at.format("%B %d, %Y").to_string()}</time>
                                                                        <span>"•"</span>
                                                                        <span>{post.reading_time_minutes} " min read"</span>
                                                                        <span>"•"</span>
                                                                        <span>{post.author.clone()}</span>
                                                                    </div>

                                                                    <p class="text-gray-700 dark:text-gray-300 mb-4 line-clamp-3">
                                                                        {post.description.clone()}
                                                                    </p>

                                                                    <div class="flex flex-wrap gap-2">
                                                                        {post.tags.iter().map(|t| view! {
                                                                            <span class="px-3 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-full text-sm">
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
