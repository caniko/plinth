use leptos::*;
use crate::AppState;
use crate::actors::content_cache::{GetAllBlogPosts, GetBlogPost, GetPostsByTag};
use shared::{BlogPost, BlogListItem};

/// Server function to get all blog posts (as list items)
#[server(GetBlogPosts, "/api")]
pub async fn get_blog_posts() -> Result<Vec<BlogListItem>, ServerFnError> {
    let app_state = expect_context::<AppState>();

    let result = app_state
        .content_cache
        .ask(GetAllBlogPosts)
        .send()
        .await
        .map_err(|e| ServerFnError::ServerError(format!("Actor error: {}", e)))?;

    result.map_err(|e| ServerFnError::ServerError(e))
}

/// Server function to get a single blog post by slug
#[server(GetBlogPostBySlug, "/api")]
pub async fn get_blog_post_by_slug(slug: String) -> Result<Option<BlogPost>, ServerFnError> {
    let app_state = expect_context::<AppState>();

    let result = app_state
        .content_cache
        .ask(GetBlogPost(slug))
        .send()
        .await
        .map_err(|e| ServerFnError::ServerError(format!("Actor error: {}", e)))?;

    result.map_err(|e| ServerFnError::ServerError(e))
}

/// Server function to get blog posts by tag
#[server(GetBlogPostsByTag, "/api")]
pub async fn get_blog_posts_by_tag(tag: String) -> Result<Vec<BlogListItem>, ServerFnError> {
    let app_state = expect_context::<AppState>();

    let result = app_state
        .content_cache
        .ask(GetPostsByTag(tag))
        .send()
        .await
        .map_err(|e| ServerFnError::ServerError(format!("Actor error: {}", e)))?;

    result.map_err(|e| ServerFnError::ServerError(e))
}
