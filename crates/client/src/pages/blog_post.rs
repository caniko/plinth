use crate::api;
use crate::app::use_site_config;
use crate::components::{ErrorMessage, SupportCta};
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

    let series_nav = Resource::new(slug, |slug| async move { api::get_series_nav(slug).await });

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
                            let og_description = if post.description.is_empty() {
                                post.content.chars().take(160).collect::<String>()
                            } else {
                                post.description.clone()
                            };
                            let canonical_url = if config.base_url.is_empty() {
                                format!("/posts/{}", post.slug)
                            } else {
                                format!("{}/posts/{}", config.base_url, post.slug)
                            };
                            EitherOf3::A(view! {
                                <Title text={format!("{} - {}", post.title, config.name)}/>
                                <Meta name="description" content={og_description.clone()}/>
                                // Open Graph
                                <Meta property="og:title" content={post.title.clone()}/>
                                <Meta property="og:description" content={og_description.clone()}/>
                                <Meta property="og:type" content="article"/>
                                <Meta property="og:url" content={canonical_url.clone()}/>
                                <Meta property="og:site_name" content={config.name.clone()}/>
                                <Meta property="article:published_time" content={post.published_at.to_rfc3339()}/>
                                <Meta property="article:author" content={post.author.clone()}/>
                                // Twitter Card
                                <Meta name="twitter:card" content="summary"/>
                                <Meta name="twitter:title" content={post.title.clone()}/>
                                <Meta name="twitter:description" content={og_description}/>
                                // Canonical URL
                                <Link rel="canonical" href={canonical_url.clone()}/>

                                // JSON-LD structured data
                                <Script type_="application/ld+json">
                                    {format!(
                                        r#"{{"@context":"https://schema.org","@type":"Article","headline":"{}","description":"{}","datePublished":"{}","author":{{"@type":"Person","name":"{}"}},"url":"{}"{}}}"#,
                                        post.title.replace('"', r#"\""#),
                                        post.description.replace('"', r#"\""#),
                                        post.published_at.to_rfc3339(),
                                        post.author.replace('"', r#"\""#),
                                        canonical_url,
                                        post.updated_at.map(|u: DateTime<Utc>| format!(r#","dateModified":"{}""#, u.to_rfc3339())).unwrap_or_default(),
                                    )}
                                </Script>

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

                                            // Series banner
                                            <Suspense fallback=|| ()>
                                                {move || {
                                                    series_nav.get().map(|result| {
                                                        if let Ok(Some(nav)) = result {
                                                            Some(view! {
                                                                <div class="mb-6 px-4 py-3 bg-blue-50 dark:bg-amber-900/20 border border-blue-200 dark:border-amber-800/50 rounded-lg">
                                                                    <a
                                                                        href={format!("/series/{}", nav.series_slug)}
                                                                        class="text-blue-700 dark:text-amber-300 hover:underline font-medium"
                                                                    >
                                                                        {format!("Part {} of {} \u{2014} {}", nav.current_position, nav.total_published, nav.series_title)}
                                                                    </a>
                                                                </div>
                                                            })
                                                        } else {
                                                            None
                                                        }
                                                    })
                                                }}
                                            </Suspense>

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

                                        // Support CTA (after article content, peak reciprocity moment)
                                        <SupportCta/>

                                        // Series prev/next navigation
                                        <Suspense fallback=|| ()>
                                            {move || {
                                                series_nav.get().map(|result| {
                                                    if let Ok(Some(nav)) = result {
                                                        let prev = nav.prev.clone();
                                                        let next = nav.next.clone();
                                                        let entries = nav.entries.clone();
                                                        let series_slug = nav.series_slug.clone();
                                                        let series_title = nav.series_title.clone();
                                                        Some(view! {
                                                            <div class="mt-8 space-y-6">
                                                                // Prev/Next links
                                                                <nav class="flex justify-between items-center gap-4">
                                                                    <div class="flex-1">
                                                                        {prev.map(|p| view! {
                                                                            <a href={format!("/posts/{}", p.slug)}
                                                                               class="group flex flex-col items-start text-left">
                                                                                <span class="text-xs text-gray-500 dark:text-amber-600">
                                                                                    {format!("\u{2190} Part {}", p.position)}
                                                                                </span>
                                                                                <span class="text-blue-600 dark:text-amber-300 group-hover:underline font-medium">
                                                                                    {p.title}
                                                                                </span>
                                                                            </a>
                                                                        })}
                                                                    </div>
                                                                    <div class="flex-1 text-right">
                                                                        {next.map(|n| view! {
                                                                            <a href={format!("/posts/{}", n.slug)}
                                                                               class="group flex flex-col items-end">
                                                                                <span class="text-xs text-gray-500 dark:text-amber-600">
                                                                                    {format!("Part {} \u{2192}", n.position)}
                                                                                </span>
                                                                                <span class="text-blue-600 dark:text-amber-300 group-hover:underline font-medium">
                                                                                    {n.title}
                                                                                </span>
                                                                            </a>
                                                                        })}
                                                                    </div>
                                                                </nav>

                                                                // Series TOC (collapsible)
                                                                <details class="bg-white dark:bg-black rounded-lg shadow-md p-4">
                                                                    <summary class="cursor-pointer font-medium text-gray-900 dark:text-amber-100">
                                                                        <a href={format!("/series/{}", series_slug)}
                                                                           class="hover:underline">
                                                                            {series_title}
                                                                        </a>
                                                                        " \u{2014} Table of Contents"
                                                                    </summary>
                                                                    <ol class="mt-3 space-y-1 list-decimal list-inside text-sm">
                                                                        {entries.iter().map(|entry| {
                                                                            let is_current = entry.slug == slug();
                                                                            view! {
                                                                                <li class={if is_current { "font-bold text-gray-900 dark:text-amber-100" } else { "text-gray-600 dark:text-amber-400" }}>
                                                                                    {if is_current {
                                                                                        leptos::either::Either::Left(view! {
                                                                                            <span>{entry.title.clone()}</span>
                                                                                        })
                                                                                    } else {
                                                                                        leptos::either::Either::Right(view! {
                                                                                            <a href={format!("/posts/{}", entry.slug)} class="hover:underline">
                                                                                                {entry.title.clone()}
                                                                                            </a>
                                                                                        })
                                                                                    }}
                                                                                </li>
                                                                            }
                                                                        }).collect::<Vec<_>>()}
                                                                    </ol>
                                                                </details>
                                                            </div>
                                                        })
                                                    } else {
                                                        None
                                                    }
                                                })
                                            }}
                                        </Suspense>

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
                        Err(_) => EitherOf3::C(view! {
                            <ErrorMessage/>
                        }),
                    }
                })
            }}
        </Suspense>
    }
}
