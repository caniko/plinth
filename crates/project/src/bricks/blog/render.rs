//! HTML fragments and RSS output for blog pages.

use crate::ProjectSite;
use crate::render::{escape_attr, escape_text};

use super::{Blog, BlogPost};

/// Markup for `blog/index.html`: every published post, newest first.
pub(crate) fn render_index(blog: &Blog) -> String {
    let mut items = String::new();
    for post in &blog.posts {
        items.push_str(&format!(
            "<li class=\"post-list-item\"><article><h2><a href=\"/blog/{slug}/\">{title}</a></h2>{meta}{description}{tags}</article></li>",
            meta = meta(None, post),
            description = if post.description.is_empty() {
                String::new()
            } else {
                format!(
                    "<p class=\"post-description\">{}</p>",
                    escape_text(&post.description)
                )
            },
            tags = tags(post),
            slug = escape_attr(&post.slug),
            title = escape_text(&post.title),
        ));
    }
    format!(
        "<div class=\"blog-index\"><header class=\"blog-header\"><h1>{}</h1></header><ol class=\"post-list\">{items}</ol></div>",
        escape_text(&blog.title)
    )
}

/// Markup for `blog/<slug>/index.html`: the rendered post body inside the
/// site chrome supplied by the caller.
pub(crate) fn render_post(blog: &Blog, post: &BlogPost) -> String {
    format!(
        "<article class=\"blog-post\"><header class=\"post-header\">{}<h1>{}</h1>{}{}</header><div class=\"prose\">{}</div></article>",
        meta(Some(&blog.title), post),
        escape_text(&post.title),
        if post.description.is_empty() {
            String::new()
        } else {
            format!(
                "<p class=\"post-description\">{}</p>",
                escape_text(&post.description)
            )
        },
        tags(post),
        post.html
    )
}

/// Date, optional blog back-link, and optional author.
fn meta(blog_link: Option<&str>, post: &BlogPost) -> String {
    let blog_link = blog_link.map_or_else(String::new, |title| {
        format!("<a href=\"/blog/\">{}</a>", escape_text(title))
    });
    let author = post.author.as_deref().map_or_else(String::new, |author| {
        format!("<span class=\"post-author\">{}</span>", escape_text(author))
    });
    format!(
        "<p class=\"post-meta\">{blog_link}<time datetime=\"{date}\">{date}</time>{author}</p>",
        date = post.date.format("%Y-%m-%d"),
    )
}

fn tags(post: &BlogPost) -> String {
    if post.tags.is_empty() {
        return String::new();
    }
    let tags = post
        .tags
        .iter()
        .map(|tag| format!("<li>{}</li>", escape_text(tag)))
        .collect::<String>();
    format!("<ul class=\"post-tags\">{tags}</ul>")
}

/// RSS 2.0 feed with absolute links, or `None` without a canonical domain.
///
/// Feed URLs are only valid when the site publishes under a known domain, so
/// a site without `canonical_domain` skips the feed instead of emitting
/// relative links that feed readers cannot resolve.
pub(crate) fn render_feed(site: &ProjectSite, blog: &Blog) -> Option<String> {
    let domain = site.canonical_domain.as_deref()?.trim();
    if domain.is_empty() {
        return None;
    }
    let origin = format!("https://{}", domain.trim_matches('/'));

    let mut items = String::new();
    for post in &blog.posts {
        let url = format!("{origin}/blog/{}/", post.slug);
        items.push_str(&format!(
            "<item><title>{title}</title><link>{url}</link><guid isPermaLink=\"true\">{url}</guid><pubDate>{date}</pubDate><description>{description}</description></item>",
            title = escape_text(&post.title),
            url = escape_text(&url),
            date = post
                .date
                .and_hms_opt(0, 0, 0)
                .map(|midnight| midnight.and_utc().to_rfc2822())
                .unwrap_or_default(),
            description = escape_text(&post.description),
        ));
    }

    Some(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\"><channel><title>{title}</title><link>{link}</link><description>{description}</description><language>en</language>{items}</channel></rss>\n",
        title = escape_text(&blog.title),
        link = escape_text(&format!("{origin}/blog/")),
        description = escape_text(&site.description),
    ))
}
