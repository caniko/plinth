use leptos_router::SsrMode;

#[cfg(feature = "ssr")]
use futures::{StreamExt, stream::BoxStream};
#[cfg(feature = "ssr")]
use leptos_router::{
    params::ParamsMap,
    static_routes::{StaticParamsMap, StaticRoute},
};
#[cfg(feature = "ssr")]
use std::sync::LazyLock;

#[cfg(feature = "ssr")]
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) enum StaticInvalidation {
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
pub(crate) enum StaticRegenerationScope {
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
/// Broadcast an invalidation event for blog static routes (index,
/// post detail, tag pages, series index/slug) so the static-regeneration
/// stream re-renders affected pages.
pub fn invalidate_blog_static_routes(slug: &str, tags: &[String], series_slug: Option<&str>) {
    let _ = STATIC_INVALIDATIONS.send(StaticInvalidation::Blog {
        slug: slug.to_string(),
        tags: tags.to_vec(),
        series_slug: series_slug.map(ToOwned::to_owned),
    });
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
/// Broadcast an invalidation event for portfolio static routes (index
/// and detail) so the static-regeneration stream re-renders affected pages.
pub fn invalidate_portfolio_static_routes(slug: &str) {
    let _ = STATIC_INVALIDATIONS.send(StaticInvalidation::Portfolio {
        slug: slug.to_string(),
    });
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
/// Broadcast an invalidation event for a site-content static route
/// (e.g. about, support) so the static-regeneration stream re-renders.
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
pub(crate) fn static_mode(scope: StaticRegenerationScope) -> SsrMode {
    SsrMode::Static(StaticRoute::new().regenerate(regenerate_on(scope)))
}

#[cfg(not(feature = "ssr"))]
#[allow(dead_code)]
pub(crate) fn static_mode(_scope: StaticRegenerationScope) -> SsrMode {
    SsrMode::OutOfOrder
}

#[cfg(feature = "ssr")]
#[allow(dead_code)]
pub(crate) fn static_mode_with_params<Fut>(
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
