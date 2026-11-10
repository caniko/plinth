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
const NAV_LINK_CURRENT: &str = "rounded-md bg-gray-200 px-3 py-2 text-[15px] font-medium text-gray-900 dark:bg-amber-900/60 dark:text-amber-100";

#[cfg_attr(
    not(any(
        feature = "brick-blog",
        feature = "brick-portfolio",
        feature = "brick-activity"
    )),
    allow(dead_code)
)]
const LINK: &str = "text-blue-600 hover:text-blue-700 motion-safe:transition-colors dark:text-amber-300 dark:hover:text-amber-200";
/// Non-interactive pill (tech stack).
#[cfg(feature = "brick-portfolio")]
const PILL: &str = "inline-flex items-center rounded-full border border-gray-300 px-2.5 py-1 text-xs font-medium text-gray-600 dark:border-amber-900 dark:text-amber-400";
/// Linked pill (tags); taller on touch screens.
#[cfg(any(feature = "brick-blog", feature = "brick-todo"))]
const CHIP: &str = "min-h-8 sm:min-h-0 inline-flex items-center rounded-full border border-gray-300 px-2.5 py-1 text-xs font-medium text-gray-600 hover:border-gray-400 hover:text-gray-800 motion-safe:transition-colors dark:border-amber-900 dark:text-amber-400 dark:hover:border-amber-700 dark:hover:text-amber-300";
/// Footer links get a touch-sized box on small screens.
const FOOTER_LINK: &str = "inline-flex min-h-11 items-center text-blue-600 hover:text-blue-700 motion-safe:transition-colors dark:text-amber-300 dark:hover:text-amber-200 md:min-h-0";
const RULE: &str = "border-gray-200 dark:border-amber-900/50";
const HEADING: &str = "text-gray-900 dark:text-amber-100";
const BODY: &str = "text-gray-700 dark:text-amber-200";
/// Taglines and ledes. Amber-300 is reserved for links.
const LEAD: &str = "text-gray-700 dark:text-amber-200";
const META: &str = "text-gray-600 dark:text-amber-500";
/// Every page title except the home hero.
const H1: &str =
    "text-balance text-3xl font-bold tracking-tight text-gray-900 dark:text-amber-100 md:text-4xl";
const INLINE_CODE: &str = "rounded bg-gray-100 px-1.5 py-0.5 text-sm text-gray-800 dark:bg-neutral-900 dark:text-amber-200";

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
    let default_theme = site_config().default_theme.clone();
    #[cfg(any(feature = "web", feature = "server"))]
    let favicon = site_config().favicon.clone();
    #[cfg(any(feature = "web", feature = "server"))]
    let favicon_link = favicon.map(|favicon| rsx! { link { rel: "icon", href: "{favicon}" } });
    #[cfg(not(any(feature = "web", feature = "server")))]
    let default_theme = "dark".to_string();
    #[cfg(not(any(feature = "web", feature = "server")))]
    let favicon_link = rsx! {};

    #[cfg(feature = "brick-blog")]
    let feed_link = rsx! {
        document::Link {
            rel: "alternate",
            r#type: "application/rss+xml",
            title: "Posts",
            href: "/feed.xml",
        }
    };
    #[cfg(not(feature = "brick-blog"))]
    let feed_link = rsx! {};

    rsx! {
        document::Stylesheet { href: tartan_ui_dioxus::SHARED_STYLES }
        document::Script { {bootstrap_script(&default_theme)} }
        {favicon_link}
        {feed_link}
        Router::<Route> {}
    }
}

/// Inline page bootstrap, independent of WebAssembly:
/// - resolves the theme before first paint (stored choice, else the default);
/// - makes the theme toggle work before hydration;
/// - restores the scroll position on Back/Forward. The router stores `[x, y]`
///   in the history state but scrolls before the route has rendered, so retry
///   for up to a second until the page is tall enough.
///
/// It avoids `&&`, `<` and `>` so the SSR text pipeline cannot escape it.
fn bootstrap_script(default_theme: &str) -> String {
    const SCRIPT: &str = r#"(function(){var r=document.documentElement,t=null;try{t=localStorage.getItem("theme")}catch(e){}if(t!=="dark"){if(t!=="light"){t="__DEFAULT__"}}r.classList.toggle("dark",t==="dark");document.addEventListener("click",function(e){if(e.target.closest("[data-theme-toggle]")){var d=r.classList.toggle("dark");try{localStorage.setItem("theme",d?"dark":"light")}catch(x){}}});addEventListener("popstate",function(e){var s=e.state,n=60;if(Array.isArray(s)){(function f(){scrollTo(s[0],s[1]);n-=1;if(n){if(Math.floor(s[1]-scrollY)){requestAnimationFrame(f)}}})()}})})();"#;
    SCRIPT.replace(
        "__DEFAULT__",
        if default_theme == "light" {
            "light"
        } else {
            "dark"
        },
    )
}

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

