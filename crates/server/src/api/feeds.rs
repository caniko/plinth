use std::fmt::Write as _;

use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

use crate::AppState;

/// Escape a string for safe inclusion in XML content.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Resolve the public base URL from configuration.
/// Uses `site.base_url` if set, otherwise falls back to `http://{host}:{port}`.
fn resolve_base_url(state: &AppState) -> String {
    let base = &state.config.site.base_url;
    if !base.is_empty() {
        base.trim_end_matches('/').to_string()
    } else {
        format!(
            "http://{}:{}",
            state.config.server.host, state.config.server.port
        )
    }
}

/// GET /feeds/blog.xml — RSS feed of published blog posts
#[cfg(feature = "brick-blog")]
pub async fn blog_feed(State(state): State<AppState>) -> Result<Response, StatusCode> {
    use crate::bricks::blog::cache::GetAllBlogPosts;
    use rss::{CategoryBuilder, ChannelBuilder, GuidBuilder, ItemBuilder};

    let base_url = resolve_base_url(&state);
    let site = &state.site_config;
    let limit = state.config.feeds.blog_limit;

    let posts = state
        .blog_cache
        .ask(GetAllBlogPosts)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let items: Vec<rss::Item> = posts
        .into_iter()
        .take(limit)
        .map(|post| {
            let link = format!("{}/posts/{}", base_url, post.slug);
            let categories: Vec<rss::Category> = post
                .tags
                .iter()
                .map(|tag: &String| CategoryBuilder::default().name(tag.clone()).build())
                .collect();

            ItemBuilder::default()
                .title(Some(post.title))
                .link(Some(link.clone()))
                .description(Some(post.description))
                .author(Some(post.author))
                .categories(categories)
                .guid(Some(
                    GuidBuilder::default().value(link).permalink(true).build(),
                ))
                .pub_date(Some(post.published_at.to_rfc2822()))
                .build()
        })
        .collect();

    let mut builder = ChannelBuilder::default();
    builder
        .title(format!("{} - {}", site.name, site.pages.blog.title))
        .link(format!("{}/posts", base_url))
        .description(if site.pages.blog.description.is_empty() {
            site.description.clone()
        } else {
            site.pages.blog.description.clone()
        })
        .language(Some(site.lang.clone()))
        .last_build_date(Some(chrono::Utc::now().to_rfc2822()))
        .items(items);

    if !site.author.email.is_empty() {
        builder.managing_editor(Some(format!(
            "{} ({})",
            site.author.email, site.author.name
        )));
    }

    let channel = builder.build();
    let xml = channel.to_string();

    Ok((
        [
            (header::CONTENT_TYPE, "application/rss+xml; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        xml,
    )
        .into_response())
}

/// GET /feeds/projects.xml — RSS feed of portfolio items
#[cfg(feature = "brick-portfolio")]
pub async fn projects_feed(State(state): State<AppState>) -> Result<Response, StatusCode> {
    use crate::bricks::portfolio::cache::GetAllPortfolioItems;
    use rss::{CategoryBuilder, ChannelBuilder, GuidBuilder, ItemBuilder};

    let base_url = resolve_base_url(&state);
    let site = &state.site_config;
    let limit = state.config.feeds.projects_limit;

    let items_data = state
        .portfolio_cache
        .ask(GetAllPortfolioItems)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let items: Vec<rss::Item> = items_data
        .into_iter()
        .take(limit)
        .map(|item| {
            let link = format!("{}/projects/{}", base_url, item.slug);
            let categories: Vec<rss::Category> = item
                .tech_stack
                .iter()
                .map(|tech: &String| CategoryBuilder::default().name(tech.clone()).build())
                .collect();

            ItemBuilder::default()
                .title(Some(item.title))
                .link(Some(link.clone()))
                .description(Some(item.description))
                .categories(categories)
                .guid(Some(
                    GuidBuilder::default().value(link).permalink(true).build(),
                ))
                .pub_date(Some(item.date.to_rfc2822()))
                .build()
        })
        .collect();

    let channel = ChannelBuilder::default()
        .title(format!("{} - {}", site.name, site.pages.portfolio.title))
        .link(format!("{}/projects", base_url))
        .description(if site.pages.portfolio.description.is_empty() {
            site.description.clone()
        } else {
            site.pages.portfolio.description.clone()
        })
        .language(Some(site.lang.clone()))
        .last_build_date(Some(chrono::Utc::now().to_rfc2822()))
        .items(items)
        .build();

    let xml = channel.to_string();

    Ok((
        [
            (header::CONTENT_TYPE, "application/rss+xml; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        xml,
    )
        .into_response())
}

/// GET /feeds/series/:slug.xml — RSS feed for a specific blog series
#[cfg(feature = "brick-blog")]
pub async fn series_feed(
    State(state): State<AppState>,
    axum::extract::Path(slug): axum::extract::Path<String>,
) -> Result<Response, StatusCode> {
    use crate::bricks::blog::cache::GetSeriesPosts;
    use rss::{CategoryBuilder, ChannelBuilder, GuidBuilder, ItemBuilder};

    // Strip .xml extension if present
    let series_slug = slug.strip_suffix(".xml").unwrap_or(&slug).to_string();

    let base_url = resolve_base_url(&state);
    let site = &state.site_config;

    let posts = state
        .blog_cache
        .ask(GetSeriesPosts(series_slug.clone()))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if posts.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let series_title = posts[0]
        .series_title
        .clone()
        .unwrap_or_else(|| series_slug.clone());

    let items: Vec<rss::Item> = posts
        .into_iter()
        .map(|post| {
            let link = format!("{}/posts/{}", base_url, post.slug);
            let categories: Vec<rss::Category> = post
                .tags
                .iter()
                .map(|tag: &String| CategoryBuilder::default().name(tag.clone()).build())
                .collect();

            ItemBuilder::default()
                .title(Some(post.title))
                .link(Some(link.clone()))
                .description(Some(post.description))
                .author(Some(post.author))
                .categories(categories)
                .guid(Some(
                    GuidBuilder::default().value(link).permalink(true).build(),
                ))
                .pub_date(Some(post.published_at.to_rfc2822()))
                .build()
        })
        .collect();

    let channel = ChannelBuilder::default()
        .title(format!("{} - {}", site.name, series_title))
        .link(format!("{}/series/{}", base_url, series_slug))
        .description(format!("Posts in the \"{}\" series", series_title))
        .language(Some(site.lang.clone()))
        .last_build_date(Some(chrono::Utc::now().to_rfc2822()))
        .items(items)
        .build();

    let xml = channel.to_string();

    Ok((
        [
            (header::CONTENT_TYPE, "application/rss+xml; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        xml,
    )
        .into_response())
}

/// GET /sitemap.xml — dynamic XML sitemap of all published content
pub async fn sitemap_xml(State(state): State<AppState>) -> Result<Response, StatusCode> {
    let base_url = resolve_base_url(&state);

    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#,
    );

    let escaped_base = xml_escape(&base_url);

    // Static pages (always present)
    #[allow(clippy::vec_init_then_push)]
    let mut static_pages = vec![("/", "weekly", "1.0"), ("/about", "monthly", "0.8")];

    #[cfg(feature = "brick-blog")]
    static_pages.push(("/posts", "daily", "0.9"));
    #[cfg(feature = "brick-portfolio")]
    static_pages.push(("/projects", "weekly", "0.9"));

    for (path, changefreq, priority) in static_pages {
        let _ = write!(
            xml,
            "  <url>\n    <loc>{escaped_base}{path}</loc>\n    <changefreq>{changefreq}</changefreq>\n    <priority>{priority}</priority>\n  </url>\n",
        );
    }

    // Blog posts
    #[cfg(feature = "brick-blog")]
    {
        use crate::bricks::blog::cache::GetAllBlogPosts;
        let posts = state
            .blog_cache
            .ask(GetAllBlogPosts)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        for post in &posts {
            let lastmod = post.published_at.format("%Y-%m-%d");
            let _ = write!(
                xml,
                "  <url>\n    <loc>{escaped_base}/posts/{}</loc>\n    <lastmod>{lastmod}</lastmod>\n    <changefreq>monthly</changefreq>\n    <priority>0.7</priority>\n  </url>\n",
                xml_escape(&post.slug)
            );
        }
    }

    // Portfolio items
    #[cfg(feature = "brick-portfolio")]
    {
        use crate::bricks::portfolio::cache::GetAllPortfolioItems;
        let portfolio = state
            .portfolio_cache
            .ask(GetAllPortfolioItems)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        for item in &portfolio {
            let lastmod = item.date.format("%Y-%m-%d");
            let _ = write!(
                xml,
                "  <url>\n    <loc>{escaped_base}/projects/{}</loc>\n    <lastmod>{lastmod}</lastmod>\n    <changefreq>monthly</changefreq>\n    <priority>0.6</priority>\n  </url>\n",
                xml_escape(&item.slug)
            );
        }
    }

    xml.push_str("</urlset>\n");

    Ok((
        [
            (header::CONTENT_TYPE, "application/xml; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        xml,
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_escape_ampersand() {
        assert_eq!(xml_escape("foo&bar"), "foo&amp;bar");
    }

    #[test]
    fn test_xml_escape_angle_brackets() {
        assert_eq!(xml_escape("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn test_xml_escape_quotes() {
        assert_eq!(xml_escape(r#"a"b'c"#), "a&quot;b&apos;c");
    }

    #[test]
    fn test_xml_escape_passthrough() {
        assert_eq!(xml_escape("hello-world_123"), "hello-world_123");
    }

    #[test]
    fn test_xml_escape_combined() {
        assert_eq!(
            xml_escape("a&b<c>d\"e'f"),
            "a&amp;b&lt;c&gt;d&quot;e&apos;f"
        );
    }
}
