#![allow(non_snake_case)]
// Brick-disabled fallbacks intentionally share component bodies with the
// feature-enabled render branches.  When all bricks are enabled, those
// fallback expressions are statically unreachable; keep that configuration
// warning-free without duplicating every component for every feature matrix.
#![allow(unreachable_code)]

use dioxus::prelude::*;
use dioxus_router::{Link, Routable, Router};
use tartan_ui_core::NavigationLink;
use tartan_ui_dioxus::{CardGrid, EmptyState, LoadingState, NavigationList, ProductShell};

#[cfg(any(feature = "web", feature = "server"))]
mod loaders;
#[cfg(any(feature = "web", feature = "server"))]
pub mod page_cache;

/// The complete public page route contract. Brick-specific variants are
/// feature-gated here so reduced builds do not expose dead routes, while the
/// enum remains a single source of truth for navigation and SSR.
#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/about")]
    About {},
    #[route("/support")]
    Support {},
    #[cfg(feature = "brick-blog")]
    #[route("/posts")]
    Posts {},
    #[cfg(feature = "brick-blog")]
    #[route("/posts/:slug")]
    Post { slug: String },
    #[cfg(feature = "brick-blog")]
    #[route("/posts/tag/:tag")]
    PostTag { tag: String },
    #[cfg(feature = "brick-blog")]
    #[route("/series")]
    Series {},
    #[cfg(feature = "brick-blog")]
    #[route("/series/:slug")]
    SeriesDetail { slug: String },
    #[cfg(feature = "brick-portfolio")]
    #[route("/projects")]
    Projects {},
    #[cfg(feature = "brick-portfolio")]
    #[route("/projects/:slug")]
    Project { slug: String },
    #[cfg(feature = "brick-activity")]
    #[route("/activity")]
    Activity {},
    #[cfg(feature = "brick-activity")]
    #[route("/activity/:id")]
    ActivityDetail { id: i64 },
    #[cfg(feature = "brick-todo")]
    #[route("/todos")]
    Todos {},
    #[cfg(feature = "brick-todo")]
    #[route("/todos/tag/:tag")]
    TodoTag { tag: String },
    #[cfg(feature = "brick-todo")]
    #[route("/todos/:slug")]
    Todo { slug: String },
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}

/// Dioxus root component. The server and browser targets share this exact
/// component tree so hydration receives the same route/document contract.
#[component]
pub fn App() -> Element {
    #[cfg(any(feature = "web", feature = "server"))]
    let site_config = use_loader(loaders::load_site_config)?;
    #[cfg(any(feature = "web", feature = "server"))]
    use_context_provider(|| site_config);
    #[cfg(any(feature = "web", feature = "server"))]
    let description = site_config().description.clone();
    #[cfg(any(feature = "web", feature = "server"))]
    let favicon = site_config().favicon.clone();
    #[cfg(any(feature = "web", feature = "server"))]
    let favicon_link = favicon.map(|favicon| rsx! { link { rel: "icon", href: "{favicon}" } });
    #[cfg(not(any(feature = "web", feature = "server")))]
    let description =
        "A self-hosted personal website platform built with Rust and Dioxus.".to_string();
    #[cfg(not(any(feature = "web", feature = "server")))]
    let favicon_link = rsx! {};

    rsx! {
        document::Meta { charset: "utf-8" }
        document::Meta {
            name: "viewport",
            content: "width=device-width, initial-scale=1.0",
        }
        document::Meta {
            name: "description",
            content: description,
        }
        {favicon_link}
        Router::<Route> {}
    }
}

