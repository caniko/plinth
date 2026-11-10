#![allow(non_snake_case)]
// Brick-disabled fallbacks intentionally share component bodies with the
// feature-enabled render branches.  When all bricks are enabled, those
// fallback expressions are statically unreachable; keep that configuration
// warning-free without duplicating every component for every feature matrix.
#![allow(unreachable_code)]

use dioxus::prelude::*;
use dioxus_router::{Link, Routable, Router};

#[cfg(any(feature = "web", feature = "server"))]
mod loaders;
#[cfg(any(feature = "web", feature = "server"))]
pub mod page_cache;

/// Shared focus treatment: a visible 2px ring offset from the control, amber
/// on the dark canvas and blue on the light one.
const FOCUS_RING: &str = "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600 dark:focus-visible:outline-amber-400";

const NAV_LINK: &str = "rounded-md px-3 py-2 text-[15px] font-medium text-gray-700 hover:bg-gray-100 hover:text-gray-900 motion-safe:transition-colors dark:text-amber-200 dark:hover:bg-white/5 dark:hover:text-amber-100";
const NAV_LINK_CURRENT: &str = "rounded-md bg-gray-200 px-3 py-2 text-[15px] font-medium text-gray-900 dark:bg-amber-950 dark:text-amber-100";

#[cfg_attr(
    not(any(
        feature = "brick-blog",
        feature = "brick-portfolio",
        feature = "brick-activity"
    )),
    allow(dead_code)
)]
const LINK: &str = "text-blue-600 hover:text-blue-700 motion-safe:transition-colors dark:text-amber-300 dark:hover:text-amber-200";
#[cfg(any(feature = "brick-blog", feature = "brick-todo"))]
const CHIP: &str = "inline-flex items-center rounded-full border border-gray-300 px-2.5 py-1 text-xs font-medium text-gray-600 hover:border-gray-400 hover:text-gray-800 motion-safe:transition-colors dark:border-amber-900 dark:text-amber-400 dark:hover:border-amber-700 dark:hover:text-amber-300";
const RULE: &str = "border-gray-200 dark:border-amber-900/50";
const HEADING: &str = "text-gray-900 dark:text-amber-100";
const BODY: &str = "text-gray-700 dark:text-amber-200";
const LEAD: &str = "text-gray-700 dark:text-amber-300";
const META: &str = "text-gray-600 dark:text-amber-500";

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
    let default_theme = site_config().default_theme.clone();
    #[cfg(any(feature = "web", feature = "server"))]
    let favicon = site_config().favicon.clone();
    #[cfg(any(feature = "web", feature = "server"))]
    let favicon_link = favicon.map(|favicon| rsx! { link { rel: "icon", href: "{favicon}" } });
    #[cfg(not(any(feature = "web", feature = "server")))]
    let description =
        "A self-hosted personal website platform built with Rust and Dioxus.".to_string();
    #[cfg(not(any(feature = "web", feature = "server")))]
    let default_theme = "dark".to_string();
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
        document::Stylesheet { href: tartan_ui_dioxus::SHARED_STYLES }
        // Resolve the theme before the first paint. The static shell already
        // starts on `dark`; this only switches users who chose light (or a
        // site whose configured default is light).
        document::Script { {theme_init_script(&default_theme)} }
        {favicon_link}
        Router::<Route> {}
    }
}

/// The inline theme bootstrap. It deliberately avoids `&&` so the SSR text
/// pipeline cannot HTML-escape the script body.
fn theme_init_script(default_theme: &str) -> String {
    let fallback = if default_theme == "light" {
        "light"
    } else {
        "dark"
    };
    format!(
        r#"(function(){{try{{var t=localStorage.getItem("theme");if(t==="dark"){{}}else if(t==="light"){{}}else{{t="{fallback}"}}if(t==="dark"){{document.documentElement.classList.add("dark")}}else{{document.documentElement.classList.remove("dark")}}}}catch(e){{}}}})();"#
    )
}

/// Set the document title without Dioxus' web `Title` implementation, which
/// evaluates JavaScript (`document.title = …`) and therefore requires
/// `'unsafe-eval'` — unavailable under Plinth's CSP.
/// Article bodies can be full HTML documents and often repeat the page title
/// as their first heading (a markdown `# Title`). The detail page already
/// renders that title, so keep the document body and drop the one duplicate
/// leading heading.
#[cfg(any(
    feature = "web",
    feature = "server",
    feature = "brick-blog",
    feature = "brick-portfolio",
    feature = "brick-todo",
    test
))]
fn article_html(html: &str, title: &str) -> String {
    let body = document_body(html).trim_start();
    let Some(rest) = body.strip_prefix('<') else {
        return body.to_string();
    };
    let Some(tag_end) = rest.find('>') else {
        return body.to_string();
    };
    let tag_name = rest[..tag_end]
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(tag_name.as_str(), "h1" | "h2" | "h3") {
        return body.to_string();
    }
    let close = format!("</{tag_name}>");
    let Some(heading_end) = body.find(&close) else {
        return body.to_string();
    };
    let heading = text_without_tags(&body[tag_end + 1..heading_end]);
    if heading.eq_ignore_ascii_case(title.trim()) {
        body[heading_end + close.len()..].trim_start().to_string()
    } else {
        body.to_string()
    }
}

