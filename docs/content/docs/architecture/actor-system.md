+++
title = "Actor System"
description = "Kameo actors for caching and vector search"
weight = 20
+++

Plinth uses [Kameo](https://github.com/tqwewe/kameo) actors for concurrent, in-memory operations that would be expensive to run on every request.

## ContentCache

**Location**: `crates/server/src/actors/content_cache.rs`

An in-memory cache of blog posts, portfolio items, tags, and site content. Avoids hitting SurrealDB on every page load.

**Messages**:
- `GetAllPosts` — returns cached `Vec<BlogListItem>`
- `GetPostBySlug(String)` — returns a full `BlogPost` by slug
- `GetPostsByTag(String)` — returns posts filtered by tag slug
- `GetFeaturedPosts` — returns featured posts
- `GetAllPortfolioItems` — returns cached portfolio items
- `GetPortfolioItemBySlug(String)` — returns a single portfolio item
- `GetAllTags` — returns all tags with post counts
- `GetSiteContent(String)` — returns site content by key
- `InvalidateCache` — clears the cache, forcing a reload from DB on next access

The cache is lazily populated: the first request after invalidation triggers a DB query, and subsequent requests are served from memory.

## VectorSearch

**Location**: `crates/server/src/actors/vector_search.rs`

Handles semantic search using fastembed 384-dimensional embeddings and cosine similarity.

**Messages**:
- `SearchSimilarArticles { query, limit }` — embeds the query text and finds the most similar articles
- `FindRelatedArticles { slug, limit }` — finds articles related to a given post
- `TrackOpinionEvolution { topic, min_similarity }` — finds posts about a topic sorted chronologically (for tracking how opinions evolve over time)

The actor loads all post embeddings into memory from SurrealDB and performs cosine similarity comparisons in-process. This avoids the need for an external vector database.

## Lifecycle

Both actors are spawned during server startup in `main.rs` and stored in `AppState`. They receive a clone of the SurrealDB handle for lazy data loading. Cache invalidation is triggered automatically after admin API operations (publish, delete, tag changes).
