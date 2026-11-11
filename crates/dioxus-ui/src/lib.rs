#![allow(non_snake_case)]
// Brick-disabled fallbacks intentionally share component bodies with the
// feature-enabled render branches.  When all bricks are enabled, those
// fallback expressions are statically unreachable; keep that configuration
// warning-free without duplicating every component for every feature matrix.
#![allow(unreachable_code)]

use dioxus::prelude::*;
use dioxus_router::{Link, Outlet, Routable, Router};

#[cfg(any(feature = "web", feature = "server"))]
mod loaders;
#[cfg(any(feature = "web", feature = "server"))]
pub mod page_cache;

/// Shared focus treatment: a visible 2px ring offset from the control, amber
/// on the dark canvas and blue on the light one.
const FOCUS_RING: &str = "focus-visible:transition-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-600 dark:focus-visible:outline-amber-400";

const NAV_LINK: &str = "rounded-md px-3 py-2 text-[0.9375rem] font-medium text-gray-700 hover:bg-gray-100 hover:text-gray-900 motion-safe:transition-colors dark:text-amber-200 dark:hover:bg-white/5 dark:hover:text-amber-100";
const NAV_LINK_CURRENT: &str = "rounded-md bg-gray-200 px-3 py-2 text-[0.9375rem] font-medium text-gray-900 dark:bg-amber-900/60 dark:text-amber-100";

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
/// Linked pill (tags): link colours, unlike the static tech pills; taller on
/// touch screens.
#[cfg(any(feature = "brick-blog", feature = "brick-todo"))]
const CHIP: &str = "aria-[current=page]:bg-gray-200 aria-[current=page]:text-gray-900 dark:aria-[current=page]:bg-amber-900/60 dark:aria-[current=page]:text-amber-100 min-h-8 sm:min-h-0 inline-flex items-center rounded-full border border-blue-200 px-2.5 py-1 text-xs font-medium text-blue-700 hover:border-blue-400 hover:text-blue-800 motion-safe:transition-colors dark:border-amber-800 dark:text-amber-300 dark:hover:border-amber-600 dark:hover:text-amber-200";
/// Footer links get a touch-sized box on small screens.
const FOOTER_LINK: &str = "inline-flex min-h-11 items-center text-blue-600 hover:text-blue-700 motion-safe:transition-colors dark:text-amber-300 dark:hover:text-amber-200 md:min-h-0";
const RULE: &str = "border-gray-200 dark:border-amber-900/80";
const HEADING: &str = "text-gray-900 dark:text-amber-100";
const BODY: &str = "text-gray-700 dark:text-amber-200";
/// Taglines and ledes. Amber-300 is reserved for links.
const LEAD: &str = "text-gray-700 dark:text-amber-200";
const META: &str = "text-gray-600 dark:text-amber-500";
/// Every page title except the home hero.
const H1: &str =
    "text-balance text-3xl font-bold tracking-tight text-gray-900 dark:text-amber-100 md:text-4xl";
const BUTTON: &str = "inline-flex min-h-11 items-center gap-1.5 rounded-md border border-gray-300 px-4 text-sm font-medium text-gray-900 hover:bg-gray-50 motion-safe:transition-colors dark:border-amber-800 dark:text-amber-100 dark:hover:bg-white/5";
const INLINE_CODE: &str = "rounded bg-gray-100 px-1.5 py-0.5 text-sm text-gray-800 dark:bg-neutral-900 dark:text-amber-200";

/// The complete public page route contract. Brick-specific variants are
/// feature-gated here so reduced builds do not expose dead routes, while the
/// enum remains a single source of truth for navigation and SSR.
#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[layout(RouteFrame)]
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
        // A failed loader would otherwise replace the site with a raw error.
        ErrorBoundary {
            handle_error: |_| rsx! { LoadError {} },
            Router::<Route> {}
        }
    }
}