/// Dioxus' web `Title` evaluates JavaScript, which Plinth's CSP forbids, so
/// the client sets the title through the DOM. Link-preview metadata is
/// server-rendered only. After a client-side navigation focus moves to the new
/// page.
#[component]
fn PageHead(title: String, description: String) -> Element {
    #[cfg(feature = "server")]
    {
        return rsx! {
            document::Title { "{title}" }
            document::Meta { name: "description", content: "{description}" }
            document::Meta { property: "og:title", content: "{title}" }
            document::Meta { property: "og:description", content: "{description}" }
        };
    }
    #[cfg(feature = "web")]
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        use web_sys::wasm_bindgen::JsCast;
        static NAVIGATED: AtomicBool = AtomicBool::new(false);

        use_effect(use_reactive!(|title| {
            let Some(document) = web_sys::window().and_then(|window| window.document()) else {
                return;
            };
            document.set_title(&title);
            // The first page keeps the browser's own focus start.
            if NAVIGATED.swap(true, Ordering::Relaxed)
                && let Some(main) = document
                    .get_element_by_id("content")
                    .and_then(|main| main.dyn_into::<web_sys::HtmlElement>().ok())
            {
                let options = web_sys::FocusOptions::new();
                options.set_prevent_scroll(true);
                let _ = main.focus_with_options(&options);
            }
        }));
    }
    let _ = description;
    // Hydration walks one placeholder per head component the server rendered.
    rsx! {
        HeadSlot {}
        HeadSlot {}
        HeadSlot {}
        HeadSlot {}
    }
}

