use crate::api;
use crate::app::use_site_config;
use chrono::{DateTime, Utc};
use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn BlogPostPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.with(|p| p.get("slug").unwrap_or_default());

    let blog_post = Resource::new(slug, |slug| async move {
        api::get_blog_post_by_slug(slug).await
    });

    view! {
        <Suspense fallback=move || view! {
            <div class="min-h-screen bg-gray-50 dark:bg-black">
                <div class="container mx-auto px-4 py-16 max-w-4xl">
                    <div class="text-center">
                        <p class="text-gray-600 dark:text-amber-400">"Loading post..."</p>
                    </div>
                </div>
            </div>
        }>
            {move || {
                blog_post.get().map(|result| {
                    match result {
                        Ok(Some(post)) => {
                            let config = use_site_config();
                            EitherOf3::A(view! {
                                <Title text={format!("{} - {}", post.title, config.name)}/>
                                <Meta name="description" content={post.content.chars().take(160).collect::<String>()}/>

                                <div class="min-h-screen bg-gray-50 dark:bg-black">
                                    <article class="container mx-auto px-4 py-16 max-w-4xl">
                                        // Back button
                                        <a href="/posts" class="inline-flex items-center text-blue-600 dark:text-amber-300 hover:underline mb-8">
                                            "\u{2190} Back to Posts"
                                        </a>

                                        // Header
                                        <header class="mb-12">
                                            <h1 class="text-5xl font-bold mb-6 text-gray-900 dark:text-amber-100 leading-tight">
                                                {post.title.clone()}
                                            </h1>

                                            <div class="flex flex-wrap items-center gap-4 text-gray-600 dark:text-amber-400 mb-6">
                                                <time>{post.published_at.format("%B %d, %Y").to_string()}</time>
                                                <span>"\u{2022}"</span>
                                                <span>{post.reading_time_minutes} " min read"</span>
                                                <span>"\u{2022}"</span>
                                                <span>{post.author.clone()}</span>
                                            </div>

                                            <div class="flex flex-wrap gap-2">
                                                {post.tags.iter().map(|tag: &String| view! {
                                                    <a
                                                        href={format!("/posts/tag/{}", tag)}
                                                        class="px-3 py-1 bg-blue-100 dark:bg-amber-900/30 text-blue-800 dark:text-amber-200 rounded-full text-sm hover:bg-blue-200 dark:hover:bg-amber-800/30 transition-colors"
                                                    >
                                                        {tag.clone()}
                                                    </a>
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        </header>

                                        // Content
                                        <div class="prose prose-lg dark:prose-invert max-w-none bg-white dark:bg-black rounded-lg shadow-lg p-8 md:p-12">
                                            <div inner_html={post.html_content}></div>
                                        </div>

                                        // Footer
                                        <footer class="mt-12 pt-8 border-t border-gray-200 dark:border-amber-900/50">
                                            <div class="flex justify-between items-center">
                                                <a href="/posts" class="btn-secondary">
                                                    "\u{2190} All Posts"
                                                </a>
                                                <div class="text-sm text-gray-500 dark:text-amber-600">
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
                            <div class="min-h-screen bg-gray-50 dark:bg-black">
                                <div class="container mx-auto px-4 py-16 max-w-4xl text-center">
                                    <h1 class="text-4xl font-bold mb-4 text-gray-900 dark:text-amber-100">
                                        "Post Not Found"
                                    </h1>
                                    <p class="text-gray-600 dark:text-amber-400 mb-8">
                                        "The post you're looking for doesn't exist."
                                    </p>
                                    <a href="/posts" class="btn-primary">
                                        "View All Posts"
                                    </a>
                                </div>
                            </div>
                        }),
                        Err(e) => EitherOf3::C(view! {
                            <div class="min-h-screen bg-gray-50 dark:bg-black">
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