/// Extract `<body>` contents when a renderer produced a complete document.
#[cfg(any(
    feature = "web",
    feature = "server",
    feature = "brick-blog",
    feature = "brick-portfolio",
    feature = "brick-todo",
    test
))]
fn document_body(html: &str) -> &str {
    let trimmed = html.trim_start();
    let Some(index) = trimmed.find("<body") else {
        return trimmed;
    };
    let rest = &trimmed[index..];
    let Some(open_end) = rest.find('>') else {
        return trimmed;
    };
    let after_open = &rest[open_end + 1..];
    match after_open.find("</body>") {
        Some(end) => &after_open[..end],
        None => after_open,
    }
}

#[cfg(any(
    feature = "web",
    feature = "server",
    feature = "brick-blog",
    feature = "brick-portfolio",
    feature = "brick-todo",
    test
))]
fn text_without_tags(fragment: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for character in fragment.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            character if !in_tag => text.push(character),
            _ => {}
        }
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[component]
fn PageTitle(title: String) -> Element {
    #[cfg(feature = "server")]
    {
        return rsx! { document::Title { "{title}" } };
    }
    #[cfg(feature = "web")]
    {
        use_effect(use_reactive!(|title| {
            if let Some(document) = web_sys::window().and_then(|window| window.document()) {
                document.set_title(&title);
            }
        }));
    }
    rsx! {}
}

/// True when a configured navigation path names the current route. Detail
/// pages keep their section link current (`/posts/foo` marks `/posts`).
fn is_current_nav(nav_path: &str, current_path: &str) -> bool {
    if nav_path == "/" {
        return current_path == "/";
    }
    let base = nav_path.trim_end_matches('/');
    current_path == base || current_path.starts_with(&format!("{base}/"))
}

#[component]
fn Shell(title: String, children: Element) -> Element {
    #[cfg(any(feature = "web", feature = "server"))]
    let site_config = use_context::<dioxus::fullstack::Loader<plinth_shared::SiteConfig>>();
    #[cfg(any(feature = "web", feature = "server"))]
    let site_name = site_config().name.clone();
    #[cfg(not(any(feature = "web", feature = "server")))]
    let site_name = "Plinth".to_string();

    rsx! {
        div { class: "min-h-screen bg-white text-gray-700 dark:bg-black dark:text-amber-200",
            PageTitle { title: format!("{title} - {site_name}") }
            a {
                href: "#content",
                class: "sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-4 focus:z-50 focus:rounded-md focus:bg-white focus:px-4 focus:py-2 focus:text-sm focus:font-medium focus:text-gray-900 focus:shadow-lg dark:focus:bg-neutral-950 dark:focus:text-amber-100 {FOCUS_RING}",
                "Skip to content"
            }
            Header {}
            main {
                id: "content",
                class: "mx-auto w-full max-w-4xl px-4 pb-24 pt-10 md:px-6 md:pt-16",
                {children}
            }
            Footer {}
        }
    }
}

