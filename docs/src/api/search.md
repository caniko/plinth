# Search API

The search API is publicly accessible (no authentication required). It uses fastembed vector embeddings for semantic similarity.

## Semantic search

```
GET /api/search?q=<query>&limit=<n>
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `q` | string | required | Search query text |
| `limit` | integer | `10` | Maximum results |

The query text is embedded into a 384-dimensional vector and matched against article embeddings in Postgres using pgvector's HNSW approximate nearest-neighbour index with cosine distance.

**Response** (200):

```json
[
  {
    "post": {
      "id": "blog_posts:abc",
      "slug": "my-article",
      "title": "My Article",
      "description": "First 200 characters of content...",
      "published_at": "2025-01-15T10:00:00Z",
      "author": "Jane Doe",
      "tags": ["rust"],
      "featured": false,
      "reading_time_minutes": 5
    },
    "similarity": 0.87
  }
]
```

Results are sorted by similarity (highest first). Because HNSW is approximate, very close neighbours may not be returned in perfect exhaustive order.

## Related articles

```
GET /api/articles/{slug}/related?limit=<n>
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `slug` | path | required | Source article slug |
| `limit` | integer | `5` | Maximum results |

Finds articles whose embeddings are most similar to the given article's embedding. Useful for "related posts" sections.

**Response**: same format as semantic search.

## Opinion evolution

```
GET /api/opinion?topic=<topic>&min_similarity=<threshold>
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `topic` | string | required | Topic to track |
| `min_similarity` | float | `0.5` | Minimum similarity threshold |

Returns articles related to the topic sorted **chronologically** (oldest first), allowing you to see how your writing about a topic has evolved over time. Only articles above the similarity threshold are included.

**Response**: same format as semantic search, but sorted by `published_at` date.