#[component]
fn Shell(title: String, children: Element) -> Element {
    #[cfg(any(feature = "web", feature = "server"))]
    let site_config = use_context::<dioxus::fullstack::Loader<plinth_shared::SiteConfig>>();
    #[cfg(any(feature = "web", feature = "server"))]
    let site_name = site_config().name.clone();
    #[cfg(not(any(feature = "web", feature = "server")))]
    let site_name = "Plinth".to_string();

    let mut navigation = vec![
        NavigationLink {
            key: "home".to_string(),
            label: "Home".to_string(),
            href: "/".to_string(),
            current: title == "Home",
        },
        NavigationLink {
            key: "about".to_string(),
            label: "About".to_string(),
            href: "/about".to_string(),
            current: title == "About",
        },
        NavigationLink {
            key: "support".to_string(),
            label: "Support".to_string(),
            href: "/support".to_string(),
            current: title == "Support",
        },
    ];
    #[cfg(feature = "brick-blog")]
    navigation.push(NavigationLink {
        key: "posts".to_string(),
        label: "Posts".to_string(),
        href: "/posts".to_string(),
        current: title == "Posts",
    });
    #[cfg(feature = "brick-portfolio")]
    navigation.push(NavigationLink {
        key: "projects".to_string(),
        label: "Projects".to_string(),
        href: "/projects".to_string(),
        current: title == "Projects",
    });
    #[cfg(feature = "brick-activity")]
    navigation.push(NavigationLink {
        key: "activity".to_string(),
        label: "Activity".to_string(),
        href: "/activity".to_string(),
        current: title == "Activity",
    });

    rsx! {
        ProductShell {
            title: format!("{title} - {site_name}"),
            brand: site_name,
            home_href: "/".to_string(),
            identity: None,
            div { class: "container mx-auto flex flex-wrap items-center justify-between gap-3 px-4 py-4",
                NavigationList { items: navigation, aria_label: "Primary navigation".to_string() }
                ThemeToggle {}
                MobileMenu {}
            }
            div { class: "container mx-auto max-w-6xl px-4 py-12", {children} }
            footer { class: "border-t border-gray-200 px-4 py-6 text-center text-sm text-gray-500 dark:border-amber-900/30 dark:text-amber-500",
                "Built with Plinth"
            }
        }
    }
}

#[component]
fn ThemeToggle() -> Element {
    let mut dark = use_signal(|| true);

    #[cfg(feature = "web")]
    use_effect(move || {
        use web_sys::wasm_bindgen::JsCast;

        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Some(html) = document.document_element() else {
            return;
        };
        let Ok(html) = html.dyn_into::<web_sys::HtmlElement>() else {
            return;
        };
        let _ = html.class_list().toggle_with_force("dark", dark());
    });

    rsx! {
        button {
            r#type: "button",
            class: "rounded px-2 py-1 text-sm hover:bg-gray-100 dark:hover:bg-gray-900",
            aria_label: "Toggle dark mode",
            onclick: move |_| dark.toggle(),
            if dark() { "☀" } else { "☾" }
        }
    }
}

#[component]
fn MobileMenu() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        div { class: "md:hidden",
            button {
                r#type: "button",
                aria_label: "Toggle navigation menu",
                aria_expanded: "{open}",
                class: "rounded px-2 py-1",
                onclick: move |_| open.toggle(),
                "☰"
            }
            if open() {
                div { class: "absolute right-4 top-16 flex flex-col gap-3 rounded border border-gray-200 bg-white p-4 shadow-lg dark:border-amber-900/30 dark:bg-black",
                    MobileBlogLink { close: move || open.set(false) }
                    MobilePortfolioLink { close: move || open.set(false) }
                    MobileActivityLink { close: move || open.set(false) }
                    Link { to: Route::About {}, onclick: move |_| open.set(false), "About" }
                }
            }
        }
    }
}

#[component]
fn MobileBlogLink(close: EventHandler<()>) -> Element {
    #[cfg(feature = "brick-blog")]
    return rsx! { Link { to: Route::Posts {}, onclick: move |_| close.call(()), "Posts" } };
    rsx! {}
}

