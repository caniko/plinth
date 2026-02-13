use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;

use crate::api;

#[component]
pub fn BlogListPage() -> impl IntoView {
    let blog_posts = Resource::new(|| (), |_| async move { api::get_blog_posts().await });

    view! {
        <Title text="Blog - Personal Website"/>
        <Meta name="description" content="Read my latest thoughts on software development"/>

        <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
            <div class="container mx-auto px-4 py-16 max-w-5xl">
                <div class="mb-12">
                    <h1 class="text-5xl font-bold mb-4 text-gray-900 dark:text-white">
                        "Blog"
                    </h1>
                    <p class="text-xl text-gray-600 dark:text-gray-400">
                        "Thoughts on software development, Rust, and web technologies"
                    </p>
                    <div class="h-1 w-20 bg-blue-600 rounded mt-4"></div>
                </div>

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
                                                <p class="text-gray-600 dark:text-gray-400">
                                                    "No blog posts yet. Check back soon!"
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
                                                            href={format!("/blog/{}", slug)}
                                                            class="block bg-white dark:bg-gray-800 rounded-lg shadow-md hover:shadow-lg transition-all p-6"
                                                        >
                                                            <div class="flex items-start justify-between mb-3">
                                                                <h3 class="text-2xl font-bold text-gray-900 dark:text-white">
                                                                    {post.title}
                                                                </h3>
                                                                <span class="text-sm text-gray-500 dark:text-gray-400 ml-4">
                                                                    {post.published_at.format("%b %d, %Y").to_string()}
                                                                </span>
                                                            </div>
                                                            <p class="text-gray-600 dark:text-gray-400 mb-4">
                                                                {post.description}
                                                            </p>
                                                            <div class="flex items-center justify-between">
                                                                <div class="flex flex-wrap gap-2">
                                                                    {post.tags.iter().take(3).map(|tag| view! {
                                                                        <span class="px-2 py-1 bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 rounded text-xs">
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
