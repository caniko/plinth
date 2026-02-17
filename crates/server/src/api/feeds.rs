use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use rss::{CategoryBuilder, ChannelBuilder, GuidBuilder, ItemBuilder};

use crate::AppState;
use crate::actors::content_cache::{GetAllBlogPosts, GetAllPortfolioItems};

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
pub async fn blog_feed(State(state): State<AppState>) -> Result<Response, StatusCode> {
    let base_url = resolve_base_url(&state);
    let site = &state.site_config;
    let limit = state.config.feeds.blog_limit;

    let posts = state
        .content_cache
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
pub async fn projects_feed(State(state): State<AppState>) -> Result<Response, StatusCode> {
    let base_url = resolve_base_url(&state);
    let site = &state.site_config;
    let limit = state.config.feeds.projects_limit;

    let items_data = state
        .content_cache
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

#[cfg(test)]
mod tests {
    use rss::ChannelBuilder;

    #[test]
    fn test_blog_feed_xml_structure() {
        let channel = ChannelBuilder::default()
            .title("Test Blog")
            .link("https://example.com/posts")
            .description("A test blog")
            .build();
        let xml = channel.to_string();
        assert!(xml.contains("<rss"));
        assert!(xml.contains("Test Blog"));
        assert!(xml.contains("https://example.com/posts"));
    }

    #[test]
    fn test_rfc2822_date_format() {
        let date = chrono::Utc::now();
        let formatted = date.to_rfc2822();
        assert!(!formatted.is_empty());
        // RFC 2822 dates contain a day abbreviation like "Mon," or "Tue,"
        assert!(formatted.contains('+') || formatted.contains('-'));
    }
}
