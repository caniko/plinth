use chrono::{DateTime, Utc};
use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;
use shared::BlogPost;

use crate::api;

#[component]
pub fn BlogPostPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.with(|p| p.get("slug").unwrap_or_default());

    let blog_post = Resource::new(slug, |slug| async move {
        api::get_blog_post_by_slug(slug).await
    });

    view! {
        <Suspense fallback=move || view! {
            <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
                <div class="container mx-auto px-4 py-16 max-w-4xl">
                    <div class="text-center">
                        <p class="text-gray-600 dark:text-gray-400">"Loading post..."</p>
                    </div>
                </div>
            </div>
        }>
            {move || {
                blog_post.get().map(|result: Result<Option<BlogPost>, String>| {
                    match result {
                        Ok(Some(post)) => {
                            EitherOf3::A(view! {
                                <Title text={format!("{} - Blog", post.title)}/>
                                <Meta name="description" content={post.content.chars().take(160).collect::<String>()}/>

                                <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
                                    <article class="container mx-auto px-4 py-16 max-w-4xl">
                                        // Back button
                                        <a href="/blog" class="inline-flex items-center text-blue-600 dark:text-blue-400 hover:underline mb-8">
                                            "← Back to Blog"
                                        </a>

                                        // Header
                                        <header class="mb-12">
                                            <h1 class="text-5xl font-bold mb-6 text-gray-900 dark:text-white leading-tight">
                                                {post.title.clone()}
                                            </h1>

                                            <div class="flex flex-wrap items-center gap-4 text-gray-600 dark:text-gray-400 mb-6">
                                                <time>{post.published_at.format("%B %d, %Y").to_string()}</time>
                                                <span>"•"</span>
                                                <span>{post.reading_time_minutes} " min read"</span>
                                                <span>"•"</span>
                                                <span>{post.author.clone()}</span>
                                            </div>

                                            <div class="flex flex-wrap gap-2">
                                                {post.tags.iter().map(|tag: &String| view! {
                                                    <a
                                                        href={format!("/blog/tag/{}", tag)}
                                                        class="px-3 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-full text-sm hover:bg-blue-200 dark:hover:bg-blue-800 transition-colors"
                                                    >
                                                        {tag.clone()}
                                                    </a>
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        </header>

                                        // Content
                                        <div class="prose prose-lg dark:prose-invert max-w-none bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8 md:p-12">
                                            <div inner_html={post.html_content}></div>
                                        </div>

                                        // Footer
                                        <footer class="mt-12 pt-8 border-t border-gray-200 dark:border-gray-700">
                                            <div class="flex justify-between items-center">
                                                <a href="/blog" class="btn-secondary">
                                                    "← All Posts"
                                                </a>
                                                <div class="text-sm text-gray-500 dark:text-gray-500">
                                                    {post.updated_at.map(|updated: DateTime<Utc>| {
                                                        format!("Updated: {}", updated.format("%b %d, %Y"))
                                                    })}
                                                </div>
                                            </div>
                                        </footer>
                                    </article>
                                </div>
                            })
                        },
                        Ok(None) => EitherOf3::B(view! {
                            <Title text="Post Not Found"/>
                            <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
                                <div class="container mx-auto px-4 py-16 max-w-4xl text-center">
                                    <h1 class="text-4xl font-bold mb-4 text-gray-900 dark:text-white">
                                        "Post Not Found"
                                    </h1>
                                    <p class="text-gray-600 dark:text-gray-400 mb-8">
                                        "The blog post you're looking for doesn't exist."
                                    </p>
                                    <a href="/blog" class="btn-primary">
                                        "View All Posts"
                                    </a>
                                </div>
                            </div>
                        }),
                        Err(e) => EitherOf3::C(view! {
                            <div class="min-h-screen bg-gray-50 dark:bg-gray-900">
                                <div class="container mx-auto px-4 py-16 max-w-4xl text-center">
                                    <p class="text-red-600 dark:text-red-400">
                                        "Error: " {e.to_string()}
                                    </p>
                                </div>
                            </div>
                        }),
                    }
                })
            }}
        </Suspense>
    }
}