#[component]
fn MobilePortfolioLink(close: EventHandler<()>) -> Element {
    #[cfg(feature = "brick-portfolio")]
    return rsx! { Link { to: Route::Projects {}, onclick: move |_| close.call(()), "Projects" } };
    rsx! {}
}

#[component]
fn MobileActivityLink(close: EventHandler<()>) -> Element {
    #[cfg(feature = "brick-activity")]
    return rsx! { Link { to: Route::Activity {}, onclick: move |_| close.call(()), "Activity" } };
    rsx! {}
}

#[component]
fn HomeBlogCard() -> Element {
    #[cfg(feature = "brick-blog")]
    return rsx! {
        SummaryCard {
            title: "Posts",
            text: "Writing and notes from the blog.".to_string(),
            to: Route::Posts {},
        }
    };
    rsx! {}
}

#[component]
fn HomePortfolioCard() -> Element {
    #[cfg(feature = "brick-portfolio")]
    return rsx! {
        SummaryCard {
            title: "Projects",
            text: "Tools and software projects.".to_string(),
            to: Route::Projects {},
        }
    };
    rsx! {}
}

#[component]
fn HomeActivityCard() -> Element {
    #[cfg(feature = "brick-activity")]
    return rsx! {
        SummaryCard {
            title: "Activity",
            text: "Recent open-source activity.".to_string(),
            to: Route::Activity {},
        }
    };
    rsx! {}
}

#[component]
fn Home() -> Element {
    rsx! {
        Shell { title: "Home".to_string(),
            section { class: "space-y-6",
                h1 { class: "text-5xl font-bold", "Plinth" }
                HomeIntro {}
                CardGrid {
                    HomeBlogCard {}
                    HomePortfolioCard {}
                    HomeActivityCard {}
                }
                HomeRecentPosts {}
                HomeFeaturedProjects {}
                HomeRecentActivity {}
            }
        }
    }
}

#[component]
fn HomeIntro() -> Element {
    #[cfg(any(feature = "web", feature = "server"))]
    {
        let intro = use_server_future(|| loaders::load_site_content("home-intro".to_string()))?;
        return rsx! {
            SuspenseBoundary {
                fallback: |_| rsx! { LoadingState { label: "Loading introduction…".to_string() } },
                {if let Some(Ok(Some(content))) = intro() {
                    rsx! { div { class: "prose prose-lg max-w-2xl dark:prose-invert", dangerous_inner_html: content.html_content } }
                } else {
                    rsx! { p { class: "max-w-2xl text-xl text-gray-600 dark:text-amber-200", "A self-hosted personal website platform for publishing rich content, projects, and activity." } }
                }}
            }
        };
    }
    rsx! { p { class: "max-w-2xl text-xl text-gray-600 dark:text-amber-200", "A self-hosted personal website platform for publishing rich content, projects, and activity." } }
}

#[component]
fn HomeRecentPosts() -> Element {
    #[cfg(feature = "brick-blog")]
    {
        let posts = use_server_future(loaders::load_posts)?;
        return rsx! {
            SuspenseBoundary {
                fallback: |_| rsx! { LoadingState { label: "Loading posts…".to_string() } },
                {if let Some(Ok(posts)) = posts() {
                    rsx! {
                        section { class: "space-y-4",
                            h2 { class: "text-sm font-semibold uppercase tracking-wider text-gray-500 dark:text-amber-400", "Recent Posts" }
                            div { class: "space-y-2",
                                {posts.iter().take(3).map(|post| rsx! {
                                    Link { to: Route::Post { slug: post.slug.clone() }, class: "block rounded border border-gray-200 bg-white p-4 hover:shadow-sm dark:border-amber-900/30 dark:bg-gray-950",
                                        h3 { class: "font-semibold", "{post.title}" }
                                        p { class: "text-sm text-gray-600 dark:text-amber-300", "{post.description}" }
                                    }
                                })}
                            }
                        }
                    }
                } else { rsx! {} }}
            }
        };
    }
    rsx! {}
}

