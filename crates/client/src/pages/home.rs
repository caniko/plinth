use leptos::either::EitherOf3;
use leptos::prelude::*;
use leptos_meta::*;

use crate::api;
use crate::app::use_site_config;

#[component]
pub fn HomePage() -> impl IntoView {
    let config = use_site_config();

    let title = if config.pages.home.title.is_empty() {
        config.name.clone()
    } else {
        config.pages.home.title.clone()
    };

    let description = if config.pages.home.description.is_empty() {
        config.description.clone()
    } else {
        config.pages.home.description.clone()
    };

    let tagline = config.tagline.clone();

    let intro = Resource::new(
        || (),
        |_| async move { api::get_site_content("home-intro".to_string()).await },
    );

    let tagline_fallback = tagline.clone();
    let tagline_body = tagline.clone();

    view! {
        <Title text={title}/>
        <Meta name="description" content={description}/>

        <div class="min-h-screen bg-gray-50 dark:bg-black">
            <div class="max-w-3xl mx-auto px-4 py-16">
                // Intro
                <Suspense fallback=move || {
                    let tagline = tagline_fallback.clone();
                    view! {
                        <p class="text-lg text-gray-600 dark:text-amber-400 mb-16">
                            {tagline}
                        </p>
                    }
                }>
                    {move || {
                        let tagline = tagline_body.clone();
                        intro.get().map(move |result| {
                            let tagline = tagline.clone();
                            match result {
                                Ok(Some(content)) => EitherOf3::A(view! {
                                    <div class="text-lg text-gray-600 dark:text-amber-400 mb-16"
                                         inner_html={content.html_content}>
                                    </div>
                                }),
                                Ok(None) => EitherOf3::B(view! {
                                    <p class="text-lg text-gray-600 dark:text-amber-400 mb-16">
                                        {tagline}
                                    </p>
                                }),
                                Err(_) => {
                                    let tagline = tagline.clone();
                                    EitherOf3::C(view! {
                                        <p class="text-lg text-gray-600 dark:text-amber-400 mb-16">
                                            {tagline}
                                        </p>
                                    })
                                },
                            }
                        })
                    }}
                </Suspense>

                {blog_section()}
                {portfolio_section()}
                {activity_section()}
            </div>
        </div>
    }
}

