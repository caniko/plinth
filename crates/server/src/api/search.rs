use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    actors::vector_search::{FindRelatedArticles, SearchSimilarArticles, TrackOpinionEvolution},
    AppState,
};
use shared::BlogListItem;

/// Query parameters for semantic search
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// Search query text
    q: String,

    /// Maximum number of results (default: 10)
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    10
}

/// Query parameters for related articles
#[derive(Debug, Deserialize)]
pub struct RelatedQuery {
    /// Maximum number of results (default: 5)
    #[serde(default = "default_related_limit")]
    limit: usize,
}

fn default_related_limit() -> usize {
    5
}

/// Query parameters for opinion evolution tracking
#[derive(Debug, Deserialize)]
pub struct OpinionQuery {
    /// Topic to track
    topic: String,

    /// Minimum similarity threshold (default: 0.5)
    #[serde(default = "default_min_similarity")]
    min_similarity: f32,
}

fn default_min_similarity() -> f32 {
    0.5
}

/// Search result with similarity score
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub post: BlogListItem,
    pub similarity: f32,
}

/// Semantic search endpoint
pub async fn search_articles(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, StatusCode> {
    let results = state
        .vector_search
        .ask(SearchSimilarArticles {
            query: params.q,
            limit: params.limit,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert to search results
    let search_results: Vec<SearchResult> = results
        .into_iter()
        .map(|(post, similarity)| SearchResult {
            post: BlogListItem {
                id: post.id,
                slug: post.slug,
                title: post.title,
                description: post.content.chars().take(200).collect::<String>() + "...",
                published_at: post.published_at,
                author: post.author,
                tags: post.tags,
                featured: post.featured,
                reading_time_minutes: post.reading_time_minutes,
            },
            similarity,
        })
        .collect();

    Ok(Json(search_results))
}

/// Related articles endpoint
pub async fn related_articles(
    State(state): State<AppState>,
    axum::extract::Path(slug): axum::extract::Path<String>,
    Query(params): Query<RelatedQuery>,
) -> Result<Json<Vec<SearchResult>>, StatusCode> {
    let results = state
        .vector_search
        .ask(FindRelatedArticles {
            slug,
            limit: params.limit,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert to search results
    let search_results: Vec<SearchResult> = results
        .into_iter()
        .map(|(post, similarity)| SearchResult {
            post: BlogListItem {
                id: post.id,
                slug: post.slug,
                title: post.title,
                description: post.content.chars().take(200).collect::<String>() + "...",
                published_at: post.published_at,
                author: post.author,
                tags: post.tags,
                featured: post.featured,
                reading_time_minutes: post.reading_time_minutes,
            },
            similarity,
        })
        .collect();

    Ok(Json(search_results))
}

/// Opinion evolution tracking endpoint
pub async fn track_opinion(
    State(state): State<AppState>,
    Query(params): Query<OpinionQuery>,
) -> Result<Json<Vec<SearchResult>>, StatusCode> {
    let results = state
        .vector_search
        .ask(TrackOpinionEvolution {
            topic: params.topic,
            min_similarity: params.min_similarity,
        })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert to search results (already sorted by date)
    let search_results: Vec<SearchResult> = results
        .into_iter()
        .map(|(post, similarity)| SearchResult {
            post: BlogListItem {
                id: post.id,
                slug: post.slug,
                title: post.title,
                description: post.content.chars().take(200).collect::<String>() + "...",
                published_at: post.published_at,
                author: post.author,
                tags: post.tags,
                featured: post.featured,
                reading_time_minutes: post.reading_time_minutes,
            },
            similarity,
        })
        .collect();

    Ok(Json(search_results))
}