/// Keeps the header and footer on screen while the next page's data loads
/// during client navigation, and shows a load failure inside the same shell;
/// the server waits for loaders, so it never renders the fallback.
#[component]
fn RouteFrame() -> Element {
    rsx! {
        ErrorBoundary {
            handle_error: |errors: ErrorContext| rsx! { RouteError { errors } },
            SuspenseBoundary {
                fallback: |_| rsx! { PendingPage {} },
                Outlet::<Route> {}
            }
        }
    }
}

/// A page whose data failed to load. Navigating to another route clears the
/// failure so that route renders and loads normally.
#[component]
fn RouteError(errors: ErrorContext) -> Element {
    let path = use_route::<Route>().to_string();
    let failed = use_hook(|| path.clone());
    use_effect(use_reactive!(|path| {
        if path != failed {
            errors.clear_errors();
        }
    }));
    rsx! {
        Shell { title: "Not loaded".to_string(), LoadErrorBody {} }
    }
}

/// While the next page loads, the outgoing page's content stays in place,
/// dimmed and inert, under a loading bar that appears after 300 ms. The
/// bootstrap copies it when the navigation starts (the old page has left the
/// document by the time this renders) and records the page height, so the
/// scroll position the router saves is not clamped. There is no `#content`,
/// so the bootstrap watchdog still reloads a load that never finishes.
#[component]
fn PendingPage() -> Element {
    #[cfg(feature = "web")]
    let previous = use_hook(|| {
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("route-previous"))
            .map(|snapshot| snapshot.inner_html())
            .unwrap_or_default()
    });
    #[cfg(not(feature = "web"))]
    let previous = String::new();
    rsx! {
        div {
            class: "flex min-h-screen flex-col bg-white text-gray-700 dark:bg-black dark:text-amber-200",
            style: "min-height: max(100vh, var(--route-min-h, 0px))",
            div { class: "route-pending fixed inset-x-0 top-0 z-50 h-0.5 bg-blue-600 dark:bg-amber-400" }
            p { class: "sr-only", role: "status", "Loading" }
            Header {}
            main {
                class: "mx-auto w-full max-w-4xl flex-1 px-4 pb-24 pt-10 opacity-75 md:px-6 md:pt-16 dark:opacity-60",
                aria_busy: "true",
                inert: true,
                dangerous_inner_html: previous,
            }
            Footer {}
        }
    }
}

/// Shown instead of the router when data above the routes (the site
/// configuration) cannot be loaded; page loaders fail inside `RouteFrame`.
#[component]
fn LoadError() -> Element {
    #[cfg(any(feature = "web", feature = "server"))]
    let site_name = use_context::<dioxus::fullstack::Loader<plinth_shared::SiteConfig>>()()
        .name
        .clone();
    #[cfg(not(any(feature = "web", feature = "server")))]
    let site_name = "Plinth".to_string();
    rsx! {
        div {
            class: "min-h-screen bg-white text-gray-700 dark:bg-black dark:text-amber-200",
            PageHead { title: format!("Not loaded - {site_name}"), description: String::new() }
            header { class: "border-b {RULE}",
                div { class: "mx-auto flex h-14 w-full max-w-4xl items-center px-4 md:h-16 md:px-6",
                    a {
                        href: "/",
                        class: "text-[0.9375rem] font-semibold text-gray-900 dark:text-amber-100 md:text-base {FOCUS_RING}",
                        "{site_name}"
                    }
                }
            }
            main {
                id: "content",
                tabindex: "-1",
                class: "mx-auto w-full max-w-4xl px-4 pb-24 pt-10 outline-none md:px-6 md:pt-16",
                LoadErrorBody {}
            }
        }
    }
}