/// Recent Posts section — only compiled when brick-blog is enabled.
#[cfg(feature = "brick-blog")]
fn blog_section() -> impl IntoView {
    let blog_posts = Resource::new(|| (), |_| async move { api::get_blog_posts().await });

    view! {
        <section class="mb-16">
            <h2 class="text-sm font-semibold uppercase tracking-wider text-gray-500 dark:text-amber-400 mb-6">
                "Recent Posts"
            </h2>

            <Suspense fallback=move || view! {
                <p class="text-gray-500 dark:text-amber-400">"Loading..."</p>
            }>
                {move || {
                    blog_posts.get().map(|result| {
                        match result {
                            Ok(posts) => {
                                if posts.is_empty() {
                                    EitherOf3::A(view! {
                                        <p class="text-gray-500 dark:text-amber-400">
                                            "No posts yet."
                                        </p>
                                    })
                                } else {
                                    let posts: Vec<_> = posts.into_iter().take(3).collect();
                                    EitherOf3::B(view! {
                                        <div class="space-y-4">
                                            {posts.into_iter().map(|post| {
                                                let slug = post.slug.clone();
                                                view! {
                                                    <a
                                                        href={format!("/posts/{}", slug)}
                                                        class="flex items-baseline justify-between gap-4 group py-2"
                                                    >
                                                        <span class="text-gray-900 dark:text-amber-100 group-hover:text-blue-600 dark:group-hover:text-amber-200 transition-colors">
                                                            {post.title}
                                                        </span>
                                                        <span class="text-sm text-gray-400 dark:text-amber-600 shrink-0">
                                                            {post.published_at.format("%b %Y").to_string()}
                                                        </span>
                                                    </a>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    })
                                }
                            },
                            Err(_) => EitherOf3::C(view! {
                                <p class="text-gray-500 dark:text-amber-400">
                                    "Could not load posts."
                                </p>
                            }),
                        }
                    })
                }}
            </Suspense>

            <a href="/posts" class="inline-block mt-4 text-sm text-blue-600 dark:text-amber-300 hover:underline">
                "All posts \u{2192}"
            </a>
        </section>
    }
}

#[cfg(not(feature = "brick-blog"))]
fn blog_section() -> impl IntoView {
    ()
}

/// Projects section — only compiled when brick-portfolio is enabled.
#[cfg(feature = "brick-portfolio")]
fn portfolio_section() -> impl IntoView {
    let projects = Resource::new(|| (), |_| async move { api::get_portfolio_items().await });

    view! {
        <section>
            <h2 class="text-sm font-semibold uppercase tracking-wider text-gray-500 dark:text-amber-400 mb-6">
                "Projects"
            </h2>

            <Suspense fallback=move || view! {
                <p class="text-gray-500 dark:text-amber-400">"Loading..."</p>
            }>
                {move || {
                    projects.get().map(|result| {
                        match result {
                            Ok(items) => {
                                if items.is_empty() {
                                    EitherOf3::A(view! {
                                        <p class="text-gray-500 dark:text-amber-400">
                                            "No projects yet."
                                        </p>
                                    })
                                } else {
                                    let items: Vec<_> = items.into_iter().take(3).collect();
                                    EitherOf3::B(view! {
                                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                            {items.into_iter().map(|item| {
                                                let slug = item.slug.clone();
                                                view! {
                                                    <a
                                                        href={format!("/projects/{}", slug)}
                                                        class="block p-4 border border-gray-200 dark:border-amber-900/50 rounded-lg hover:border-blue-400 dark:hover:border-amber-500 transition-colors"
                                                    >
                                                        <h3 class="font-semibold text-gray-900 dark:text-amber-100 mb-1">
                                                            {item.title}
                                                        </h3>
                                                        <p class="text-sm text-gray-600 dark:text-amber-400 mb-3">
                                                            {item.description}
                                                        </p>
                                                        <div class="flex flex-wrap gap-1">
                                                            {item.tech_stack.iter().map(|tech| view! {
                                                                <span class="text-xs text-gray-500 dark:text-amber-400">
                                                                    {tech.clone()}
                                                                </span>
                                                            }).collect::<Vec<_>>()}
                                                        </div>
                                                    </a>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    })
                                }
                            },
                            Err(_) => EitherOf3::C(view! {
                                <p class="text-gray-500 dark:text-amber-400">
                                    "Could not load projects."
                                </p>
                            }),
                        }
                    })
                }}
            </Suspense>

            <a href="/projects" class="inline-block mt-4 text-sm text-blue-600 dark:text-amber-300 hover:underline">
                "All projects \u{2192}"
            </a>
        </section>
    }
}

#[cfg(not(feature = "brick-portfolio"))]
fn portfolio_section() -> impl IntoView {
    ()
}

/// Recent Activity strip — top-N by score; only compiled when brick-activity is enabled.
#[cfg(feature = "brick-activity")]
fn activity_section() -> impl IntoView {
    let items = Resource::new(|| (), |_| async move { api::get_activity_list().await });

    view! {
        <section class="mb-16">
            <h2 class="text-sm font-semibold uppercase tracking-wider text-gray-500 dark:text-amber-400 mb-6">
                "Recent Activity"
            </h2>

            <Suspense fallback=move || view! {
                <p class="text-gray-500 dark:text-amber-400">"Loading..."</p>
            }>
                {move || {
                    items.get().map(|result| {
                        match result {
                            Ok(items) => {
                                if items.is_empty() {
                                    EitherOf3::A(view! {
                                        <p class="text-gray-500 dark:text-amber-400">"No activity yet."</p>
                                    })
                                } else {
                                    let items: Vec<_> = items.into_iter().take(4).collect();
                                    EitherOf3::B(view! {
                                        <div class="space-y-4">
                                            {items.into_iter().map(|item| {
                                                let id = item.id;
                                                let repo = format!("{}/{}", item.repo_owner, item.repo_name);
                                                view! {
                                                    <a
                                                        href={format!("/activity/{}", id)}
                                                        class="flex items-baseline justify-between gap-4 group py-2"
                                                    >
                                                        <span class="text-gray-900 dark:text-amber-100 group-hover:text-blue-600 dark:group-hover:text-amber-200 transition-colors">
                                                            {item.title}
                                                        </span>
                                                        <span class="text-sm text-gray-400 dark:text-amber-600 shrink-0">
                                                            {repo}
                                                        </span>
                                                    </a>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    })
                                }
                            },
                            Err(_) => EitherOf3::C(view! {
                                <p class="text-gray-500 dark:text-amber-400">"Could not load activity."</p>
                            }),
                        }
                    })
                }}
            </Suspense>

            <a href="/activity" class="inline-block mt-4 text-sm text-blue-600 dark:text-amber-300 hover:underline">
                "All activity \u{2192}"
            </a>
        </section>
    }
}

#[cfg(not(feature = "brick-activity"))]
fn activity_section() -> impl IntoView {
    ()
}