#[component]
fn HomeFeaturedProjects() -> Element {
    #[cfg(feature = "brick-portfolio")]
    {
        let projects = use_server_future(loaders::load_projects)?;
        return rsx! {
            SuspenseBoundary {
                fallback: |_| rsx! { LoadingState { label: "Loading projects…".to_string() } },
                {if let Some(Ok(projects)) = projects() {
                    rsx! {
                        section { class: "space-y-4",
                            h2 { class: "text-sm font-semibold uppercase tracking-wider text-gray-500 dark:text-amber-400", "Featured Projects" }
                            CardGrid {
                                {projects.iter().filter(|project| project.featured).take(3).map(|project| rsx! {
                                    Link { to: Route::Project { slug: project.slug.clone() }, class: "rounded border border-gray-200 bg-white p-4 hover:shadow-sm dark:border-amber-900/30 dark:bg-gray-950",
                                        h3 { class: "font-semibold", "{project.title}" }
                                        p { class: "text-sm text-gray-600 dark:text-amber-300", "{project.description}" }
                                    }
                                })}
                            }
                        }
                    }
                } else { rsx! {} }}
            }
        };
    }
    rsx! {}
}

#[component]
fn HomeRecentActivity() -> Element {
    #[cfg(feature = "brick-activity")]
    {
        let activity = use_server_future(loaders::load_activity)?;
        return rsx! {
            SuspenseBoundary {
                fallback: |_| rsx! { LoadingState { label: "Loading activity…".to_string() } },
                {if let Some(Ok(activity)) = activity() {
                    rsx! {
                        section { class: "space-y-4",
                            h2 { class: "text-sm font-semibold uppercase tracking-wider text-gray-500 dark:text-amber-400", "Recent Activity" }
                            div { class: "space-y-2",
                                {activity.iter().take(3).map(|item| rsx! {
                                    Link { to: Route::ActivityDetail { id: item.id }, class: "block rounded border border-gray-200 bg-white p-4 hover:shadow-sm dark:border-amber-900/30 dark:bg-gray-950",
                                        h3 { class: "font-semibold", "{item.title}" }
                                        p { class: "text-sm text-gray-600 dark:text-amber-300", "{item.repo_owner}/{item.repo_name}" }
                                    }
                                })}
                            }
                        }
                    }
                } else { rsx! {} }}
            }
        };
    }
    rsx! {}
}

#[component]
fn SummaryCard<R: Routable + Clone + PartialEq + 'static>(
    title: &'static str,
    text: String,
    to: R,
) -> Element {
    rsx! {
        Link { to,
            class: "rounded-lg border border-gray-200 bg-white p-6 shadow-sm transition hover:-translate-y-0.5 hover:shadow-md dark:border-amber-900/30 dark:bg-gray-950",
            h2 { class: "mb-2 text-xl font-semibold", "{title}" }
            p { class: "text-gray-600 dark:text-amber-300", "{text}" }
        }
    }
}

#[component]
fn ContentPage(title: String, body: String) -> Element {
    rsx! {
        Shell { title: title.clone(),
            article { class: "prose prose-lg dark:prose-invert",
                h1 { "{title}" }
                p { "{body}" }
            }
        }
    }
}

#[component]
fn SiteContentPage(content_key: String, title: String, fallback: String) -> Element {
    #[cfg(any(feature = "web", feature = "server"))]
    {
        let content = use_loader({
            let key = content_key.clone();
            move || loaders::load_site_content(key.clone())
        })?;
        if let Some(content) = content().clone() {
            let content_title = content.title.clone().unwrap_or_else(|| title.clone());
            let html_content = content.html_content.clone();
            return rsx! {
                Shell { title: title.clone(),
                    article { class: "prose prose-lg dark:prose-invert",
                        h1 { "{content_title}" }
                        div { dangerous_inner_html: html_content }
                    }
                }
            };
        }
    }

    rsx! { ContentPage { title, body: fallback } }
}

