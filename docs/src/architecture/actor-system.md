# Actor System

Plinth uses [Kameo](https://github.com/tqwewe/kameo) actors for concurrent, in-memory operations that would be expensive to run on every request.

## Content Caches

**Location**: `crates/server/src/actors/core_cache.rs`, `crates/server/src/bricks/*/cache.rs`

In-memory caches for blog posts, portfolio items, todos, tags, and site content. They avoid hitting Postgres on every page load.

**Messages**:
- `GetAllBlogPosts` — returns cached `Vec<BlogListItem>`
- `GetBlogPost(String)` — returns a full `BlogPost` by slug
- `GetPostsByTag(String)` — returns posts filtered by tag slug
- `GetAllPortfolioItems` — returns cached portfolio items
- `GetPortfolioItemBySlug(String)` — returns a single portfolio item
- `GetAllTodos` and `GetTodosByTag(String)` — return todo list items
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

The vector search path stores embeddings in Postgres via pgvector and uses the HNSW index for approximate nearest-neighbour lookup.

## Lifecycle

Actors are spawned during server startup in `main.rs` and stored in `AppState`. They receive a clone of the Postgres pool for lazy data loading. Cache invalidation is triggered automatically after admin API operations (publish, delete, tag changes).