#[component]
fn Header() -> Element {
    #[cfg(any(feature = "web", feature = "server"))]
    let site = use_context::<dioxus::fullstack::Loader<plinth_shared::SiteConfig>>()();
    #[cfg(not(any(feature = "web", feature = "server")))]
    let site = plinth_shared::SiteConfig::default();
    let current_path = use_route::<Route>().to_string();
    let logo = site
        .logo
        .clone()
        .unwrap_or_else(|| "/plinth-logo.svg".to_string());

    rsx! {
        header { class: "border-b {RULE}",
            div { class: "group mx-auto w-full max-w-4xl px-4 md:px-6",
                div { class: "flex h-14 items-center justify-between gap-3 md:h-16",
                    a {
                        href: "/",
                        class: "flex min-w-0 items-center gap-2.5 {FOCUS_RING}",
                        img { src: "{logo}", alt: "", class: "h-[26px] w-auto shrink-0" }
                        span { class: "truncate text-[15px] font-semibold text-gray-900 dark:text-amber-100 md:text-base", "{site.name}" }
                    }
                    div { class: "flex shrink-0 items-center gap-2",
                        nav { class: "hidden items-center gap-1 md:flex", aria_label: "Primary",
                            for item in site.nav.iter() {
                                NavLink {
                                    item: item.clone(),
                                    current: is_current_nav(&item.path, &current_path),
                                }
                            }
                        }
                        ThemeToggle {}
                        details { class: "md:hidden",
                            summary {
                                class: "menu-summary flex h-11 w-11 cursor-pointer list-none items-center justify-center rounded-md border border-gray-300 text-gray-700 dark:border-amber-900/60 dark:text-amber-300 {FOCUS_RING}",
                                aria_label: "Menu",
                                svg {
                                    class: "h-5 w-5",
                                    "data-icon": "menu-open",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "1.8",
                                    stroke_linecap: "round",
                                    "aria-hidden": "true",
                                    path { d: "M4 7h16M4 12h16M4 17h16" }
                                }
                                svg {
                                    class: "h-5 w-5",
                                    "data-icon": "menu-close",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "1.8",
                                    stroke_linecap: "round",
                                    "aria-hidden": "true",
                                    path { d: "M6 6l12 12M18 6L6 18" }
                                }
                            }
                        }
                    }
                }
                nav { class: "hidden group-has-[details[open]]:block md:hidden", aria_label: "Primary",
                    for item in site.nav.iter() {
                        MobileNavLink {
                            item: item.clone(),
                            current: is_current_nav(&item.path, &current_path),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NavLink(item: plinth_shared::config::NavItem, current: bool) -> Element {
    if current {
        rsx! {
            a {
                href: "{item.path}",
                class: "motion-safe:transition-colors {NAV_LINK_CURRENT} {FOCUS_RING}",
                aria_current: "page",
                "{item.label}"
            }
        }
    } else {
        rsx! {
            a {
                href: "{item.path}",
                class: "motion-safe:transition-colors {NAV_LINK} {FOCUS_RING}",
                "{item.label}"
            }
        }
    }
}

#[component]
fn MobileNavLink(item: plinth_shared::config::NavItem, current: bool) -> Element {
    if current {
        rsx! {
            a {
                href: "{item.path}",
                class: "block border-b border-b-gray-200 border-l-[3px] border-l-amber-500 bg-gray-100 py-3.5 pl-3 pr-3 text-[15px] font-medium text-gray-900 motion-safe:transition-colors dark:border-b-amber-900/50 dark:border-l-amber-500 dark:bg-neutral-950 dark:text-amber-100 {FOCUS_RING}",
                aria_current: "page",
                onclick: |_| close_mobile_menu(),
                "{item.label}"
            }
        }
    } else {
        rsx! {
            a {
                href: "{item.path}",
                class: "block border-b border-b-gray-200 border-l-[3px] border-l-transparent py-3.5 pl-3 pr-3 text-[15px] text-gray-700 hover:bg-gray-50 motion-safe:transition-colors dark:border-b-amber-900/50 dark:text-amber-200 dark:hover:bg-white/5 {FOCUS_RING}",
                onclick: |_| close_mobile_menu(),
                "{item.label}"
            }
        }
    }
}

/// Close the native `<details>` menu after a hydrated link selection.
fn close_mobile_menu() {
    #[cfg(feature = "web")]
    if let Some(document) = web_sys::window().and_then(|window| window.document())
        && let Ok(Some(details)) = document.query_selector("header details[open]")
    {
        let _ = details.remove_attribute("open");
    }
}

#[component]
fn ThemeToggle() -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "flex h-11 w-11 shrink-0 items-center justify-center rounded-md border border-gray-300 text-gray-700 hover:bg-gray-100 hover:text-gray-900 motion-safe:transition-colors dark:border-amber-900/60 dark:text-amber-300 dark:hover:bg-white/5 dark:hover:text-amber-100 md:h-9 md:w-9 {FOCUS_RING}",
            onclick: move |_| toggle_theme(),
            span { class: "sr-only dark:hidden", "Switch to dark theme" }
            span { class: "sr-only hidden dark:inline", "Switch to light theme" }
            svg {
                class: "h-4 w-4 dark:hidden",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.7",
                stroke_linecap: "round",
                "aria-hidden": "true",
                path { d: "M20.35 15.35A9 9 0 0 1 8.65 3.65 9 9 0 1 0 20.35 15.35z" }
            }
            svg {
                class: "hidden h-4 w-4 dark:block",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "1.7",
                stroke_linecap: "round",
                "aria-hidden": "true",
                circle { cx: "12", cy: "12", r: "4" }
                path { d: "M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" }
            }
        }
    }
}

fn toggle_theme() {
    #[cfg(feature = "web")]
    {
        use web_sys::wasm_bindgen::JsCast;

        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Some(html) = document
            .document_element()
            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
        else {
            return;
        };
        let dark = html.class_list().toggle("dark").unwrap_or(false);
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item("theme", if dark { "dark" } else { "light" });
        }
    }
}

#[component]
fn Footer() -> Element {
    #[cfg(any(feature = "web", feature = "server"))]
    let site = use_context::<dioxus::fullstack::Loader<plinth_shared::SiteConfig>>()();
    #[cfg(not(any(feature = "web", feature = "server")))]
    let site = plinth_shared::SiteConfig::default();

    let social = [
        ("Codeberg", site.social.codeberg.as_str()),
        ("GitHub", site.social.github.as_str()),
        ("GitLab", site.social.gitlab.as_str()),
        ("Mastodon", site.social.mastodon.as_str()),
        ("Bluesky", site.social.bluesky.as_str()),
    ];
    let project_url = site.footer.project_url.clone();
    #[cfg(feature = "brick-blog")]
    let rss = rsx! { a { href: "/feed.xml", class: "{LINK} {FOCUS_RING}", "RSS" } };
    #[cfg(not(feature = "brick-blog"))]
    let rss = rsx! {};

    rsx! {
        footer { class: "border-t {RULE}",
            div { class: "mx-auto flex w-full max-w-4xl flex-wrap items-center gap-x-3 gap-y-2 px-4 py-8 text-sm md:px-6",
                nav { class: "order-1 flex w-full flex-wrap items-center gap-x-5 gap-y-1 md:order-2 md:ml-5 md:w-auto", aria_label: "Elsewhere",
                    for (label, url) in social.iter() {
                        if !url.is_empty() {
                            a {
                                href: "{url}",
                                rel: "me noopener",
                                class: "{LINK} {FOCUS_RING}",
                                "{label}"
                            }
                        }
                    }
                    {rss}
                }
                span { class: "order-2 truncate {META} md:order-1", "{site.name}" }
                span { class: "order-3 md:hidden {META}", "·" }
                if !site.footer.project_name.is_empty() {
                    a {
                        href: "{project_url}",
                        class: "order-4 {LEAD} hover:underline md:order-3 md:ml-auto {FOCUS_RING}",
                        "Built with {site.footer.project_name}"
                    }
                }
            }
        }
    }
}

#[component]
fn PageHeader(title: String, #[props(default)] subtitle: String) -> Element {
    rsx! {
        header { class: "pb-10",
            h1 { class: "text-4xl font-bold tracking-tight {HEADING}", "{title}" }
            if !subtitle.is_empty() {
                p { class: "mt-2 text-lg {LEAD}", "{subtitle}" }
            }
        }
    }
}

#[component]
fn SectionHeader(label: String, href: String, action: String) -> Element {
    rsx! {
        h2 { class: "flex items-baseline justify-between gap-4 text-xs font-semibold uppercase leading-4 tracking-[0.16em] {META}",
            "{label}"
            a {
                href: "{href}",
                class: "shrink-0 text-sm font-medium normal-case tracking-normal {LINK} {FOCUS_RING}",
                "{action} \u{2192}"
            }
        }
    }
}

#[cfg(feature = "brick-blog")]
#[component]
fn TagChip(tag: String) -> Element {
    rsx! {
        Link {
            to: Route::PostTag { tag: tag.clone() },
            class: "{CHIP} {FOCUS_RING}",
            "#{tag}"
        }
    }
}

#[cfg(feature = "brick-blog")]
#[component]
fn PostRow(
    post: plinth_shared::BlogListItem,
    #[props(default)] large: bool,
    #[props(default)] tags: bool,
) -> Element {
    let datetime = post.published_at.to_rfc3339();
    let date = post.published_at.format("%b %-d, %Y").to_string();
    let title_class = if large {
        format!(
            "text-xl font-semibold tracking-tight text-gray-900 hover:text-blue-600 motion-safe:transition-colors dark:text-amber-100 dark:hover:text-amber-300 {FOCUS_RING}"
        )
    } else {
        format!(
            "text-lg font-semibold tracking-tight text-gray-900 hover:text-blue-600 motion-safe:transition-colors dark:text-amber-100 dark:hover:text-amber-300 {FOCUS_RING}"
        )
    };

    rsx! {
        article { class: "flex flex-col gap-1 border-t pt-4 pb-4 {RULE} sm:pt-5 sm:pb-5 sm:grid sm:grid-cols-[minmax(0,1fr)_auto] sm:items-baseline sm:gap-x-6 sm:gap-y-1",
            Link {
                to: Route::Post { slug: post.slug.clone() },
                class: "text-balance sm:col-start-1 sm:row-start-1 {title_class}",
                "{post.title}"
            }
            if !post.description.is_empty() {
                p { class: "max-w-2xl text-[15px] leading-normal {BODY} sm:col-start-1 sm:row-start-2", "{post.description}" }
            }
            p { class: "text-sm whitespace-nowrap sm:col-start-2 sm:row-start-1 sm:text-right {META}",
                time { datetime: "{datetime}", "{date}" }
                "\u{a0}· {post.reading_time_minutes} min read"
            }
            if tags && !post.tags.is_empty() {
                div { class: "mt-2 flex flex-wrap gap-2 sm:col-span-2 sm:row-start-3",
                    for tag in post.tags.iter() {
                        TagChip { tag: tag.clone() }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "brick-portfolio")]
#[component]
fn ProjectCard(project: plinth_shared::PortfolioItem, #[props(default)] image: bool) -> Element {
    rsx! {
        Link {
            to: Route::Project { slug: project.slug.clone() },
            class: "flex flex-col rounded-lg border border-gray-200 bg-white p-5 hover:border-gray-400 motion-safe:transition-colors dark:border-amber-900/50 dark:bg-neutral-950 dark:hover:border-amber-700 {FOCUS_RING}",
            if image && let Some(image_url) = project.image_url.clone() {
                img {
                    src: "{image_url}",
                    alt: "",
                    class: "mb-4 h-40 w-full rounded-md object-cover",
                }
            }
            h3 { class: "text-base font-semibold {HEADING}", "{project.title}" }
            p { class: "mt-2 text-[15px] leading-relaxed {BODY}", "{project.description}" }
            if !project.tech_stack.is_empty() {
                div { class: "mt-4 flex flex-wrap gap-2",
                    for tech in project.tech_stack.iter() {
                        span { class: "inline-flex items-center rounded-full border border-gray-300 px-2.5 py-1 text-xs font-medium text-gray-600 dark:border-amber-900 dark:text-amber-400", "{tech}" }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "brick-activity")]
#[component]
fn ActivityRow(item: plinth_shared::ActivityListItem) -> Element {
    rsx! {
        Link {
            to: Route::ActivityDetail { id: item.id },
            class: "flex flex-col gap-1 border-t py-2.5 sm:flex-row sm:items-baseline sm:justify-between sm:gap-6 {RULE} {FOCUS_RING}",
            span { class: "min-w-0 text-[15px] font-medium text-gray-900 hover:text-blue-600 motion-safe:transition-colors dark:text-amber-100 dark:hover:text-amber-300", "{item.title}" }
            span { class: "shrink-0 text-sm {META}", "{item.repo_owner}/{item.repo_name}" }
        }
    }
}

#[component]
fn Home() -> Element {
    #[cfg(any(feature = "web", feature = "server"))]
    let site = use_context::<dioxus::fullstack::Loader<plinth_shared::SiteConfig>>()();
    #[cfg(not(any(feature = "web", feature = "server")))]
    let site = plinth_shared::SiteConfig::default();

    #[cfg(feature = "brick-blog")]
    let recent_posts = rsx! { HomeRecentPosts {} };
    #[cfg(not(feature = "brick-blog"))]
    let recent_posts = rsx! {};
    #[cfg(feature = "brick-portfolio")]
    let featured_projects = rsx! { HomeFeaturedProjects {} };
    #[cfg(not(feature = "brick-portfolio"))]
    let featured_projects = rsx! {};
    #[cfg(feature = "brick-activity")]
    let recent_activity = rsx! { HomeRecentActivity {} };
    #[cfg(not(feature = "brick-activity"))]
    let recent_activity = rsx! {};

    rsx! {
        Shell { title: "Home".to_string(),
            section {
                h1 { class: "text-balance text-[34px] font-bold leading-tight tracking-tight {HEADING} md:mt-1 md:text-5xl", "{site.name}" }
                p { class: "mt-3 text-xl leading-snug {LEAD}", "{site.tagline}" }
                HomeIntro { description: site.description.clone() }
            }
            {recent_posts}
            {featured_projects}
            {recent_activity}
        }
    }
}

#[component]
fn HomeIntro(description: String) -> Element {
    #[cfg(any(feature = "web", feature = "server"))]
    {
        let intro = use_server_future(|| loaders::load_site_content("home-intro".to_string()))?;
        let fallback_description = description.clone();
        return rsx! {
            SuspenseBoundary {
                fallback: move |_| rsx! { p { class: "mt-6 max-w-[40rem] text-base leading-relaxed {BODY}", "{fallback_description}" } },
                {if let Some(Ok(Some(content))) = intro() {
                    rsx! { div { class: "prose mt-4 max-w-[40rem] dark:prose-invert", dangerous_inner_html: content.html_content } }
                } else {
                    rsx! { p { class: "mt-4 max-w-[40rem] text-base leading-snug {BODY}", "{description}" } }
                }}
            }
        };
    }
    rsx! { p { class: "mt-4 max-w-[40rem] text-base leading-snug {BODY}", "{description}" } }
}

#[cfg(feature = "brick-blog")]
#[component]
fn HomeRecentPosts() -> Element {
    let posts = use_server_future(loaders::load_posts)?;
    rsx! {
        SuspenseBoundary {
            fallback: |_| rsx! {},
            {if let Some(Ok(posts)) = posts() {
                if !posts.is_empty() {
                    rsx! {
                        section { class: "mt-12 md:mt-16",
                            SectionHeader {
                                label: "Recent posts".to_string(),
                                href: "/posts".to_string(),
                                action: "All posts".to_string(),
                            }
                            div { class: "mt-2 md:mt-3",
                                for post in posts.iter().take(3) {
                                    PostRow { post: post.clone() }
                                }
                            }
                        }
                    }
                } else {
                    rsx! {}
                }
            } else {
                rsx! {}
            }}
        }
    }
}

#[cfg(feature = "brick-portfolio")]
#[component]
fn HomeFeaturedProjects() -> Element {
    let projects = use_server_future(loaders::load_projects)?;
    rsx! {
        SuspenseBoundary {
            fallback: |_| rsx! {},
            {if let Some(Ok(projects)) = projects() {
                let featured: Vec<_> = projects.iter().filter(|project| project.featured).take(2).cloned().collect();
                if !featured.is_empty() {
                    rsx! {
                        section { class: "mt-12 md:mt-16",
                            SectionHeader {
                                label: "Featured projects".to_string(),
                                href: "/projects".to_string(),
                                action: "All projects".to_string(),
                            }
                            div { class: "mt-3 grid gap-5 sm:grid-cols-2",
                                for project in featured {
                                    ProjectCard { project }
                                }
                            }
                        }
                    }
                } else {
                    rsx! {}
                }
            } else {
                rsx! {}
            }}
        }
    }
}

#[cfg(feature = "brick-activity")]
#[component]
fn HomeRecentActivity() -> Element {
    let activity = use_server_future(loaders::load_activity)?;
    rsx! {
        SuspenseBoundary {
            fallback: |_| rsx! {},
            {if let Some(Ok(activity)) = activity() {
                if !activity.is_empty() {
                    rsx! {
                        section { class: "mt-12 md:mt-16",
                            SectionHeader {
                                label: "Recent activity".to_string(),
                                href: "/activity".to_string(),
                                action: "All activity".to_string(),
                            }
                            div { class: "mt-2 md:mt-3",
                                for item in activity.iter().take(3) {
                                    ActivityRow { item: item.clone() }
                                }
                            }
                        }
                    }
                } else {
                    rsx! {}
                }
            } else {
                rsx! {}
            }}
        }
    }
}

#[component]
fn ContentPage(title: String, body: String) -> Element {
    rsx! {
        Shell { title: title.clone(),
            article { class: "max-w-2xl",
                h1 { class: "text-3xl font-bold tracking-tight {HEADING}", "{title}" }
                p { class: "mt-4 {BODY}", "{body}" }
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
            let html_content = article_html(&content.html_content, &content_title);
            return rsx! {
                Shell { title: content_title.clone(),
                    article { class: "max-w-2xl",
                        h1 { class: "text-3xl font-bold tracking-tight {HEADING}", "{content_title}" }
                        div { class: "prose prose-lg mt-6 max-w-none dark:prose-invert", dangerous_inner_html: html_content }
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
            div { class: "max-w-2xl",
                h1 { class: "text-4xl font-bold tracking-tight {HEADING}", "Page not found" }
                p { class: "mt-4 {BODY}",
                    "No route matches "
                    code { class: "rounded bg-gray-100 px-1.5 py-0.5 text-sm text-gray-800 dark:bg-neutral-900 dark:text-amber-200", "{path}" }
                }
                a {
                    href: "/",
                    class: "mt-8 inline-flex items-center rounded-md border border-gray-300 px-4 py-2 text-sm font-medium {LEAD} hover:bg-gray-50 motion-safe:transition-colors dark:border-amber-900 dark:hover:bg-white/5 {FOCUS_RING}",
                    "Return home"
                }
            }
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
    #[cfg(any(feature = "web", feature = "server"))]
    let title = use_context::<dioxus::fullstack::Loader<plinth_shared::SiteConfig>>()()
        .pages
        .about
        .title;
    #[cfg(not(any(feature = "web", feature = "server")))]
    let title = "About".to_string();
    rsx! { SiteContentPage {
        content_key: "about".to_string(),
        title,
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
        let site = use_context::<dioxus::fullstack::Loader<plinth_shared::SiteConfig>>()();
        let posts = use_loader(loaders::load_posts)?;
        let posts_value = posts().clone();
        return rsx! {
            Shell { title: site.pages.blog.title.clone(),
                PageHeader {
                    title: site.pages.blog.title.clone(),
                    subtitle: site.pages.blog.subtitle.clone(),
                }
                if posts_value.is_empty() {
                    tartan_ui_dioxus::EmptyState {
                        heading: "No published posts yet".to_string(),
                        message: "Check back soon for new writing and notes.".to_string(),
                    }
                } else {
                    div { class: "border-b {RULE}",
                        for post in posts_value.iter() {
                            PostRow { post: post.clone(), large: true, tags: true }
                        }
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
        let datetime = post.published_at.to_rfc3339();
        let date = post.published_at.format("%b %-d, %Y").to_string();
        return rsx! {
            Shell { title: post.title.clone(),
                article { class: "max-w-2xl",
                    p { class: "mb-4 text-sm {META}",
                        time { datetime: "{datetime}", "{date}" }
                        "\u{a0}· {post.reading_time_minutes} min read · {post.author}"
                    }
                    h1 { class: "text-balance text-3xl font-bold tracking-tight {HEADING} md:text-4xl", "{post.title}" }
                    if !post.description.is_empty() {
                        p { class: "mt-4 text-lg leading-relaxed {LEAD}", "{post.description}" }
                    }
                    div { class: "prose prose-lg mt-8 max-w-none dark:prose-invert", dangerous_inner_html: article_html(&post.html_content, &post.title) }
                    if !post.tags.is_empty() {
                        footer { class: "mt-8 flex flex-wrap gap-2",
                            for tag in post.tags.iter() {
                                TagChip { tag: tag.clone() }
                            }
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
                PageHeader { title: format!("Posts tagged #{tag}") }
                div { class: "border-b {RULE}",
                    for post in posts_value.iter() {
                        PostRow { post: post.clone(), large: true, tags: true }
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
                PageHeader { title: "Series".to_string() }
                div { class: "border-b {RULE}",
                    for item in series_value.iter() {
                    Link {
                        to: Route::SeriesDetail { slug: item.slug.clone() },
                        class: "flex flex-col gap-1 border-t py-4 sm:flex-row sm:items-baseline sm:justify-between sm:gap-6 {RULE} {FOCUS_RING}",
                        span { class: "text-lg font-semibold text-gray-900 hover:text-blue-600 motion-safe:transition-colors dark:text-amber-100 dark:hover:text-amber-300", "{item.title}" }
                        span { class: "shrink-0 text-sm {META}", "{item.post_count} posts · {item.total_reading_time} min" }
                    }
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
                PageHeader { title: slug.clone() }
                div { class: "border-b {RULE}",
                    for post in posts_value.iter() {
                        PostRow { post: post.clone() }
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
        let site = use_context::<dioxus::fullstack::Loader<plinth_shared::SiteConfig>>()();
        let projects = use_loader(loaders::load_projects)?;
        let projects_value = projects().clone();
        return rsx! {
            Shell { title: site.pages.portfolio.title.clone(),
                PageHeader {
                    title: site.pages.portfolio.title.clone(),
                    subtitle: site.pages.portfolio.subtitle.clone(),
                }
                div { class: "grid gap-5 sm:grid-cols-2",
                    for project in projects_value.iter() {
                        ProjectCard { project: project.clone(), image: true }
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
        let datetime = project.date.to_rfc3339();
        let date = project.date.format("%b %-d, %Y").to_string();
        return rsx! {
            Shell { title: project.title.clone(),
                article { class: "max-w-2xl",
                    p { class: "mb-4 text-sm {META}",
                        time { datetime: "{datetime}", "{date}" }
                    }
                    h1 { class: "text-balance text-3xl font-bold tracking-tight {HEADING} md:text-4xl", "{project.title}" }
                    p { class: "mt-4 text-lg leading-relaxed {LEAD}", "{project.description}" }
                    if let Some(html) = project.html_content.clone() {
                        div { class: "prose prose-lg mt-8 max-w-none dark:prose-invert", dangerous_inner_html: article_html(&html, &project.title) }
                    }
                    if !project.tech_stack.is_empty() {
                        div { class: "mt-6 flex flex-wrap gap-2",
                            for tech in project.tech_stack.iter() {
                                span { class: "inline-flex items-center rounded-full border border-gray-300 px-2.5 py-1 text-xs font-medium text-gray-600 dark:border-amber-900 dark:text-amber-400", "{tech}" }
                            }
                        }
                    }
                    div { class: "mt-8 flex flex-wrap gap-4",
                        if let Some(link) = project.link.clone() {
                            a { href: "{link}", target: "_blank", rel: "noreferrer", class: "text-sm font-medium {LINK} {FOCUS_RING}", "Source" }
                        }
                        if let Some(demo) = project.demo.clone() {
                            a { href: "{demo}", target: "_blank", rel: "noreferrer", class: "text-sm font-medium {LINK} {FOCUS_RING}", "Demo" }
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
                PageHeader { title: "Activity".to_string() }
                div { class: "border-b {RULE}",
                    for item in activity_value.iter() {
                        ActivityRow { item: item.clone() }
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
        let datetime = item.created_at.to_rfc3339();
        let date = item.created_at.format("%b %-d, %Y").to_string();
        return rsx! {
            Shell { title: item.title.clone(),
                article { class: "max-w-2xl",
                    p { class: "mb-4 text-sm {META}",
                        time { datetime: "{datetime}", "{date}" }
                        "\u{a0}· {item.repo_owner}/{item.repo_name} · {item.kind:?} · {item.state:?}"
                    }
                    h1 { class: "text-balance text-3xl font-bold tracking-tight {HEADING} md:text-4xl", "{item.title}" }
                    if let Some(body) = item.body.clone() {
                        // Forge bodies are untrusted markdown source: render as text.
                        p { class: "mt-6 whitespace-pre-line break-words {BODY}", "{body}" }
                    }
                    a {
                        href: "{item.url}",
                        target: "_blank",
                        rel: "noreferrer",
                        class: "mt-8 inline-flex text-sm font-medium {LINK} {FOCUS_RING}",
                        "Open on forge"
                    }
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
        let site = use_context::<dioxus::fullstack::Loader<plinth_shared::SiteConfig>>()();
        let todos = use_loader(loaders::load_todos)?;
        let todos_value = todos().clone();
        return rsx! {
            Shell { title: site.pages.todos.title.clone(),
                PageHeader {
                    title: site.pages.todos.title.clone(),
                    subtitle: site.pages.todos.subtitle.clone(),
                }
                div { class: "border-b {RULE}",
                for todo in todos_value.iter() {
                    article { class: "flex flex-col gap-1 border-t pt-5 pb-4 {RULE}",
                        Link {
                            to: Route::Todo { slug: todo.slug.clone() },
                            class: "text-lg font-semibold text-gray-900 hover:text-blue-600 motion-safe:transition-colors dark:text-amber-100 dark:hover:text-amber-300 {FOCUS_RING}",
                            "{todo.title}"
                        }
                        if !todo.description.is_empty() {
                            p { class: "text-[15px] leading-relaxed {BODY}", "{todo.description}" }
                        }
                        div { class: "mt-1 flex flex-wrap items-center gap-2",
                            if todo.completed {
                                span { class: "text-sm font-medium text-green-700 dark:text-green-400", "Completed" }
                            }
                            for tag in todo.tags.iter() {
                                Link {
                                    to: Route::TodoTag { tag: tag.clone() },
                                    class: "{CHIP} {FOCUS_RING}",
                                    "#{tag}"
                                }
                            }
                        }
                    }
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
                PageHeader { title: format!("Todos tagged #{tag}") }
                div { class: "border-b {RULE}",
                for todo in todos_value.iter() {
                    article { class: "border-t pt-5 pb-4 {RULE}",
                        Link {
                            to: Route::Todo { slug: todo.slug.clone() },
                            class: "text-lg font-semibold text-gray-900 hover:text-blue-600 motion-safe:transition-colors dark:text-amber-100 dark:hover:text-amber-300 {FOCUS_RING}",
                            "{todo.title}"
                        }
                        if !todo.description.is_empty() {
                            p { class: "mt-1 text-[15px] leading-relaxed {BODY}", "{todo.description}" }
                        }
                    }
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
                article { class: "max-w-2xl",
                    if todo.completed {
                        p { class: "mb-4 text-sm font-medium text-green-700 dark:text-green-400", "Completed" }
                    }
                    h1 { class: "text-balance text-3xl font-bold tracking-tight {HEADING} md:text-4xl", "{todo.title}" }
                    p { class: "mt-4 text-lg leading-relaxed {LEAD}", "{todo.description}" }
                    if let Some(html) = todo.html_content.clone() {
                        div { class: "prose prose-lg mt-8 max-w-none dark:prose-invert", dangerous_inner_html: article_html(&html, &todo.title) }
                    }
                    if !todo.tags.is_empty() {
                        footer { class: "mt-8 flex flex-wrap gap-2",
                            for tag in todo.tags.iter() {
                                Link {
                                    to: Route::TodoTag { tag: tag.clone() },
                                    class: "{CHIP} {FOCUS_RING}",
                                    "#{tag}"
                                }
                            }
                        }
                    }
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

#[cfg(test)]
mod tests {
    use super::article_html;

    #[test]
    fn drops_only_the_duplicate_leading_title_heading() {
        assert_eq!(
            article_html("<h1>Title</h1><p>Body</p>", "Title"),
            "<p>Body</p>"
        );
        assert_eq!(
            article_html(
                "<!DOCTYPE html>\n<html><head><title>x</title></head><body>\n<h2>About</h2>\n<p>Body</p></body></html>",
                "About"
            ),
            "<p>Body</p>"
        );
        assert_eq!(
            article_html("<p>Body</p><h1>Later</h1>", "Title"),
            "<p>Body</p><h1>Later</h1>"
        );
        assert_eq!(
            article_html("<h1>Another title</h1><p>Body</p>", "Title"),
            "<h1>Another title</h1><p>Body</p>"
        );
    }
}
