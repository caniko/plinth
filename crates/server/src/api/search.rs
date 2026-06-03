use std::time::Duration;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tracing::error;

const VECTOR_SEARCH_TIMEOUT: Duration = Duration::from_secs(10);

use crate::{
    AppState,
    actors::vector_search::{FindRelatedArticles, SearchSimilarArticles, TrackOpinionEvolution},
};
use plinth_shared::BlogListItem;

const MAX_SEARCH_LIMIT: usize = 50;
const MAX_RELATED_LIMIT: usize = 20;
const MAX_QUERY_LENGTH: usize = 500;

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

/// Heterogeneous search hit returned by the main semantic search endpoint.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SearchHit {
    Blog {
        post: BlogListItem,
        similarity: f32,
    },
    #[cfg(feature = "brick-activity")]
    Activity {
        item: plinth_shared::ActivityListItem,
        similarity: f32,
    },
}

impl SearchHit {
    fn similarity(&self) -> f32 {
        match self {
            SearchHit::Blog { similarity, .. } => *similarity,
            #[cfg(feature = "brick-activity")]
            SearchHit::Activity { similarity, .. } => *similarity,
        }
    }
}

/// Convert a list of (BlogPost, similarity) tuples into SearchResult vec.
fn to_search_results(results: Vec<(plinth_shared::BlogPost, f32)>) -> Vec<SearchResult> {
    results
        .into_iter()
        .map(|(post, similarity)| SearchResult {
            post: BlogListItem::from(post),
            similarity,
        })
        .collect()
}