#[component]
fn NotFoundPage(path: String) -> Element {
    rsx! {
        Shell { title: "Not found".to_string(),
            h1 { class: "text-4xl font-bold", "Page not found" }
            p { class: "mt-4", "No route matches {path}." }
            Link { class: "mt-6 inline-block underline", to: Route::Home {}, "Return home" }
        }
    }
}

/// Render a missing content record with an actual HTTP 404 during SSR.
///
/// A route can match syntactically while its database record is absent; that
/// case must not be mistaken for a successful cached page.
#[component]
fn MissingPage(path: String) -> Element {
    #[cfg(feature = "server")]
    dioxus::fullstack::FullstackContext::commit_http_status(
        axum::http::StatusCode::NOT_FOUND,
        Some("content not found".to_string()),
    );
    rsx! { NotFoundPage { path } }
}

#[component]
fn About() -> Element {
    rsx! { SiteContentPage {
        content_key: "about".to_string(),
        title: "About".to_string(),
        fallback: "Learn more about the person and work behind this site.".to_string()
    } }
}

#[component]
fn Support() -> Element {
    rsx! { SiteContentPage {
        content_key: "support".to_string(),
        title: "Support".to_string(),
        fallback: "Support the work behind Plinth.".to_string()
    } }
}

#[component]
fn Posts() -> Element {
    #[cfg(feature = "brick-blog")]
    {
        let posts = use_loader(loaders::load_posts)?;
        let posts_value = posts().clone();
        return rsx! {
            Shell { title: "Posts".to_string(),
                section { class: "space-y-8",
                    h1 { class: "text-4xl font-bold", "Posts" }
                    if posts_value.is_empty() {
                        EmptyState {
                            heading: "No published posts yet".to_string(),
                            message: "Check back soon for new writing and notes.".to_string(),
                        }
                    }
                    div { class: "grid gap-6 md:grid-cols-2",
                        {posts_value.iter().map(|post| rsx! {
                            article { class: "rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-amber-900/30 dark:bg-gray-950",
                                Link { to: Route::Post { slug: post.slug.clone() },
                                    h2 { class: "text-2xl font-semibold hover:underline", "{post.title}" }
                                }
                                p { class: "mt-2 text-gray-600 dark:text-amber-300", "{post.description}" }
                                div { class: "mt-4 flex flex-wrap gap-2 text-sm text-gray-500 dark:text-amber-500",
                                    span { "{post.author}" }
                                    span { "·" }
                                    span { "{post.reading_time_minutes} min read" }
                                }
                            }
                        })}
                    }
                }
            }
        };
    }

    rsx! { ContentPage { title: "Posts".to_string(), body: "The blog brick is disabled in this build.".to_string() } }
}

#[component]
fn Post(slug: String) -> Element {
    #[cfg(feature = "brick-blog")]
    {
        let post = use_loader({
            let slug = slug.clone();
            move || loaders::load_post(slug.clone())
        })?;
        let Some(post) = post().clone() else {
            return rsx! { MissingPage { path: format!("/posts/{slug}") } };
        };
        return rsx! {
            Shell { title: post.title.clone(),
                article { class: "prose prose-lg max-w-3xl dark:prose-invert",
                    p { class: "not-prose text-sm text-gray-500 dark:text-amber-500", "{post.author} · {post.reading_time_minutes} min read" }
                    h1 { "{post.title}" }
                    p { class: "lead", "{post.description}" }
                    div { dangerous_inner_html: post.html_content }
                    if !post.tags.is_empty() {
                        footer { class: "not-prose mt-8 flex flex-wrap gap-2",
                            {post.tags.iter().map(|tag| rsx! {
                                Link { to: Route::PostTag { tag: tag.clone() }, class: "rounded bg-gray-100 px-2 py-1 text-sm dark:bg-amber-950", "#{tag}" }
                            })}
                        }
                    }
                }
            }
        };
    }

    rsx! { ContentPage { title: slug, body: "The blog brick is disabled in this build.".to_string() } }
}