/// The failure message and its two ways out. The links are plain anchors: a
/// full load is the way out.
#[component]
fn LoadErrorBody() -> Element {
    #[cfg(feature = "server")]
    dioxus::fullstack::FullstackContext::commit_http_status(
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        Some("page data unavailable".to_string()),
    );
    rsx! {
        div { "data-load-error": "true",
            div { role: "alert",
                h1 { class: "{H1}", "This page could not be loaded" }
                p { class: "mt-4 {BODY}", "Something went wrong while fetching it. Try again in a moment." }
            }
            div { class: "mt-8 flex flex-wrap gap-3",
                a { href: "", class: "{BUTTON} {FOCUS_RING}", "Try again" }
                a { href: "/", class: "{BUTTON} {FOCUS_RING}", "Home" }
            }
        }
    }
}

/// Inline page bootstrap, independent of WebAssembly:
/// - resolves the theme before first paint (stored choice, else the default)
///   and keeps the `theme-color` meta in step with it;
/// - makes the theme toggle work before hydration and mirrors the menu's
///   `open` state into `aria-expanded`;
/// - on link clicks and Back/Forward, records the page height
///   (`--route-min-h`) and copies the page content into
///   `template#route-previous` for the route fallback, and closes the menu on
///   a tap outside it;
/// - keeps the scroll position in the history entry (`[x, y]`, the router's
///   own state shape) and restores it on Back, Forward and reload. The router
///   scrolls once, before the route has rendered, so re-apply until the
///   position has held for a few frames or the reader takes over;
/// - reloads on Back/Forward while the load-error page is shown, because the
///   router is not mounted then;
/// - closes the mobile menu on Escape;
/// - reloads when the page has had no content for eight seconds: a pending
///   loader suspends the whole tree on the client, and a stalled request would
///   otherwise leave a blank document. A boundary below the router would fix
///   that but makes the server stream fallbacks (no content without
///   JavaScript, no 404 status).
///
/// It avoids `&&`, `<` and `>` so the SSR text pipeline cannot escape it.
fn bootstrap_script(default_theme: &str) -> String {
    const SCRIPT: &str = r##"(function(){var r=document.documentElement,t=null,k=0;try{t=localStorage.getItem("theme")}catch(e){}if(t!=="dark"){if(t!=="light"){t="__DEFAULT__"}}document.addEventListener("toggle",function(e){var s=e.target.querySelector("summary");if(s){s.setAttribute("aria-expanded",e.target.open?"true":"false")}},true);function H(){r.style.setProperty("--route-min-h",document.documentElement.scrollHeight+"px");var c=document.getElementById("content");if(c){var p=document.getElementById("route-previous");if(!p){p=document.createElement("template");p.id="route-previous";document.body.appendChild(p)}p.innerHTML=c.innerHTML}}document.addEventListener("click",function(e){if(e.target.closest("a[href]")){H()}var o=document.querySelector("header details[open]");if(o){if(!e.target.closest("header")){o.removeAttribute("open")}}},true);function C(){var m=document.querySelector("meta[name=theme-color]");if(m){m.content=r.classList.contains("dark")?"#000000":"#ffffff"}}r.classList.toggle("dark",t==="dark");C();document.addEventListener("click",function(e){if(e.target.closest("[data-theme-toggle]")){var d=r.classList.toggle("dark");C();try{localStorage.setItem("theme",d?"dark":"light")}catch(x){}}});function R(s){if(!Array.isArray(s))return;var n=180,h=0;["wheel","touchstart","keydown"].forEach(function(v){addEventListener(v,function(){n=0},{once:true,passive:true})});(function f(){if(n===0)return;n-=1;if(Math.floor(Math.abs(s[1]-scrollY))===0){h+=1;if(h===5)return}else{h=0;scrollTo(s[0],s[1])}requestAnimationFrame(f)})()}addEventListener("popstate",function(e){H();if(document.querySelector("[data-load-error]")){location.reload()}else{R(e.state)}});addEventListener("DOMContentLoaded",function(){R(history.state)});addEventListener("keydown",function(e){if(e.key==="Escape"){var d=document.querySelector("header details[open]");if(d){d.removeAttribute("open");d.querySelector("summary").focus()}}});var m=0;setInterval(function(){if(document.getElementById("content")){m=0}else{m+=1;if(m===8){location.reload()}}},1000);addEventListener("scroll",function(){clearTimeout(k);k=setTimeout(function(){try{history.replaceState([scrollX,scrollY],"")}catch(x){}},150)},{passive:true})})();"##;
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
    accessible_html(&strip_title_heading(html, title))
}