#[component]
fn HeadSlot() -> Element {
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
fn Shell(title: String, #[props(default)] description: String, children: Element) -> Element {
    #[cfg(any(feature = "web", feature = "server"))]
    let site_config = use_context::<dioxus::fullstack::Loader<plinth_shared::SiteConfig>>();
    #[cfg(any(feature = "web", feature = "server"))]
    let site_name = site_config().name.clone();
    #[cfg(any(feature = "web", feature = "server"))]
    let description = if description.is_empty() {
        site_config().description.clone()
    } else {
        description
    };
    #[cfg(not(any(feature = "web", feature = "server")))]
    let site_name = "Plinth".to_string();

    rsx! {
        div { class: "flex min-h-screen flex-col bg-white text-gray-700 dark:bg-black dark:text-amber-200",
            PageHead {
                title: if title.is_empty() { site_name } else { format!("{title} - {site_name}") },
                description,
            }
            a {
                href: "#content",
                class: "sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-4 focus:z-50 focus:rounded-md focus:bg-white focus:px-4 focus:py-2 focus:text-sm focus:font-medium focus:text-gray-900 focus:shadow-lg dark:focus:bg-neutral-950 dark:focus:text-amber-100 {FOCUS_RING}",
                "Skip to content"
            }
            Header {}
            main {
                id: "content",
                tabindex: "-1",
                class: "mx-auto w-full max-w-4xl flex-1 px-4 pb-24 pt-10 outline-none md:px-6 md:pt-16",
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

    rsx! {
        header { class: "border-b {RULE}",
            div { class: "group mx-auto w-full max-w-4xl px-4 md:px-6",
                div { class: "flex h-14 items-center justify-between gap-3 md:h-16",
                    Link {
                        to: Route::Home {},
                        class: "flex min-w-0 items-center gap-2.5 {FOCUS_RING}",
                        if let Some(logo) = site.logo.clone() {
                            img { src: "{logo}", alt: "", class: "h-[26px] w-auto shrink-0" }
                        } else {
                            // The plinth mark, in the accent colour of each theme.
                            svg {
                                class: "h-[26px] w-[26px] shrink-0 text-blue-600 dark:text-amber-400",
                                view_box: "0 0 36 36",
                                fill: "currentColor",
                                "aria-hidden": "true",
                                rect { x: "11", y: "4", width: "14", height: "21", rx: "1" }
                                rect { y: "28", width: "36", height: "8", rx: "1" }
                            }
                        }
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
            Link {
                to: item.path.clone(),
                class: "motion-safe:transition-colors {NAV_LINK_CURRENT} {FOCUS_RING}",
                aria_current: "page",
                "{item.label}"
            }
        }
    } else {
        rsx! {
            Link {
                to: item.path.clone(),
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
            Link {
                to: item.path.clone(),
                class: "block border-b border-b-gray-200 border-l-[3px] border-l-amber-500 bg-gray-100 py-3.5 pl-3 pr-3 text-[15px] font-medium text-gray-900 motion-safe:transition-colors dark:border-b-amber-900/50 dark:border-l-amber-500 dark:bg-neutral-950 dark:text-amber-100 {FOCUS_RING}",
                aria_current: "page",
                onclick: |_| close_mobile_menu(),
                "{item.label}"
            }
        }
    } else {
        rsx! {
            Link {
                to: item.path.clone(),
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
            "data-theme-toggle": "true",
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
    let rss = rsx! { a { href: "/feed.xml", class: "{FOOTER_LINK} {FOCUS_RING}", "RSS" } };
    #[cfg(not(feature = "brick-blog"))]
    let rss = rsx! {};

    rsx! {
        footer { class: "border-t {RULE}",
            div { class: "mx-auto flex w-full max-w-4xl flex-wrap items-center gap-x-3 px-4 py-6 text-sm md:px-6 md:py-8",
                nav { class: "order-1 flex w-full flex-wrap items-center gap-x-5 md:order-2 md:ml-5 md:w-auto", aria_label: "Elsewhere",
                    for (label, url) in social.iter() {
                        if !url.is_empty() {
                            a {
                                href: "{url}",
                                rel: "me noopener",
                                class: "{FOOTER_LINK} {FOCUS_RING}",
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
                        class: "order-4 md:order-3 md:ml-auto {FOOTER_LINK} {FOCUS_RING}",
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
            h1 { class: "{H1}", "{title}" }
            if !subtitle.is_empty() {
                p { class: "mt-2 text-lg {LEAD}", "{subtitle}" }
            }
        }
    }
}

#[component]
fn SectionHeader(label: String, href: String, action: String) -> Element {
    rsx! {
        div { class: "flex items-baseline justify-between gap-4",
            h2 { class: "text-xs font-semibold uppercase leading-4 tracking-[0.16em] {META}", "{label}" }
            Link {
                to: href,
                class: "inline-flex min-h-11 shrink-0 items-center text-sm font-medium md:min-h-0 {LINK} {FOCUS_RING}",
                "{action} \u{2192}"
            }
        }
    }
}

/// The way back to a list from a detail or filtered page.
#[cfg(any(feature = "brick-blog", feature = "brick-portfolio"))]
#[component]
fn BackLink(to: Route, label: String) -> Element {
    rsx! {
        div {
            Link {
                to,
                class: "mt-8 inline-flex min-h-11 items-center text-sm font-medium {LINK} {FOCUS_RING}",
                "\u{2190} {label}"
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
    let heading_class = "text-balance font-semibold tracking-tight sm:col-start-1 sm:row-start-1";
    let title = rsx! {
        Link {
            to: Route::Post { slug: post.slug.clone() },
            class: "text-gray-900 hover:text-blue-600 motion-safe:transition-colors dark:text-amber-100 dark:hover:text-amber-300 {FOCUS_RING}",
            "{post.title}"
        }
    };

    rsx! {
        article { class: "flex flex-col gap-1 border-t pt-4 pb-4 {RULE} sm:pt-5 sm:pb-5 sm:grid sm:grid-cols-[minmax(0,1fr)_auto] sm:items-baseline sm:gap-x-6 sm:gap-y-1",
            // List pages sit under the page h1; the home sections under an h2.
            if large {
                h2 { class: "text-xl {heading_class}", {title} }
            } else {
                h3 { class: "text-lg {heading_class}", {title} }
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
fn ProjectCard(project: plinth_shared::PortfolioItem, #[props(default)] full: bool) -> Element {
    rsx! {
        Link {
            to: Route::Project { slug: project.slug.clone() },
            class: "flex flex-col rounded-lg border border-gray-200 bg-white p-5 hover:border-gray-400 motion-safe:transition-colors dark:border-amber-900/50 dark:bg-neutral-950 dark:hover:border-amber-700 {FOCUS_RING}",
            if full && let Some(image_url) = project.image_url.clone() {
                img {
                    src: "{image_url}",
                    alt: "",
                    class: "mb-4 h-40 w-full rounded-md object-cover",
                }
            }
            // The list page has no section heading between its h1 and the cards.
            if full {
                h2 { class: "text-base font-semibold {HEADING}", "{project.title}" }
            } else {
                h3 { class: "text-base font-semibold {HEADING}", "{project.title}" }
            }
            p { class: "mt-2 text-[15px] leading-relaxed {BODY}", "{project.description}" }
            if !project.tech_stack.is_empty() {
                div { class: "mt-4 flex flex-wrap gap-2",
                    for tech in project.tech_stack.iter() {
                        span { class: "{PILL}", "{tech}" }
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
        Shell { title: String::new(),
            section {
                h1 { class: "text-balance text-[34px] font-bold leading-tight tracking-tight {HEADING} md:mt-1 md:text-5xl", "{site.name}" }
                p { class: "mt-3 text-pretty text-xl leading-snug {LEAD}", "{site.tagline}" }
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
                    rsx! { p { class: "mt-4 max-w-[40rem] text-pretty text-base leading-snug {BODY}", "{description}" } }
                }}
            }
        };
    }
    rsx! { p { class: "mt-4 max-w-[40rem] text-pretty text-base leading-snug {BODY}", "{description}" } }
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

/// A ruled list, or the shared empty state when it has no rows.
#[component]
fn RuledList(empty: bool, nothing: String, children: Element) -> Element {
    if empty {
        rsx! {
            tartan_ui_dioxus::EmptyState {
                heading: nothing,
                message: "Check back soon.".to_string(),
            }
        }
    } else {
        rsx! { div { class: "border-b {RULE}", {children} } }
    }
}

#[component]
fn ContentPage(title: String, body: String) -> Element {
    rsx! {
        Shell { title: title.clone(),
            article { class: "max-w-2xl",
                h1 { class: "{H1}", "{title}" }
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
                        h1 { class: "{H1}", "{content_title}" }
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
                h1 { class: "{H1}", "Page not found" }
                p { class: "mt-4 {BODY}",
                    "There is no page at "
                    code { class: "{INLINE_CODE}", "{path}" }
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
            Shell { title: post.title.clone(), description: post.description.clone(),
                article { class: "max-w-2xl",
                    p { class: "mb-4 text-sm {META}",
                        time { datetime: "{datetime}", "{date}" }
                        "\u{a0}· {post.reading_time_minutes} min read · {post.author}"
                    }
                    h1 { class: "{H1}", "{post.title}" }
                    if !post.description.is_empty() {
                        p { class: "mt-4 text-pretty text-xl leading-relaxed {LEAD}", "{post.description}" }
                    }
                    div { class: "prose prose-lg mt-8 max-w-none dark:prose-invert", dangerous_inner_html: article_html(&post.html_content, &post.title) }
                    if !post.tags.is_empty() {
                        footer { class: "mt-8 flex flex-wrap gap-2",
                            for tag in post.tags.iter() {
                                TagChip { tag: tag.clone() }
                            }
                        }
                    }
                    BackLink { to: Route::Posts {}, label: "All posts".to_string() }
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
        if posts_value.is_empty() {
            return rsx! { MissingPage { path: format!("/posts/tag/{tag}") } };
        }
        return rsx! {
            Shell { title: format!("Posts tagged {tag}"),
                PageHeader { title: format!("Posts tagged #{tag}") }
                div { class: "border-b {RULE}",
                    for post in posts_value.iter() {
                        PostRow { post: post.clone(), large: true, tags: true }
                    }
                }
                BackLink { to: Route::Posts {}, label: "All posts".to_string() }
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
                RuledList { empty: series_value.is_empty(), nothing: "No series yet".to_string(),
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
        if posts_value.is_empty() {
            return rsx! { MissingPage { path: format!("/series/{slug}") } };
        }
        return rsx! {
            Shell { title: slug.clone(),
                PageHeader { title: slug.clone() }
                div { class: "border-b {RULE}",
                    for post in posts_value.iter() {
                        PostRow { post: post.clone(), large: true }
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
                if projects_value.is_empty() {
                    tartan_ui_dioxus::EmptyState {
                        heading: "No projects yet".to_string(),
                        message: "Check back soon.".to_string(),
                    }
                } else {
                    div { class: "grid gap-5 sm:grid-cols-2",
                        for project in projects_value.iter() {
                            ProjectCard { project: project.clone(), full: true }
                        }
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
            Shell { title: project.title.clone(), description: project.description.clone(),
                article { class: "max-w-2xl",
                    p { class: "mb-4 text-sm {META}",
                        time { datetime: "{datetime}", "{date}" }
                    }
                    h1 { class: "{H1}", "{project.title}" }
                    p { class: "mt-4 text-pretty text-xl leading-relaxed {LEAD}", "{project.description}" }
                    if let Some(html) = project.html_content.clone() {
                        div { class: "prose prose-lg mt-8 max-w-none dark:prose-invert", dangerous_inner_html: article_html(&html, &project.title) }
                    }
                    if !project.tech_stack.is_empty() {
                        div { class: "mt-6 flex flex-wrap gap-2",
                            for tech in project.tech_stack.iter() {
                                span { class: "{PILL}", "{tech}" }
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
                    BackLink { to: Route::Projects {}, label: "All projects".to_string() }
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
                RuledList { empty: activity_value.is_empty(), nothing: "No activity yet".to_string(),
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
                    h1 { class: "{H1}", "{item.title}" }
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
                RuledList { empty: todos_value.is_empty(), nothing: "Nothing on the list yet".to_string(),
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
        if todos_value.is_empty() {
            return rsx! { MissingPage { path: format!("/todos/tag/{tag}") } };
        }
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
                    h1 { class: "{H1}", "{todo.title}" }
                    p { class: "mt-4 text-pretty text-xl leading-relaxed {LEAD}", "{todo.description}" }
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