#[component]
fn PostTag(tag: String) -> Element {
    #[cfg(feature = "brick-blog")]
    {
        let posts = use_loader({
            let tag = tag.clone();
            move || loaders::load_posts_by_tag(tag.clone())
        })?;
        let posts_value = posts().clone();
        return rsx! {
            Shell { title: format!("Posts tagged {tag}"),
                section { class: "space-y-8",
                    h1 { class: "text-4xl font-bold", "Posts tagged #{tag}" }
                    div { class: "grid gap-6 md:grid-cols-2",
                        {posts_value.iter().map(|post| rsx! {
                            article { class: "rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-amber-900/30 dark:bg-gray-950",
                                Link { to: Route::Post { slug: post.slug.clone() }, h2 { class: "text-2xl font-semibold hover:underline", "{post.title}" } }
                                p { class: "mt-2 text-gray-600 dark:text-amber-300", "{post.description}" }
                            }
                        })}
                    }
                }
            }
        };
    }

    rsx! { ContentPage { title: format!("Posts tagged {tag}"), body: "The blog brick is disabled in this build.".to_string() } }
}

#[component]
fn Series() -> Element {
    #[cfg(feature = "brick-blog")]
    {
        let series = use_loader(loaders::load_series)?;
        let series_value = series().clone();
        return rsx! {
            Shell { title: "Series".to_string(),
                section { class: "space-y-8",
                    h1 { class: "text-4xl font-bold", "Series" }
                    div { class: "grid gap-6 md:grid-cols-2",
                        {series_value.iter().map(|item| rsx! {
                            Link { to: Route::SeriesDetail { slug: item.slug.clone() }, class: "rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-amber-900/30 dark:bg-gray-950",
                                h2 { class: "text-2xl font-semibold", "{item.title}" }
                                p { class: "mt-2 text-gray-600 dark:text-amber-300", "{item.post_count} posts · {item.total_reading_time} min" }
                            }
                        })}
                    }
                }
            }
        };
    }

    rsx! { ContentPage { title: "Series".to_string(), body: "The blog brick is disabled in this build.".to_string() } }
}

#[component]
fn SeriesDetail(slug: String) -> Element {
    #[cfg(feature = "brick-blog")]
    {
        let posts = use_loader({
            let slug = slug.clone();
            move || loaders::load_series_posts(slug.clone())
        })?;
        let posts_value = posts().clone();
        return rsx! {
            Shell { title: slug.clone(),
                section { class: "space-y-8",
                    h1 { class: "text-4xl font-bold", "{slug}" }
                    ol { class: "space-y-4",
                        {posts_value.iter().map(|post| rsx! {
                            li { class: "rounded-lg border border-gray-200 bg-white p-5 dark:border-amber-900/30 dark:bg-gray-950",
                                Link { to: Route::Post { slug: post.slug.clone() }, h2 { class: "text-xl font-semibold hover:underline", "{post.title}" } }
                                p { class: "mt-1 text-gray-600 dark:text-amber-300", "{post.description}" }
                            }
                        })}
                    }
                }
            }
        };
    }

    rsx! { ContentPage { title: slug, body: "The blog brick is disabled in this build.".to_string() } }
}

