use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::{SsrMode, path};

#[cfg(feature = "ssr")]
use leptos_router::static_routes::StaticParamsMap;

use crate::pages::*;

use super::invalidation::{StaticRegenerationScope, static_mode};

#[cfg(feature = "ssr")]
use super::invalidation::static_mode_with_params;

#[cfg(feature = "ssr")]
use sqlx::Row;

#[cfg(feature = "ssr")]
#[allow(dead_code)]
async fn query_static_param_values(sql: &'static str, column: &'static str) -> Vec<String> {
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
    use leptos_router::static_routes::StaticParamsMap;

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
    use leptos_router::static_routes::StaticParamsMap;

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
    use leptos_router::static_routes::StaticParamsMap;

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
    use leptos_router::static_routes::StaticParamsMap;

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

/// Build all routes. This function exists so we can use #[cfg] to compose
/// the route tuple at compile time — Leptos Routes requires a statically
/// known tuple of Route components.
#[cfg(all(
    feature = "brick-blog",
    feature = "brick-portfolio",
    feature = "brick-todo",
    feature = "brick-activity"
))]
pub(crate) fn app_routes() -> impl IntoView {
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
pub(crate) fn app_routes() -> impl IntoView {
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