/// Semantic search endpoint
pub async fn search_articles(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<SearchHit>>, StatusCode> {
    let query = params.q.trim();
    if query.is_empty() || query.len() > MAX_QUERY_LENGTH {
        return Err(StatusCode::BAD_REQUEST);
    }

    let vs = state
        .vector_search
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let limit = params.limit.min(MAX_SEARCH_LIMIT);

    let results = tokio::time::timeout(
        VECTOR_SEARCH_TIMEOUT,
        vs.ask(SearchSimilarArticles {
            query: query.to_string(),
            limit,
        }),
    )
    .await
    .map_err(|_| {
        error!("Search query timed out");
        StatusCode::GATEWAY_TIMEOUT
    })?
    .map_err(|e| {
        error!("Search query failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut hits: Vec<SearchHit> = results
        .into_iter()
        .map(|(post, similarity)| SearchHit::Blog {
            post: BlogListItem::from(post),
            similarity,
        })
        .collect();

    #[cfg(feature = "brick-activity")]
    {
        use crate::actors::vector_search::SearchActivity;

        let activity_hits = tokio::time::timeout(
            VECTOR_SEARCH_TIMEOUT,
            vs.ask(SearchActivity {
                query: query.to_string(),
                limit,
                min_similarity: state.config.search.min_similarity,
            }),
        )
        .await
        .map_err(|_| {
            error!("Activity search query timed out");
            StatusCode::GATEWAY_TIMEOUT
        })?
        .map_err(|e| {
            error!("Activity search query failed: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        hits.extend(
            activity_hits
                .into_iter()
                .map(|(item, similarity)| SearchHit::Activity { item, similarity }),
        );
    }

    hits.sort_by(|a, b| {
        b.similarity()
            .partial_cmp(&a.similarity())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    hits.truncate(limit);

    Ok(Json(hits))
}

/// Related articles endpoint
pub async fn related_articles(
    State(state): State<AppState>,
    axum::extract::Path(slug): axum::extract::Path<String>,
    Query(params): Query<RelatedQuery>,
) -> Result<Json<Vec<SearchResult>>, StatusCode> {
    let vs = state
        .vector_search
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let limit = params.limit.min(MAX_RELATED_LIMIT);

    let results = tokio::time::timeout(
        VECTOR_SEARCH_TIMEOUT,
        vs.ask(FindRelatedArticles { slug, limit }),
    )
    .await
    .map_err(|_| {
        error!("Related articles query timed out");
        StatusCode::GATEWAY_TIMEOUT
    })?
    .map_err(|e| {
        error!("Related articles query failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(to_search_results(results)))
}

/// Opinion evolution tracking endpoint
pub async fn track_opinion(
    State(state): State<AppState>,
    Query(params): Query<OpinionQuery>,
) -> Result<Json<Vec<SearchResult>>, StatusCode> {
    let topic = params.topic.trim();
    if topic.is_empty() || topic.len() > MAX_QUERY_LENGTH {
        return Err(StatusCode::BAD_REQUEST);
    }

    let vs = state
        .vector_search
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let results = tokio::time::timeout(
        VECTOR_SEARCH_TIMEOUT,
        vs.ask(TrackOpinionEvolution {
            topic: topic.to_string(),
            min_similarity: params.min_similarity,
        }),
    )
    .await
    .map_err(|_| {
        error!("Opinion tracking query timed out");
        StatusCode::GATEWAY_TIMEOUT
    })?
    .map_err(|e| {
        error!("Opinion tracking query failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(to_search_results(results)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_limit_clamped() {
        assert_eq!(999_999_usize.min(MAX_SEARCH_LIMIT), MAX_SEARCH_LIMIT);
        assert_eq!(10_usize.min(MAX_SEARCH_LIMIT), 10);
    }

    #[test]
    fn test_related_limit_clamped() {
        assert_eq!(100_usize.min(MAX_RELATED_LIMIT), MAX_RELATED_LIMIT);
        assert_eq!(5_usize.min(MAX_RELATED_LIMIT), 5);
    }

    #[test]
    fn test_default_limits() {
        assert_eq!(default_limit(), 10);
        assert_eq!(default_related_limit(), 5);
        assert_eq!(default_min_similarity(), 0.5);
    }

    #[test]
    fn test_search_query_deserialize_defaults() {
        let q: SearchQuery = serde_json::from_str(r#"{"q": "hello"}"#).unwrap();
        assert_eq!(q.q, "hello");
        assert_eq!(q.limit, 10); // default_limit()
    }

    #[test]
    fn test_search_query_explicit_limit() {
        let q: SearchQuery = serde_json::from_str(r#"{"q": "test", "limit": 25}"#).unwrap();
        assert_eq!(q.limit, 25);
    }

    #[test]
    fn test_related_query_deserialize_defaults() {
        let q: RelatedQuery = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(q.limit, 5); // default_related_limit()
    }

    #[test]
    fn test_opinion_query_deserialize() {
        let q: OpinionQuery = serde_json::from_str(r#"{"topic": "rust ownership"}"#).unwrap();
        assert_eq!(q.topic, "rust ownership");
        assert_eq!(q.min_similarity, 0.5); // default
    }

    #[test]
    fn test_opinion_query_custom_similarity() {
        let q: OpinionQuery =
            serde_json::from_str(r#"{"topic": "ai", "min_similarity": 0.8}"#).unwrap();
        assert_eq!(q.min_similarity, 0.8);
    }

    #[test]
    fn test_max_query_length_constant() {
        assert_eq!(MAX_QUERY_LENGTH, 500);
    }

    #[test]
    fn test_query_boundary_validation_logic() {
        // Mirrors the handler's validation: empty or over MAX_QUERY_LENGTH → reject
        let empty = "";
        assert!(empty.trim().is_empty() || empty.len() > MAX_QUERY_LENGTH);

        let whitespace = "   ";
        assert!(whitespace.trim().is_empty());

        let too_long = "a".repeat(MAX_QUERY_LENGTH + 1);
        assert!(too_long.len() > MAX_QUERY_LENGTH);

        let exactly_max = "a".repeat(MAX_QUERY_LENGTH);
        assert!(!exactly_max.trim().is_empty() && exactly_max.len() <= MAX_QUERY_LENGTH);
    }

    #[test]
    fn test_vector_search_timeout_value() {
        assert_eq!(VECTOR_SEARCH_TIMEOUT, Duration::from_secs(10));
    }
}