/// Scrollable code blocks and tables become focusable so the keyboard can
/// scroll them, tables get a scroll container instead of breaking words, and
/// task-list checkboxes get a name.
#[cfg(any(
    feature = "web",
    feature = "server",
    feature = "brick-blog",
    feature = "brick-portfolio",
    feature = "brick-todo",
    test
))]
fn accessible_html(html: &str) -> String {
    html.replace("<pre ", "<pre tabindex=\"0\" ")
        .replace("<pre>", "<pre tabindex=\"0\">")
        .replace(
            "<table",
            "<div class=\"table-scroll\" tabindex=\"0\"><table",
        )
        .replace("</table>", "</table></div>")
        .replace("type=\"checkbox\"", "type=\"checkbox\" aria-label=\"Task\"")
}

/// Drop a leading heading that repeats the page title.
#[cfg(any(
    feature = "web",
    feature = "server",
    feature = "brick-blog",
    feature = "brick-portfolio",
    feature = "brick-todo",
    test
))]
fn strip_title_heading(html: &str, title: &str) -> String {
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
/// the client keeps the title and link-preview tags current through the DOM
/// after the server rendered them. `path` adds the canonical URL when
/// `base_url` is set.
#[component]
fn PageHead(title: String, description: String, #[props(default)] path: Option<String>) -> Element {
    #[cfg(any(feature = "web", feature = "server"))]
    let base_url = use_context::<dioxus::fullstack::Loader<plinth_shared::SiteConfig>>()()
        .base_url
        .trim_end_matches('/')
        .to_string();
    #[cfg(not(any(feature = "web", feature = "server")))]
    let base_url = String::new();
    let url = path
        .filter(|_| !base_url.is_empty())
        .map(|path| format!("{base_url}{path}"));
    #[cfg(feature = "server")]
    {
        return rsx! {
            document::Title { "{title}" }
            document::Meta { name: "description", content: "{description}" }
            document::Meta { property: "og:title", content: "{title}" }
            document::Meta { property: "og:description", content: "{description}" }
            if let Some(url) = url {
                document::Link { rel: "canonical", href: "{url}" }
                document::Meta { property: "og:url", content: "{url}" }
            }
        };
    }
    #[cfg(feature = "web")]
    {
        let current = url.clone();
        use_effect(use_reactive!(|title, description, current| {
            let Some(document) = web_sys::window().and_then(|window| window.document()) else {
                return;
            };
            document.set_title(&title);
            let set = |selector: &str, attribute: &str, value: &str| {
                if let Ok(Some(element)) = document.query_selector(selector) {
                    let _ = element.set_attribute(attribute, value);
                }
            };
            set("meta[name=description]", "content", &description);
            set("meta[property='og:title']", "content", &title);
            set("meta[property='og:description']", "content", &description);
            if let Some(current) = &current {
                set("meta[property='og:url']", "content", current);
                set("link[rel=canonical]", "href", current);
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
        if url.is_some() {
            HeadSlot {}
            HeadSlot {}
        }
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
    let path = use_route::<Route>().to_string();
    #[cfg(feature = "web")]
    use_effect(use_reactive!(|path| {
        let _ = path;
        page_focus::route_changed();
    }));

    rsx! {
        div { class: "flex min-h-screen flex-col bg-white text-gray-700 dark:bg-black dark:text-amber-200",
            PageHead {
                title: if title.is_empty() { site_name } else { format!("{title} - {site_name}") },
                description,
                path,
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
                onmounted: |_| {
                    #[cfg(feature = "web")]
                    page_focus::main_mounted();
                },
                {children}
            }
            Footer {}
        }
    }
}

/// After a client-side navigation focus moves to the new page: at once when
/// its `main` is reused, or when a suspended page mounts its `main` later. The
/// first page keeps the browser's own focus start.
#[cfg(feature = "web")]
mod page_focus {
    use std::sync::atomic::{AtomicBool, Ordering};
    use web_sys::wasm_bindgen::JsCast;

    static NAVIGATED: AtomicBool = AtomicBool::new(false);
    static PENDING: AtomicBool = AtomicBool::new(false);

    fn focus_main() {
        if let Some(main) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("content"))
            .and_then(|main| main.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let options = web_sys::FocusOptions::new();
            options.set_prevent_scroll(true);
            let _ = main.focus_with_options(&options);
        }
    }

    pub fn route_changed() {
        if NAVIGATED.swap(true, Ordering::Relaxed) {
            PENDING.store(true, Ordering::Relaxed);
            focus_main();
        }
    }

    pub fn main_mounted() {
        if PENDING.swap(false, Ordering::Relaxed) {
            focus_main();
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
                        span { class: "truncate text-[0.9375rem] font-semibold text-gray-900 dark:text-amber-100 md:text-base", "{site.name}" }
                    }
                    div { class: "flex shrink-0 items-center gap-2",
                        nav { class: "hidden items-center gap-1 md:flex", aria_label: "Primary",
                            for item in site.nav.iter() {
                                NavLink {
                                    item: item.clone(),
                                    current: is_current_nav(&item.path, &current_path),
                                    exact: item.path == current_path,
                                }
                            }
                        }
                        ThemeToggle {}
                        details { class: "md:hidden",
                            summary {
                                class: "menu-summary flex h-11 w-11 cursor-pointer list-none items-center justify-center rounded-md border border-gray-300 text-gray-700 dark:border-amber-800 dark:text-amber-300 {FOCUS_RING}",
                                aria_label: "Menu",
                                aria_controls: "mobile-nav",
                                // Kept in step with `open` by the bootstrap.
                                aria_expanded: "false",
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
                // Full-bleed rows; the 3px current marker plus 13px aligns labels with the gutter.
                nav { id: "mobile-nav", class: "-mx-4 hidden group-has-[details[open]]:block md:hidden", aria_label: "Menu",
                    for item in site.nav.iter() {
                        MobileNavLink {
                            item: item.clone(),
                            current: is_current_nav(&item.path, &current_path),
                            exact: item.path == current_path,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NavLink(item: plinth_shared::config::NavItem, current: bool, exact: bool) -> Element {
    if current {
        rsx! {
            Link {
                to: item.path.clone(),
                class: "motion-safe:transition-colors {NAV_LINK_CURRENT} {FOCUS_RING}",
                aria_current: if exact { "page" } else { "true" },
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
fn MobileNavLink(item: plinth_shared::config::NavItem, current: bool, exact: bool) -> Element {
    if current {
        rsx! {
            Link {
                to: item.path.clone(),
                class: "block border-b border-b-gray-200 border-l-[3px] border-l-blue-600 bg-gray-100 py-3.5 pl-[13px] pr-4 text-[0.9375rem] font-medium text-gray-900 motion-safe:transition-colors dark:border-b-amber-900/50 dark:border-l-amber-500 dark:bg-neutral-950 dark:text-amber-100 {FOCUS_RING}",
                aria_current: if exact { "page" } else { "true" },
                onclick: |_| close_mobile_menu(),
                "{item.label}"
            }
        }
    } else {
        rsx! {
            Link {
                to: item.path.clone(),
                class: "block border-b border-b-gray-200 border-l-[3px] border-l-transparent py-3.5 pl-[13px] pr-4 text-[0.9375rem] text-gray-700 hover:bg-gray-50 motion-safe:transition-colors dark:border-b-amber-900/50 dark:text-amber-200 dark:hover:bg-white/5 {FOCUS_RING}",
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
            class: "flex h-11 w-11 shrink-0 items-center justify-center rounded-md border border-gray-300 text-gray-700 hover:bg-gray-100 hover:text-gray-900 motion-safe:transition-colors dark:border-amber-800 dark:text-amber-300 dark:hover:bg-white/5 dark:hover:text-amber-100 md:h-9 md:w-9 {FOCUS_RING}",
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
                Link { to: Route::Home {}, class: "order-2 truncate {FOOTER_LINK} {FOCUS_RING} md:order-1", "{site.name}" }
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
                class: "inline-flex min-h-11 shrink-0 items-center gap-1 text-sm font-medium md:min-h-0 {LINK} {FOCUS_RING}",
                "{action}"
                Arrow { d: "M5 12h14M13 6l6 6-6 6" }
            }
        }
    }
}

/// The way back to a list from a detail or filtered page.
#[cfg(any(feature = "brick-blog", feature = "brick-portfolio"))]
#[component]
fn BackLink(to: Route, label: String, #[props(default)] top: bool) -> Element {
    let spacing = if top { "-mt-2 mb-4" } else { "mt-8" };
    rsx! {
        div { class: "{spacing}",
            Link {
                to,
                class: "inline-flex min-h-11 items-center gap-1 text-sm font-medium {LINK} {FOCUS_RING}",
                Arrow { d: "M19 12H5M11 6l-6 6 6 6" }
                "{label}"
            }
        }
    }
}

/// A stroked arrow drawn inline: the text arrows fall back to another font
/// and sit small next to the label.
#[component]
fn Arrow(d: &'static str) -> Element {
    rsx! {
        svg {
            class: "h-3.5 w-3.5 shrink-0",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d }
        }
    }
}

/// One end of the older/newer pair under a post; `end` keeps "Newer" in the
/// right-hand column even when there is no older post.
#[cfg(feature = "brick-blog")]
#[component]
fn NeighbourLink(item: plinth_shared::BlogListItem, label: &'static str, end: bool) -> Element {
    let align = if end {
        "sm:col-start-2 sm:text-right"
    } else {
        ""
    };
    rsx! {
        Link {
            to: Route::Post { slug: item.slug.clone() },
            rel: if end { "next" } else { "prev" },
            class: "group flex min-h-11 flex-col {align} {FOCUS_RING}",
            span { class: "text-xs font-semibold uppercase tracking-[0.16em] {META}", "{label}" }
            span { class: "mt-1 text-balance font-semibold {HEADING} group-hover:text-blue-600 motion-safe:transition-colors dark:group-hover:text-amber-300", "{item.title}" }
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
    let heading_class = "text-balance font-semibold tracking-tight lg:col-start-1 lg:row-start-1";
    let title = rsx! {
        Link {
            to: Route::Post { slug: post.slug.clone() },
            class: "text-gray-900 after:absolute after:inset-0 hover:text-blue-600 motion-safe:transition-colors dark:text-amber-100 dark:hover:text-amber-300 {FOCUS_RING}",
            "{post.title}"
        }
    };

    rsx! {
        // The title link stretches over the row; tag chips sit above it.
        article { class: "relative flex flex-col gap-1 border-t pt-4 pb-4 {RULE} lg:pt-5 lg:pb-5 lg:grid lg:grid-cols-[minmax(0,1fr)_auto] lg:items-baseline lg:gap-x-6 lg:gap-y-1",
            // List pages sit under the page h1; the home sections under an h2.
            if large {
                h2 { class: "text-xl {heading_class}", {title} }
            } else {
                h3 { class: "text-lg {heading_class}", {title} }
            }
            if !post.description.is_empty() {
                p { class: "max-w-2xl text-[0.9375rem] leading-normal {BODY} lg:col-start-1 lg:row-start-2", "{post.description}" }
            }
            p { class: "text-sm whitespace-nowrap lg:col-start-2 lg:row-start-1 lg:text-right {META}",
                time { datetime: "{datetime}", "{date}" }
                "\u{a0}· {post.reading_time_minutes} min read"
            }
            if tags && !post.tags.is_empty() {
                div { class: "relative z-10 mt-2 flex flex-wrap gap-2 lg:col-span-2 lg:row-start-3",
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
            aria_labelledby: "project-{project.slug}",
            class: "group flex flex-col rounded-lg border border-gray-200 bg-white p-5 hover:border-gray-400 motion-safe:transition-colors dark:border-amber-900 dark:bg-neutral-950 dark:hover:border-amber-600 {FOCUS_RING}",
            if full && let Some(image_url) = project.image_url.clone() {
                img {
                    src: "{image_url}",
                    alt: "",
                    class: "mb-4 h-40 w-full rounded-md object-cover",
                }
            }
            div { class: "flex items-start justify-between gap-3",
                // The list page has no section heading between its h1 and the cards.
                if full {
                    h2 { id: "project-{project.slug}", class: "text-lg font-semibold {HEADING}", "{project.title}" }
                } else {
                    h3 { id: "project-{project.slug}", class: "text-lg font-semibold {HEADING}", "{project.title}" }
                }
                // The whole card is the link; the arrow says so.
                span { class: "mt-1.5 text-gray-400 group-hover:text-blue-600 motion-safe:transition-colors dark:text-amber-700 dark:group-hover:text-amber-300",
                    Arrow { d: "M5 12h14M13 6l6 6-6 6" }
                }
            }
            p { class: "mt-2 text-[0.9375rem] leading-relaxed {BODY}", "{project.description}" }
            if !project.tech_stack.is_empty() {
                div { class: "mt-auto flex flex-wrap gap-2 pt-4",
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
            span { class: "min-w-0 text-[0.9375rem] font-medium text-gray-900 hover:text-blue-600 motion-safe:transition-colors dark:text-amber-100 dark:hover:text-amber-300", "{item.title}" }
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
                h1 { class: "text-balance text-[2.125rem] font-bold leading-tight tracking-tight {HEADING} md:mt-1 md:text-5xl", "{site.name}" }
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
        let content = use_loader(use_reactive!(|content_key| loaders::load_site_content(
            content_key
        )))?;
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
                div { class: "mt-8",
                    Link { to: Route::Home {}, class: "{BUTTON} {FOCUS_RING}", "Return home" }
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
            Shell { title: site.pages.blog.title.clone(), description: site.pages.blog.subtitle.clone(),
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
        let post = use_loader(use_reactive!(|slug| loaders::load_post(slug)))?;
        let posts = use_loader(loaders::load_posts)?;
        let Some(post) = post().clone() else {
            return rsx! { MissingPage { path: format!("/posts/{slug}") } };
        };
        // The list is newest first: the previous post is the next entry.
        let posts = posts();
        let index = posts.iter().position(|item| item.slug == post.slug);
        let previous = index.and_then(|index| posts.get(index + 1)).cloned();
        let next = index
            .and_then(|index| index.checked_sub(1))
            .and_then(|index| posts.get(index))
            .cloned();
        let datetime = post.published_at.to_rfc3339();
        let date = post.published_at.format("%b %-d, %Y").to_string();
        return rsx! {
            Shell { title: post.title.clone(), description: post.description.clone(),
                article { class: "max-w-2xl",
                    p { class: "mb-4 text-sm {META}",
                        time { datetime: "{datetime}", "{date}" }
                        "\u{a0}· {post.reading_time_minutes} min read · "
                        span { class: "whitespace-nowrap", "{post.author}" }
                    }
                    h1 { class: "{H1}", "{post.title}" }
                    if !post.description.is_empty() {
                        p { class: "mt-4 text-pretty text-xl leading-relaxed {HEADING}", "{post.description}" }
                    }
                    div { class: "prose prose-lg mt-8 max-w-none dark:prose-invert", dangerous_inner_html: article_html(&post.html_content, &post.title) }
                    if !post.tags.is_empty() {
                        footer { class: "mt-8 flex flex-wrap gap-2",
                            for tag in post.tags.iter() {
                                TagChip { tag: tag.clone() }
                            }
                        }
                    }
                    if previous.is_some() || next.is_some() {
                        nav { class: "mt-10 grid gap-6 border-t pt-6 sm:grid-cols-2 {RULE}", aria_label: "More posts",
                            if let Some(item) = previous {
                                NeighbourLink { item, label: "Older", end: false }
                            }
                            if let Some(item) = next {
                                NeighbourLink { item, label: "Newer", end: true }
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
        let posts = use_loader(use_reactive!(|tag| loaders::load_posts_by_tag(tag)))?;
        let posts_value = posts().clone();
        if posts_value.is_empty() {
            return rsx! { MissingPage { path: format!("/posts/tag/{tag}") } };
        }
        return rsx! {
            Shell { title: format!("Posts tagged #{tag}"), description: format!("Posts tagged #{tag}."),
                BackLink { to: Route::Posts {}, label: "All posts".to_string(), top: true }
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
        let posts = use_loader(use_reactive!(|slug| loaders::load_series_posts(slug)))?;
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
            Shell { title: site.pages.portfolio.title.clone(), description: site.pages.portfolio.subtitle.clone(),
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
        let project = use_loader(use_reactive!(|slug| loaders::load_project(slug)))?;
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
                    p { class: "mt-4 text-pretty text-xl leading-relaxed {HEADING}", "{project.description}" }
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
                    div { class: "mt-8 flex flex-wrap gap-3",
                        if let Some(link) = project.link.clone() {
                            a { href: "{link}", rel: "noreferrer", class: "{BUTTON} {FOCUS_RING}", "Source" Arrow { d: "M7 17L17 7M8 7h9v9" } }
                        }
                        if let Some(demo) = project.demo.clone() {
                            a { href: "{demo}", rel: "noreferrer", class: "{BUTTON} {FOCUS_RING}", "Demo" Arrow { d: "M7 17L17 7M8 7h9v9" } }
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
        let item = use_loader(use_reactive!(|id| loaders::load_activity_item(id)))?;
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
            Shell { title: site.pages.todos.title.clone(), description: site.pages.todos.subtitle.clone(),
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
                            p { class: "text-[0.9375rem] leading-relaxed {BODY}", "{todo.description}" }
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
        let todos = use_loader(use_reactive!(|tag| loaders::load_todos_by_tag(tag)))?;
        let todos_value = todos().clone();
        if todos_value.is_empty() {
            return rsx! { MissingPage { path: format!("/todos/tag/{tag}") } };
        }
        return rsx! {
            Shell { title: format!("Todos tagged #{tag}"), description: format!("Todos tagged #{tag}."),
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
                            p { class: "mt-1 text-[0.9375rem] leading-relaxed {BODY}", "{todo.description}" }
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
        let todo = use_loader(use_reactive!(|slug| loaders::load_todo(slug)))?;
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
                    p { class: "mt-4 text-pretty text-xl leading-relaxed {HEADING}", "{todo.description}" }
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
    use super::{accessible_html, article_html};

    #[test]
    fn article_scroll_areas_and_task_boxes_are_accessible() {
        let html = accessible_html(
            "<pre><code>x</code></pre><pre class=\"a\">y</pre><table><tr><td>1</td></tr></table>\
             <li><input disabled=\"\" type=\"checkbox\"/>t</li>",
        );
        assert_eq!(html.matches("<pre tabindex=\"0\"").count(), 2, "{html}");
        assert!(html.contains("<div class=\"table-scroll\" tabindex=\"0\"><table>"));
        assert!(html.contains("</table></div>"));
        assert!(html.contains("type=\"checkbox\" aria-label=\"Task\""));
    }

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
