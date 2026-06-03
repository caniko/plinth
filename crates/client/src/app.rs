use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::{SsrMode, path};
use plinth_shared::SiteConfig;

use crate::components::{Footer, Header};
use crate::pages::*;

#[cfg(any(feature = "csr", not(feature = "islands")))]
use crate::api;

#[cfg(feature = "ssr")]
use futures::{StreamExt, stream::BoxStream};
#[cfg(feature = "ssr")]
use leptos_router::{
    params::ParamsMap,
    static_routes::{StaticParamsMap, StaticRoute},
};
#[cfg(feature = "ssr")]
use std::sync::LazyLock;

/// Rendering-mode source of truth for the default all-bricks route table.
///
/// Static routes are publish-cadence content regenerated through Leptos
/// `StaticRoute::regenerate` streams when their admin publish/update path runs.
/// Request-time routes intentionally remain dynamic.
#[allow(dead_code)]
pub const ROUTE_RENDERING_MODES: &[(&str, &str)] = &[
    (
        "/",
        "SsrMode::OutOfOrder (streaming SSR; Phase 03 owns home)",
    ),
    ("/about", "SsrMode::Static (site content key: about)"),
    ("/support", "SsrMode::Static (site content key: support)"),
    ("/posts", "SsrMode::Static (blog index)"),
    ("/posts/:slug", "SsrMode::Static (blog post slug)"),
    ("/posts/tag/:tag", "SsrMode::Static (blog tag name/slug)"),
    ("/series", "SsrMode::Static (series index)"),
    ("/series/:slug", "SsrMode::Static (series slug)"),
    ("/projects", "SsrMode::Static (portfolio index)"),
    ("/projects/:slug", "SsrMode::Static (portfolio slug)"),
    ("/activity", "SsrMode::OutOfOrder (request-time SSR)"),
    ("/activity/:id", "SsrMode::OutOfOrder (request-time SSR)"),
    ("/todos", "SsrMode::OutOfOrder (request-time SSR)"),
    ("/todos/tag/:tag", "SsrMode::OutOfOrder (request-time SSR)"),
    ("/todos/:slug", "SsrMode::OutOfOrder (request-time SSR)"),
];