#[component]
fn Projects() -> Element {
    #[cfg(feature = "brick-portfolio")]
    {
        let projects = use_loader(loaders::load_projects)?;
        let projects_value = projects().clone();
        return rsx! {
            Shell { title: "Projects".to_string(),
                section { class: "space-y-8",
                    h1 { class: "text-4xl font-bold", "Projects" }
                    div { class: "grid gap-6 md:grid-cols-3",
                        {projects_value.iter().map(|project| rsx! {
                            article { class: "overflow-hidden rounded-lg border border-gray-200 bg-white shadow-sm dark:border-amber-900/30 dark:bg-gray-950",
                                if let Some(image) = &project.image_url {
                                    img { src: "{image}", alt: "{project.title}", class: "h-44 w-full object-cover" }
                                }
                                div { class: "p-6",
                                    Link { to: Route::Project { slug: project.slug.clone() }, h2 { class: "text-xl font-semibold hover:underline", "{project.title}" } }
                                    p { class: "mt-2 text-gray-600 dark:text-amber-300", "{project.description}" }
                                    p { class: "mt-4 text-sm text-gray-500 dark:text-amber-500", "{project.tech_stack.join(\", \")}" }
                                }
                            }
                        })}
                    }
                }
            }
        };
    }

    rsx! { ContentPage { title: "Projects".to_string(), body: "The portfolio brick is disabled in this build.".to_string() } }
}

#[component]
fn Project(slug: String) -> Element {
    #[cfg(feature = "brick-portfolio")]
    {
        let project = use_loader({
            let slug = slug.clone();
            move || loaders::load_project(slug.clone())
        })?;
        let Some(project) = project().clone() else {
            return rsx! { MissingPage { path: format!("/projects/{slug}") } };
        };
        return rsx! {
            Shell { title: project.title.clone(),
                article { class: "prose prose-lg max-w-3xl dark:prose-invert",
                    h1 { "{project.title}" }
                    p { class: "lead", "{project.description}" }
                    if let Some(html) = project.html_content {
                        div { dangerous_inner_html: html }
                    }
                    if !project.tech_stack.is_empty() {
                        p { class: "not-prose text-sm text-gray-500 dark:text-amber-500", "{project.tech_stack.join(\", \")}" }
                    }
                    div { class: "not-prose flex flex-wrap gap-4",
                        if let Some(link) = project.link {
                            a { href: "{link}", target: "_blank", rel: "noreferrer", class: "underline", "Source" }
                        }
                        if let Some(demo) = project.demo {
                            a { href: "{demo}", target: "_blank", rel: "noreferrer", class: "underline", "Demo" }
                        }
                    }
                }
            }
        };
    }

    rsx! { ContentPage { title: slug, body: "The portfolio brick is disabled in this build.".to_string() } }
}

#[component]
fn Activity() -> Element {
    #[cfg(feature = "brick-activity")]
    {
        let activity = use_loader(loaders::load_activity)?;
        let activity_value = activity().clone();
        return rsx! {
            Shell { title: "Activity".to_string(),
                section { class: "space-y-8",
                    h1 { class: "text-4xl font-bold", "Activity" }
                    div { class: "space-y-4",
                        {activity_value.iter().map(|item| rsx! {
                            article { class: "rounded-lg border border-gray-200 bg-white p-5 dark:border-amber-900/30 dark:bg-gray-950",
                                Link { to: Route::ActivityDetail { id: item.id }, h2 { class: "text-xl font-semibold hover:underline", "{item.title}" } }
                                p { class: "mt-1 text-sm text-gray-500 dark:text-amber-500", "{item.repo_owner}/{item.repo_name} · {item.kind:?} · {item.state:?}" }
                            }
                        })}
                    }
                }
            }
        };
    }

    rsx! { ContentPage { title: "Activity".to_string(), body: "The activity brick is disabled in this build.".to_string() } }
}

#[component]
fn ActivityDetail(id: i64) -> Element {
    #[cfg(feature = "brick-activity")]
    {
        let item = use_loader(move || loaders::load_activity_item(id))?;
        let Some(item) = item().clone() else {
            return rsx! { MissingPage { path: format!("/activity/{id}") } };
        };
        return rsx! {
            Shell { title: item.title.clone(),
                article { class: "prose prose-lg max-w-3xl dark:prose-invert",
                    h1 { "{item.title}" }
                    p { "{item.repo_owner}/{item.repo_name} · {item.kind:?} · {item.state:?}" }
                    if let Some(body) = item.body {
                        p { "{body}" }
                    }
                    a { href: "{item.url}", target: "_blank", rel: "noreferrer", "Open on forge" }
                }
            }
        };
    }

    rsx! { ContentPage { title: format!("Activity {id}"), body: "The activity brick is disabled in this build.".to_string() } }
}

