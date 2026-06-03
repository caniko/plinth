//! Blog public API handlers.

use axum::{
    Json,
    extract::{Path, State},
};
use plinth_shared::{BlogListItem, BlogPost, SeriesListItem, SeriesNav};

use super::cache::{
    GetAllBlogPosts, GetAllSeries, GetBlogPost, GetPostsByTag, GetSeriesNav, GetSeriesPosts,
};
use crate::{AppState, error::PlinthError};

/// GET /api/posts
pub async fn list_blog_posts(
    State(state): State<AppState>,
) -> Result<Json<Vec<BlogListItem>>, PlinthError> {
    let posts = state
        .blog_cache
        .ask(GetAllBlogPosts)
        .await
        .map_err(PlinthError::actor)?;

    Ok(Json(posts))
}

/// GET /api/posts/{slug}
pub async fn get_blog_post(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Option<BlogPost>>, PlinthError> {
    let post = state
        .blog_cache
        .ask(GetBlogPost(slug))
        .await
        .map_err(PlinthError::actor)?;

    Ok(Json(post))
}

/// GET /api/posts/tag/{tag}
pub async fn list_blog_posts_by_tag(
    State(state): State<AppState>,
    Path(tag): Path<String>,
) -> Result<Json<Vec<BlogListItem>>, PlinthError> {
    let posts = state
        .blog_cache
        .ask(GetPostsByTag(tag))
        .await
        .map_err(PlinthError::actor)?;

    Ok(Json(posts))
}

/// GET /api/posts/{slug}/series-nav
pub async fn get_series_nav(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Option<SeriesNav>>, PlinthError> {
    let nav = state
        .blog_cache
        .ask(GetSeriesNav(slug))
        .await
        .map_err(PlinthError::actor)?;

    Ok(Json(nav))
}

/// GET /api/series/{slug}/posts
pub async fn list_series_posts(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Vec<BlogListItem>>, PlinthError> {
    let posts = state
        .blog_cache
        .ask(GetSeriesPosts(slug))
        .await
        .map_err(PlinthError::actor)?;

    Ok(Json(posts))
}

/// GET /api/series
pub async fn list_series(
    State(state): State<AppState>,
) -> Result<Json<Vec<SeriesListItem>>, PlinthError> {
    let series = state
        .blog_cache
        .ask(GetAllSeries)
        .await
        .map_err(PlinthError::actor)?;

    Ok(Json(series))
}