#[cfg(feature = "ssr")]
#[derive(Clone, Debug)]
#[allow(dead_code)]
enum StaticInvalidation {
    Blog {
        slug: String,
        tags: Vec<String>,
        series_slug: Option<String>,
    },
    Portfolio {
        slug: String,
    },
    SiteContent {
        key: String,
    },
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum StaticRegenerationScope {
    BlogIndex,
    BlogPostSlug,
    BlogTag,
    SeriesIndex,
    SeriesSlug,
    PortfolioIndex,
    PortfolioSlug,
    SiteContentKey(&'static str),
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
static STATIC_INVALIDATIONS: LazyLock<tokio::sync::broadcast::Sender<StaticInvalidation>> =
    LazyLock::new(|| {
        let (tx, _rx) = tokio::sync::broadcast::channel(256);
        tx
    });

#[cfg(feature = "ssr")]
#[allow(dead_code)]
pub fn invalidate_blog_static_routes(slug: &str, tags: &[String], series_slug: Option<&str>) {
    let _ = STATIC_INVALIDATIONS.send(StaticInvalidation::Blog {
        slug: slug.to_string(),
        tags: tags.to_vec(),
        series_slug: series_slug.map(ToOwned::to_owned),
    });
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
pub fn invalidate_portfolio_static_routes(slug: &str) {
    let _ = STATIC_INVALIDATIONS.send(StaticInvalidation::Portfolio {
        slug: slug.to_string(),
    });
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
pub fn invalidate_site_content_static_routes(key: &str) {
    let _ = STATIC_INVALIDATIONS.send(StaticInvalidation::SiteContent {
        key: key.to_string(),
    });
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
fn event_matches_scope(
    event: &StaticInvalidation,
    scope: StaticRegenerationScope,
    params: &ParamsMap,
) -> bool {
    match (event, scope) {
        (StaticInvalidation::Blog { .. }, StaticRegenerationScope::BlogIndex) => true,
        (StaticInvalidation::Blog { slug, .. }, StaticRegenerationScope::BlogPostSlug) => {
            params.get_str("slug") == Some(slug.as_str())
        }
        (StaticInvalidation::Blog { tags, .. }, StaticRegenerationScope::BlogTag) => params
            .get_str("tag")
            .is_some_and(|tag| tags.iter().any(|event_tag| event_tag == tag)),
        (
            StaticInvalidation::Blog {
                series_slug: Some(_),
                ..
            },
            StaticRegenerationScope::SeriesIndex,
        ) => true,
        (
            StaticInvalidation::Blog {
                series_slug: Some(series_slug),
                ..
            },
            StaticRegenerationScope::SeriesSlug,
        ) => params.get_str("slug") == Some(series_slug.as_str()),
        (StaticInvalidation::Portfolio { .. }, StaticRegenerationScope::PortfolioIndex) => true,
        (StaticInvalidation::Portfolio { slug }, StaticRegenerationScope::PortfolioSlug) => {
            params.get_str("slug") == Some(slug.as_str())
        }
        (
            StaticInvalidation::SiteContent { key },
            StaticRegenerationScope::SiteContentKey(want),
        ) => key == want,
        _ => false,
    }
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
fn regenerate_on(
    scope: StaticRegenerationScope,
) -> impl Fn(&ParamsMap) -> BoxStream<'static, ()> + Send + Sync + 'static {
    move |params| {
        let params = params.clone();
        let rx = STATIC_INVALIDATIONS.subscribe();

        futures::stream::unfold(rx, move |mut rx| {
            let params = params.clone();
            async move {
                loop {
                    match rx.recv().await {
                        Ok(event) if event_matches_scope(&event, scope, &params) => {
                            return Some(((), rx));
                        }
                        Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
                    }
                }
            }
        })
        .boxed()
    }
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
fn static_mode(scope: StaticRegenerationScope) -> SsrMode {
    SsrMode::Static(StaticRoute::new().regenerate(regenerate_on(scope)))
}

#[cfg(not(feature = "ssr"))]
#[allow(dead_code)]
fn static_mode(_scope: StaticRegenerationScope) -> SsrMode {
    SsrMode::OutOfOrder
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
fn static_mode_with_params<Fut>(
    scope: StaticRegenerationScope,
    params: impl Fn() -> Fut + Send + Sync + 'static,
) -> SsrMode
where
    Fut: std::future::Future<Output = StaticParamsMap> + Send + 'static,
{
    SsrMode::Static(
        StaticRoute::new()
            .prerender_params(params)
            .regenerate(regenerate_on(scope)),
    )
}

#[allow(dead_code)]
fn blog_post_static_mode() -> SsrMode {
    #[cfg(feature = "ssr")]
    {
        static_mode_with_params(StaticRegenerationScope::BlogPostSlug, blog_slug_params)
    }
    #[cfg(not(feature = "ssr"))]
    {
        SsrMode::OutOfOrder
    }
}

#[allow(dead_code)]
fn blog_tag_static_mode() -> SsrMode {
    #[cfg(feature = "ssr")]
    {
        static_mode_with_params(StaticRegenerationScope::BlogTag, blog_tag_params)
    }
    #[cfg(not(feature = "ssr"))]
    {
        SsrMode::OutOfOrder
    }
}

#[allow(dead_code)]
fn series_static_mode() -> SsrMode {
    #[cfg(feature = "ssr")]
    {
        static_mode_with_params(StaticRegenerationScope::SeriesSlug, series_slug_params)
    }
    #[cfg(not(feature = "ssr"))]
    {
        SsrMode::OutOfOrder
    }
}

#[allow(dead_code)]
fn portfolio_static_mode() -> SsrMode {
    #[cfg(feature = "ssr")]
    {
        static_mode_with_params(
            StaticRegenerationScope::PortfolioSlug,
            portfolio_slug_params,
        )
    }
    #[cfg(not(feature = "ssr"))]
    {
        SsrMode::OutOfOrder
    }
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
async fn query_static_param_values(sql: &'static str, column: &'static str) -> Vec<String> {
    use sqlx::Row;

    let db = expect_context::<sqlx::PgPool>();
    sqlx::query(sql)
        .fetch_all(&db)
        .await
        .unwrap_or_else(|e| panic!("failed to enumerate static route params for {column}: {e}"))
        .into_iter()
        .map(|row| {
            row.try_get::<String, _>(column)
                .unwrap_or_else(|e| panic!("failed to decode static route param {column}: {e}"))
        })
        .collect()
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
async fn blog_slug_params() -> StaticParamsMap {
    let slugs = query_static_param_values(
        r#"
        SELECT slug
        FROM blog_posts
        WHERE published = true
        ORDER BY published_at DESC, id DESC
        "#,
        "slug",
    )
    .await;
    let mut params = StaticParamsMap::new();
    params.insert("slug", slugs);
    params
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
async fn blog_tag_params() -> StaticParamsMap {
    let tags = query_static_param_values(
        r#"
        SELECT tag
        FROM (
            SELECT DISTINCT t.name AS tag
            FROM tags t
            JOIN blog_post_tags bpt ON bpt.tag_id = t.id
            JOIN blog_posts bp ON bp.id = bpt.post_id
            WHERE bp.published = true
            UNION
            SELECT DISTINCT t.slug AS tag
            FROM tags t
            JOIN blog_post_tags bpt ON bpt.tag_id = t.id
            JOIN blog_posts bp ON bp.id = bpt.post_id
            WHERE bp.published = true
        ) tags
        ORDER BY tag ASC
        "#,
        "tag",
    )
    .await;
    let mut params = StaticParamsMap::new();
    params.insert("tag", tags);
    params
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
async fn series_slug_params() -> StaticParamsMap {
    let slugs = query_static_param_values(
        r#"
        SELECT DISTINCT series_slug AS slug
        FROM blog_posts
        WHERE series_slug IS NOT NULL AND published = true
        ORDER BY series_slug ASC
        "#,
        "slug",
    )
    .await;
    let mut params = StaticParamsMap::new();
    params.insert("slug", slugs);
    params
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
async fn portfolio_slug_params() -> StaticParamsMap {
    let slugs = query_static_param_values(
        r#"
        SELECT slug
        FROM portfolio_items
        ORDER BY "order" ASC, date DESC, id DESC
        "#,
        "slug",
    )
    .await;
    let mut params = StaticParamsMap::new();
    params.insert("slug", slugs);
    params
}

/// Helper to get site config from context (falls back to defaults).
///
/// On SSR, the server provides `SiteConfig` via `provide_context`.
/// On the client, the App component provides it after hydrating from the
/// serialized Resource data.
pub fn use_site_config() -> SiteConfig {
    use_context::<SiteConfig>().unwrap_or_default()
}

#[component]
pub fn App() -> impl IntoView {
    // Provide meta context for SEO
    provide_meta_context();

    // Provide SiteConfig as context eagerly:
    // - On SSR: the server provides it via leptos_routes_with_context,
    //   so use_context finds the real config.
    // - On CSR/full hydration: falls back to default until the provider below resolves.
    // In islands mode the App is not hydrated, so islands receive explicit props instead.
    let initial_config = use_context::<SiteConfig>().unwrap_or_default();
    provide_context(initial_config);

    #[cfg(any(feature = "csr", not(feature = "islands")))]
    let config_provider = {
        // Fetch site config once for CSR/full-app hydration.
        let site_config = Resource::new(|| (), |_| api::get_site_config());

        view! {
            <Suspense fallback=|| ()>
                {move || site_config.get().map(|result| {
                    let config = result.unwrap_or_default();
                    provide_context(config);
                })}
            </Suspense>
        }
    };

    #[cfg(all(not(feature = "csr"), feature = "islands"))]
    let config_provider = ();

    view! {
        // Meta tags
        <Stylesheet id="leptos" href="/pkg/plinth.css"/>
        <Link rel="icon" type_="image/svg+xml" href="/favicon.svg"/>
        <Link rel="icon" type_="image/png" sizes="32x32" href="/favicon-32x32.png"/>
        <Link rel="icon" type_="image/png" sizes="16x16" href="/favicon-16x16.png"/>
        <Link rel="apple-touch-icon" sizes="180x180" href="/favicon-180x180.png"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

        // htmx script (vendored locally)
        <Script src="/htmx.min.js"/>

        {config_provider}

        <Router>
            <div class="flex flex-col min-h-screen bg-gray-50 dark:bg-black text-gray-900 dark:text-amber-100">
                // Header with navigation
                <Header/>

                // Main content
                <main class="flex-grow">
                    {app_routes()}
                </main>

                // Footer
                <Footer/>
            </div>
        </Router>
    }
}

/// Build all routes. This function exists so we can use #[cfg] to compose
/// the route tuple at compile time — Leptos Routes requires a statically
/// known tuple of Route components.
#[cfg(all(
    feature = "brick-blog",
    feature = "brick-portfolio",
    feature = "brick-todo",
    feature = "brick-activity"
))]
fn app_routes() -> impl IntoView {
    view! {
        <Routes fallback=|| view! { <NotFound/> }>
            <Route path=path!("/") view=HomePage ssr=SsrMode::OutOfOrder/>
            <Route
                path=path!("/about")
                view=AboutPage
                ssr=static_mode(StaticRegenerationScope::SiteContentKey("about"))
            />
            <Route
                path=path!("/support")
                view=SupportPage
                ssr=static_mode(StaticRegenerationScope::SiteContentKey("support"))
            />
            <Route
                path=path!("/posts")
                view=BlogListPage
                ssr=static_mode(StaticRegenerationScope::BlogIndex)
            />
            <Route path=path!("/posts/:slug") view=BlogPostPage ssr=blog_post_static_mode()/>
            <Route path=path!("/posts/tag/:tag") view=BlogTagPage ssr=blog_tag_static_mode()/>
            <Route
                path=path!("/series")
                view=SeriesListPage
                ssr=static_mode(StaticRegenerationScope::SeriesIndex)
            />
            <Route path=path!("/series/:slug") view=SeriesDetailPage ssr=series_static_mode()/>
            <Route
                path=path!("/projects")
                view=PortfolioPage
                ssr=static_mode(StaticRegenerationScope::PortfolioIndex)
            />
            <Route path=path!("/projects/:slug") view=PortfolioDetailPage ssr=portfolio_static_mode()/>
            <Route path=path!("/activity") view=ActivityPage ssr=SsrMode::OutOfOrder/>
            <Route path=path!("/activity/:id") view=ActivityDetailPage ssr=SsrMode::OutOfOrder/>
            <Route path=path!("/todos") view=TodoListPage ssr=SsrMode::OutOfOrder/>
            <Route path=path!("/todos/tag/:tag") view=TodoTagPage ssr=SsrMode::OutOfOrder/>
            <Route path=path!("/todos/:slug") view=TodoDetailPage ssr=SsrMode::OutOfOrder/>
        </Routes>
    }
}

// When not all bricks are enabled, fall back to a minimal set.
// In practice, the default feature set enables all bricks, so this is
// only used for custom builds with specific bricks disabled.
#[cfg(not(all(
    feature = "brick-blog",
    feature = "brick-portfolio",
    feature = "brick-todo",
    feature = "brick-activity"
)))]
fn app_routes() -> impl IntoView {
    // Build a routes view with only enabled brick routes.
    // We use nested cfg to include each brick's routes.
    view! {
        <Routes fallback=|| view! { <NotFound/> }>
            <Route path=path!("/") view=HomePage ssr=SsrMode::OutOfOrder/>
            <Route
                path=path!("/about")
                view=AboutPage
                ssr=static_mode(StaticRegenerationScope::SiteContentKey("about"))
            />
            <Route
                path=path!("/support")
                view=SupportPage
                ssr=static_mode(StaticRegenerationScope::SiteContentKey("support"))
            />
        </Routes>
    }
}