#[component]
fn Todos() -> Element {
    #[cfg(feature = "brick-todo")]
    {
        let todos = use_loader(loaders::load_todos)?;
        let todos_value = todos().clone();
        return rsx! {
            Shell { title: "Todos".to_string(),
                section { class: "space-y-8",
                    h1 { class: "text-4xl font-bold", "Todos" }
                    div { class: "space-y-3",
                        {todos_value.iter().map(|todo| rsx! {
                            article { class: "rounded-lg border border-gray-200 bg-white p-5 dark:border-amber-900/30 dark:bg-gray-950",
                                Link { to: Route::Todo { slug: todo.slug.clone() }, h2 { class: "text-xl font-semibold hover:underline", "{todo.title}" } }
                                p { class: "mt-1 text-gray-600 dark:text-amber-300", "{todo.description}" }
                                if todo.completed { p { class: "mt-2 text-sm text-green-700 dark:text-green-400", "Completed" } }
                            }
                        })}
                    }
                }
            }
        };
    }

    rsx! { ContentPage { title: "Todos".to_string(), body: "The todo brick is disabled in this build.".to_string() } }
}

#[component]
fn TodoTag(tag: String) -> Element {
    #[cfg(feature = "brick-todo")]
    {
        let todos = use_loader({
            let tag = tag.clone();
            move || loaders::load_todos_by_tag(tag.clone())
        })?;
        let todos_value = todos().clone();
        return rsx! {
            Shell { title: format!("Todos tagged {tag}"),
                section { class: "space-y-8",
                    h1 { class: "text-4xl font-bold", "Todos tagged #{tag}" }
                    div { class: "space-y-3",
                        {todos_value.iter().map(|todo| rsx! {
                            article { class: "rounded-lg border border-gray-200 bg-white p-5 dark:border-amber-900/30 dark:bg-gray-950",
                                Link { to: Route::Todo { slug: todo.slug.clone() }, h2 { class: "text-xl font-semibold hover:underline", "{todo.title}" } }
                                p { class: "mt-1 text-gray-600 dark:text-amber-300", "{todo.description}" }
                            }
                        })}
                    }
                }
            }
        };
    }

    rsx! { ContentPage { title: format!("Todos tagged {tag}"), body: "The todo brick is disabled in this build.".to_string() } }
}

#[component]
fn Todo(slug: String) -> Element {
    #[cfg(feature = "brick-todo")]
    {
        let todo = use_loader({
            let slug = slug.clone();
            move || loaders::load_todo(slug.clone())
        })?;
        let Some(todo) = todo().clone() else {
            return rsx! { MissingPage { path: format!("/todos/{slug}") } };
        };
        return rsx! {
            Shell { title: todo.title.clone(),
                article { class: "prose prose-lg max-w-3xl dark:prose-invert",
                    h1 { "{todo.title}" }
                    p { class: "lead", "{todo.description}" }
                    if let Some(html) = todo.html_content {
                        div { dangerous_inner_html: html }
                    }
                    if todo.completed { p { class: "not-prose text-green-700 dark:text-green-400", "Completed" } }
                }
            }
        };
    }

    rsx! { ContentPage { title: slug, body: "The todo brick is disabled in this build.".to_string() } }
}

#[component]
fn NotFound(segments: Vec<String>) -> Element {
    #[cfg(feature = "server")]
    dioxus::fullstack::FullstackContext::commit_http_status(
        axum::http::StatusCode::NOT_FOUND,
        Some("route not found".to_string()),
    );
    rsx! { NotFoundPage { path: format!("/{}", segments.join("/")) } }
}
