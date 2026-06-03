# Phase 07 — RSS/Atom feed + pgvector search integration

> **Recommended Codex model: GPT 5.5 medium**
>
> This phase is moderate. Two surfaces — an RSS feed and a semantic-search union — and
> both have an existing in-repo template to copy almost verbatim (`blog_feed` in
> `api/feeds.rs`; `search_by_embedding` in `actors/vector_search.rs`). The one genuinely
> novel piece is the `UNION ALL` that folds `activity_items` into the pgvector query while
> keeping the heterogeneous result type sound: a too-small model tends to force activity
> rows through the blog-specific row decoder (wrong columns → panic), drop the
> `min_similarity` filter, forget that the feed must read from the activity cache actor (not
> the DB directly), or mis-handle the `#[cfg(feature = "brick-activity")]` gating that the
> codebase requires on every new route, handler, and `AppState` access. A medium model with
> the inlined patterns below can execute this without rediscovering anything; high tier is
> unnecessary because there is no concurrency, refresh, or distributed-state reasoning here.

## Working tree

- `cwd = /data/nvme0/can/Projects/solo/plinth` (the plinth repo).
- **Phase 03 (server-brick-core) MUST land first.** This phase consumes the activity cache
  actor, the `bricks/activity/` module, the `activity_items` table + migration `0006_activity.sql`,
  and the `AppState.activity_cache` field — all created in Phase 03. If those do not exist, stop
  and confirm Phase 03 is merged.
- **Serialization with Phase 04 (lazy-refresh-actor):** Phase 04 and this phase **both** touch
  `crates/server/src/bricks/activity/{mod.rs,api.rs}`, `crates/server/src/main.rs` route
  registration, and `crates/server/src/lib.rs` (`AppState`). Whichever lands second **must
  `git pull --rebase` (or merge `main`) before starting** and re-resolve conflicts in those
  files. Concretely: keep both Phase 04's refresh-trigger calls in `api.rs` AND this phase's
  feed handler + search wiring; keep both phases' `main.rs` route blocks. Treat `main.rs`'s
  `feed_app`, `public_api_router`, and the `AppState { ... }` literal as merge hot spots.
- Search routes are registered in `main.rs` today under `#[cfg(feature = "brick-blog")]`. Adding
  activity to the same `/search` endpoint means the search code path must compile both with and
  without `brick-blog`. See the Pitfalls section (P4) — gate carefully.

## Goal

This phase succeeds when: (a) `GET /feeds/activity.xml` returns a valid `application/rss+xml`
document whose `<item>`s are the cached, ranked activity entries (served from the
`ActivityCache` actor, never blocking on a DB round-trip per request), and (b) `GET /api/search`
returns a combined result set in which a seeded, embedded `activity_items` row appears alongside
blog posts, ranked by cosine similarity and filtered by `min_similarity`. Both are
`#[cfg(feature = "brick-activity")]`-gated and covered by named integration tests that assert the
XML shape and the search-hit presence.

## Why this matters now

The activity brick's three other public surfaces (the `/activity` list/detail pages and the
home-page strip) ship in Phase 06; this phase adds the **last two discovery surfaces** so curated
contributions are reachable the way every other content type already is: a syndication feed
(mirroring `/feeds/blog.xml` and `/feeds/projects.xml`) and the site's existing semantic search.
Without the search union, an activity entry can only be found by browsing `/activity`; with it,
typing a topic into the site search surfaces relevant PRs/issues. Deferring this strands the
embeddings the CLI computes at add-time (Phase 05) — they would sit unused in the
`activity_items.embedding` column. Folding activity into the existing `/api/search` query is the
only thing that makes those embeddings pay off.

## Out of scope

- **Refresh actor internals (Phase 04).** Do not implement or modify the TTL / single-flight /
  stale-while-revalidate logic in `cache.rs` / `refresh.rs`. This phase only *reads* from the
  cache actor via Phase 03's existing `GetRankedActivity { limit, featured_only }` message and adds
  **no** new message to `cache.rs`. The activity search lives in the `VectorSearch` actor
  (`actors/vector_search.rs`), not in `cache.rs`.
- **CLI (Phase 05).** Embeddings are produced by `plinth activity add` in the CLI. Do **not** add
  embedding generation on the server, and do **not** re-embed during refresh (see Pitfalls P5).
- **Frontend pages / home strip (Phase 06).** No `crates/client` changes here. Adding the
  `<link rel="alternate">` feed-discovery tag to the SSR shell `<head>` is optional polish, not
  required for acceptance.
- **The forge crate (Phase 02) and shared types (Phase 01).** Reuse `ActivityListItem`,
  `ActivityItem`, `Forge`, `ActivityKind`, `ActivityState` as already defined; do not redefine them.
- **Ranking strategy implementation (Phase 03).** The feed reuses whatever ordering the cache
  actor already returns (ranked or recent); do not reimplement the ranking SQL here.

## Plan

All file paths are relative to `cwd = /data/nvme0/can/Projects/solo/plinth`.

### Step 1 — Add `activity_limit` to the `[feeds]` config section

File: `crates/shared/src/toml_config.rs`. The existing `FeedsConfig` (around line 311) is:

```rust
pub struct FeedsConfig {
    #[serde(default = "default_feed_limit")]
    pub blog_limit: usize,
    #[serde(default = "default_feed_limit")]
    pub projects_limit: usize,
    // ...
}
```

with `default_feed_limit() -> usize { 50 }` and a hand-written `impl Default`. Add a third field
mirroring the pattern exactly:

```rust
    #[serde(default = "default_feed_limit")]
    pub activity_limit: usize,
```

and in `impl Default for FeedsConfig` add `activity_limit: default_feed_limit(),`. No new
default-fn is needed (reuse `default_feed_limit`). This compiles regardless of features because
`FeedsConfig` is not feature-gated. Read it at runtime via `state.config.feeds.activity_limit`.

Add to the example `plinth.toml` under the existing `[feeds]` block:

```toml
[feeds]
blog_limit = 50
projects_limit = 50
activity_limit = 50
```

### Step 2 — Reuse the existing `GetRankedActivity` cache message (do NOT add a new one)

File: `crates/server/src/bricks/activity/cache.rs` (created in Phase 03). The feed needs an
ordered `Vec<ActivityListItem>`. Phase 03 already exposes exactly this through the canonical
`ActivityCache` actor message:

```rust
// Defined by Phase 03 in cache.rs — DO NOT redefine it here.
pub struct GetRankedActivity { pub limit: Option<i64>, pub featured_only: bool }

impl Message<GetRankedActivity> for ActivityCache {
    type Reply = Result<Vec<plinth_shared::ActivityListItem>, String>;
    // ... ranked read served from the actor's `ranked_list_cache` field (Phase 03/04 own this).
}
```

The feed handler asks for it with `GetRankedActivity { limit: Some(feeds.activity_limit),
featured_only: false }`. There is **no** `GetActivityFeedItems` / `GetAllActivityItems` message,
and the handler **never** touches the actor's private `ranked_list_cache` field directly — it only
sends the `GetRankedActivity` message. The actor owns the DB round-trip and caching; the feed is a
pure consumer.

> **Coordination note:** `cache.rs` is shared with Phase 04. This phase adds **no** message to it;
> `GetRankedActivity` already exists. Do not alter TTL/refresh fields. If Phase 04 lands first,
> nothing in this step needs reapplying — only confirm the message signature still matches.

### Step 3 — Add the `activity_feed` handler in `api/feeds.rs`

File: `crates/server/src/api/feeds.rs`. This file already has `xml_escape` (line 12),
`resolve_base_url` (line 23), `blog_feed` (line 36), and `projects_feed`. Mirror `blog_feed`
exactly. The `rss` crate is the syndication library in use (`ChannelBuilder`/`ItemBuilder`/
`GuidBuilder`/`CategoryBuilder`) — emit `application/rss+xml`. Append:

```rust
/// GET /feeds/activity.xml — RSS feed of curated external activity
#[cfg(feature = "brick-activity")]
pub async fn activity_feed(State(state): State<AppState>) -> Result<Response, StatusCode> {
    use crate::bricks::activity::cache::GetRankedActivity;
    use rss::{CategoryBuilder, ChannelBuilder, GuidBuilder, ItemBuilder};

    let base_url = resolve_base_url(&state);
    let site = &state.site_config;
    let feeds = &state.config.feeds;

    // Ask the actor — never read its private `ranked_list_cache` field, never query the DB here.
    let items_src = state
        .activity_cache
        .ask(GetRankedActivity {
            limit: Some(feeds.activity_limit as i64),
            featured_only: false,
        })
        .await
        .map_err(|e| {
            error!(error = %e, "activity feed query failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let items: Vec<rss::Item> = items_src
        .into_iter()
        .map(|item| {
            // Prefer the canonical upstream forge URL; fall back to the local detail page.
            let link = if item.url.is_empty() {
                format!("{}/activity/{}", base_url, item.id)
            } else {
                item.url.clone()
            };
            // ActivityListItem carries labels (Vec<String>) — use them as RSS categories.
            let categories: Vec<rss::Category> = item
                .labels
                .iter()
                .map(|label| CategoryBuilder::default().name(label.clone()).build())
                .collect();
            // reference date helper = merged_at.or(closed_at).unwrap_or(created_at)
            let pub_date = item.reference_date().to_rfc2822();
            ItemBuilder::default()
                .title(Some(item.title.clone()))
                .link(Some(link.clone()))
                // ActivityListItem has NO body field; the title is the description.
                .description(Some(item.title.clone()))
                .categories(categories)
                .guid(Some(GuidBuilder::default().value(link).permalink(true).build()))
                .pub_date(Some(pub_date))
                .build()
        })
        .collect();

    let mut builder = ChannelBuilder::default();
    builder
        .title(format!("{} - Activity", site.name))
        .link(format!("{}/activity", base_url))
        .description(if site.description.is_empty() {
            "Curated external contributions".to_string()
        } else {
            site.description.clone()
        })
        .language(Some(site.lang.clone()))
        .last_build_date(Some(chrono::Utc::now().to_rfc2822()))
        .items(items);

    if !site.author.email.is_empty() {
        builder.managing_editor(Some(format!("{} ({})", site.author.email, site.author.name)));
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
```

The canonical `ActivityListItem` (Phase 01) carries `id: i64`, `url`, `title`, `labels:
Vec<String>`, `created_at`, `closed_at`, `merged_at`, and the `reference_date()` helper — it does
**not** carry a `body`. Therefore the feed item's `description` is `Some(item.title.clone())`, its
categories are `item.labels`, and its `pub_date` is `item.reference_date().to_rfc2822()`. The
load-bearing requirements: pull from the **cache actor** via `GetRankedActivity { limit:
Some(feeds.activity_limit as i64), featured_only: false }` (the actor already applies the limit, so
no extra `.take(...)` is needed), set `Content-Type: application/rss+xml; charset=utf-8` +
`Cache-Control: public, max-age=3600`, and gate with `#[cfg(feature = "brick-activity")]`.

Optionally add a sitemap entry: in `sitemap_xml` (same file, ~line 255), mirror the
`#[cfg(feature = "brick-portfolio")]` block to push `/activity` static + per-item URLs. Not
required for acceptance.

### Step 4 — Register the feed route in `main.rs`

File: `crates/server/src/main.rs`. The `feed_app` router (~line 462) is built at root (NOT under
`/api`) with feeds feature-gated:

```rust
#[cfg(feature = "brick-blog")]
{
    feed_app = feed_app
        .route("/feeds/blog.xml", get(api::feeds::blog_feed))
        // ...
}
#[cfg(feature = "brick-portfolio")]
{
    feed_app = feed_app.route("/feeds/projects.xml", get(api::feeds::projects_feed));
}
```

Add immediately after the portfolio block:

```rust
#[cfg(feature = "brick-activity")]
{
    feed_app = feed_app.route("/feeds/activity.xml", get(api::feeds::activity_feed));
}
```

> **Merge hot spot:** Phase 04 also edits route registration in this file. After rebasing, ensure
> this `feed_app` block AND Phase 04's public/admin route blocks both survive.

### Step 5 — Add the search query for `activity_items`

File: `crates/server/src/actors/vector_search.rs`. The current single-source query lives in
`search_by_embedding` (line ~120):

```rust
SELECT id, slug, title, description, LEFT(content, 200) AS content, ''::text AS html_content,
       published_at, updated_at, author, tags, featured, published, reading_time_minutes,
       content_format, source, content_hash, series_slug, series_title, series_position,
       1 - (embedding <=> $1) AS similarity
FROM blog_posts
WHERE embedding IS NOT NULL AND published = true
ORDER BY embedding <=> $1
LIMIT $2
```

with `EMBEDDING_DIM = 384`, `.bind(Vector::from(embedding))`, `.bind(limit as i64)`, decoded by
`row_to_blog_post`. **Do NOT force activity rows through `row_to_blog_post`** — its columns are
blog-specific and will fail to decode an activity row. Use **Option B from the design brief
(separate query + Rust-side merge)** because it keeps the heterogeneous result type sound and
matches the brick-ownership model. Add a dedicated activity search query/message, then merge.

Add a new message + handler in `vector_search.rs` (or, if you prefer brick ownership, add a
`SearchActivity` message to `bricks/activity/cache.rs` — but the embedding model lives in the
`VectorSearch` actor, so the simplest correct path is to add the query *here* and bind the same
`Vector`). Add an activity-search method on `VectorSearch`:

```rust
/// Cosine-similarity search over embedded activity items.
#[cfg(feature = "brick-activity")]
async fn search_activity_by_embedding(
    &self,
    embedding: Vec<f32>,
    limit: usize,
    min_similarity: f32,
) -> Result<Vec<(plinth_shared::ActivityListItem, f32)>, String> {
    // Project EXACTLY the columns the canonical rows::activity_list_item decoder reads
    // (id, forge, repo_owner, repo_name, kind, number, url, title, state,
    //  created_at, closed_at, merged_at, impact, labels, featured) plus the cosine
    // similarity. The decoder reads forge/kind/state as TEXT and `s.parse()`s them, and
    // `id` as a plain i64 — so SELECT * also works as long as the score column is aliased.
    let rows = sqlx::query(
        r#"
        SELECT
            id, forge, repo_owner, repo_name, kind, number, url, title, state,
            created_at, closed_at, merged_at, impact, labels, featured,
            1 - (embedding <=> $1) AS similarity
        FROM activity_items
        WHERE embedding IS NOT NULL AND published = true
        ORDER BY embedding <=> $1
        LIMIT $2
        "#,
    )
    .bind(Vector::from(embedding))
    .bind(limit as i64)
    .fetch_all(&self.db)
    .await
    .map_err(|e| format!("Activity vector search query failed: {e}"))?;

    let mut hits: Vec<(plinth_shared::ActivityListItem, f32)> = Vec::new();
    for row in rows {
        let similarity = row.try_get::<f64, _>("similarity").map_err(|e| e.to_string())? as f32;
        if similarity < min_similarity {
            continue; // respect min_similarity
        }
        // Decode via the canonical Phase-03 decoder — NEVER row_to_blog_post.
        let item = crate::services::rows::activity_list_item(&row).map_err(|e| e.to_string())?;
        hits.push((item, similarity));
    }
    Ok(hits)
}
```

The `rows::activity_list_item` decoder is the canonical one added in Phase 03 (it builds the
`ActivityListItem` whose fields are `id: i64`, `forge`, `repo_owner`, `repo_name`, `kind`,
`number`, `url`, `title`, `state`, `created_at`, `closed_at`, `merged_at`, `impact`, `labels`,
`featured`, `score`). It reads `forge`/`kind`/`state` as TEXT via `try_get::<String,_>(col)?.parse()?`
and `id` as a plain `i64`. The `min_similarity` value comes from `SearchConfig.min_similarity`
(default `0.5`, `crates/shared/src/toml_config.rs`).

### Step 6 — Union activity into the `SearchSimilarArticles` reply and the `/api/search` handler

The cleanest way to surface activity in the **existing** `/api/search` response without breaking
the typed blog result is to generalize the handler's result. Two equivalent moves; pick one:

**6a (preferred, minimal blast radius): add a parallel activity field to the search response.**
File: `crates/server/src/api/search.rs`. Today (line ~64):

```rust
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub post: BlogListItem,
    pub similarity: f32,
}
```

Generalize to a tagged enum so both kinds serialize uniformly:

```rust
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SearchHit {
    Blog { post: BlogListItem, similarity: f32 },
    #[cfg(feature = "brick-activity")]
    Activity { item: plinth_shared::ActivityListItem, similarity: f32 },
}
```

In `search_articles` (line ~83), after fetching the existing blog `results`, also ask the vector
actor for activity hits, merge, sort by similarity descending, and truncate to `limit`:

```rust
let limit = params.limit.min(MAX_SEARCH_LIMIT);

let blog = vs.ask(SearchSimilarArticles { query: query.to_string(), limit }).await? ;
let mut hits: Vec<SearchHit> = blog
    .into_iter()
    .map(|(post, similarity)| SearchHit::Blog { post: BlogListItem::from(post), similarity })
    .collect();

#[cfg(feature = "brick-activity")]
{
    let min_similarity = state.config.search.min_similarity;
    let acts = vs
        .ask(crate::actors::vector_search::SearchActivity {
            query: query.to_string(),
            limit,
            min_similarity,
        })
        .await
        .map_err(|e| { error!("Activity search failed: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;
    hits.extend(acts.into_iter().map(|(item, similarity)| SearchHit::Activity { item, similarity }));
}

hits.sort_by(|a, b| sim_of(b).partial_cmp(&sim_of(a)).unwrap_or(std::cmp::Ordering::Equal));
hits.truncate(limit);
Ok(Json(hits))
```

where `sim_of` reads the `similarity` field from either variant. Define the
`SearchActivity { query, limit, min_similarity }` message + `impl Message<SearchActivity> for
VectorSearch` in `vector_search.rs` mirroring `SearchSimilarArticles` (line ~177): it calls
`self.generate_embedding(&query)` then `self.search_activity_by_embedding(embedding, limit,
min_similarity)`. Keep the whole call wrapped in the existing `VECTOR_SEARCH_TIMEOUT` if you reuse
the timeout pattern.

> **Wire-type note:** if Phase 06's client already consumes `SearchResult { post, similarity }`,
> coordinate. If you must not change the existing JSON shape, use **6b** instead.

**6b (alternative, additive only): keep `SearchResult` as-is, add a separate `activity` array.**
Change the handler return to `Json(SearchResponse { posts: Vec<SearchResult>, activity:
Vec<ActivitySearchResult> })`. Lower risk to existing blog consumers, but a new top-level shape.
Choose 6a unless an existing consumer pins the array-of-`SearchResult` shape.

### Step 7 — Verify feature wiring compiles in all combinations

`brick-activity` must be defined in `crates/server/Cargo.toml` (chaining to
`plinth-client/brick-activity` + `plinth-shared/brick-activity`) and added to `default` — this is
Phase 03's job, but **confirm it exists** before building. Then confirm the search path compiles
with and without `brick-blog` (see Pitfall P4): the `#[cfg(feature = "brick-activity")]` block in
`search_articles` must not depend on a `brick-blog`-only symbol, and the route registration in
`main.rs` must not assume both features are on simultaneously.

### Step 8 — Tests

Add `crates/server/tests/activity_feed_search.rs`, mirroring the structure of
`crates/server/tests/portfolio_publish.rs` (feature-gated module, `app_state(pool)` builder,
`test_app(state)` router, `oneshot` requests). Use `#[sqlx::test(migrations = "./migrations")]`
(path resolves relative to `crates/server`) so each test gets a fresh DB with `0006_activity.sql`
applied. Seed activity rows directly via `sqlx::query` (add a helper to
`crates/server/tests/common/mod.rs` if convenient). Two required named tests:

```rust
#[cfg(feature = "brick-activity")]
mod enabled {
    use super::*;

    // Insert a published activity row with a non-null 384-dim embedding.
    async fn seed_activity(pool: &sqlx::PgPool, title: &str, embedding: Vec<f32>) -> i64 { /* INSERT ... RETURNING id */ }

    #[sqlx::test(migrations = "./migrations")]
    async fn activity_feed_returns_valid_xml_with_entries(pool: sqlx::PgPool) {
        seed_activity(&pool, "Fix the parser", vec![0.0_f32; 384]).await;
        let state = app_state(pool).await;
        let app = test_app(state);
        let resp = app
            .oneshot(Request::builder().uri("/feeds/activity.xml").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp.headers().get(http::header::CONTENT_TYPE).unwrap().to_str().unwrap();
        assert!(ct.starts_with("application/rss+xml"));
        let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let xml = String::from_utf8(body.to_vec()).unwrap();
        // Valid RSS + contains the seeded entry.
        assert!(xml.contains("<rss"));
        assert!(xml.contains("<channel>"));
        assert!(xml.contains("Fix the parser"));
        // Parse-back proves well-formedness:
        let channel = rss::Channel::read_from(xml.as_bytes()).expect("valid RSS");
        assert!(channel.items().iter().any(|i| i.title() == Some("Fix the parser")));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn search_returns_seeded_activity_above_min_similarity(pool: sqlx::PgPool) {
        // Seed an activity row whose embedding == the query embedding so cosine sim = 1.0.
        // Compute the query embedding the same way the actor does, OR insert a known vector
        // and search for text whose embedding is that vector. Simplest deterministic option:
        // bypass the model by seeding the embedding equal to a fixed vector and asserting the
        // SearchActivity message (not the HTTP layer) returns it — see note below.
        // ...
        let state = app_state(pool).await; // app_state must construct VectorSearch (or skip if None)
        // Drive /api/search with a query; assert the response JSON includes a hit whose
        // title == the seeded activity title and similarity >= state.config.search.min_similarity.
    }
}

#[cfg(not(feature = "brick-activity"))]
mod disabled {
    #[test]
    fn brick_activity_disabled_compiles() {}
}
```

**Determinism for the search test:** the embedding model is non-trivial to run in CI (it downloads
a model). Two acceptable strategies — pick one and document it inline:

1. **Actor-level assertion (recommended, no model download dependency in the assertion path):**
   construct `VectorSearch` only if the model is available; seed the activity row's `embedding`
   column to a fixed 384-dim unit vector; embed the query through the actor; assert
   `SearchActivity` returns the seeded item with `similarity >= min_similarity`. If model init
   fails in the sandbox, gate with `if state.vector_search.is_none() { return; }` so the test is a
   no-op rather than a false failure (the feed test still gives hard coverage).
2. **Pure-SQL similarity assertion:** seed the activity row's embedding to vector `v`, then run the
   same `search_activity_by_embedding` SQL with bind `Vector::from(v)` directly against `pool`
   (no model), asserting the row comes back with `similarity == 1.0 >= min_similarity`. This proves
   the union SQL + `min_similarity` filter without any fastembed dependency and is the most robust
   CI option.

Prefer **strategy 2** for the named search test so it does not depend on model download in the Nix
sandbox (`flake.nix`'s `plinth-test` has no network). Name the SQL-level test
`search_returns_seeded_activity_above_min_similarity`.

Run locally:

```bash
cargo test -p plinth-server --test activity_feed_search --features brick-activity
cargo test --workspace --all-targets   # full gate (what nix flake check runs)
cargo clippy --all-targets -- --deny warnings
```

## Acceptance criteria

- [ ] `GET /feeds/activity.xml` returns HTTP `200` with header `Content-Type: application/rss+xml;
      charset=utf-8` and `Cache-Control: public, max-age=3600`.
- [ ] The feed route is registered in `crates/server/src/main.rs` inside a
      `#[cfg(feature = "brick-activity")]` block on `feed_app` (root-level, not under `/api`), and
      the handler `activity_feed` exists in `crates/server/src/api/feeds.rs` gated the same way.
- [ ] The feed handler reads items via
      `state.activity_cache.ask(GetRankedActivity { limit: Some(feeds.activity_limit as i64), featured_only: false })`
      (the cache actor), not a direct DB query and not a direct read of the actor's private
      `ranked_list_cache` field, and respects `state.config.feeds.activity_limit`.
- [ ] `FeedsConfig` in `crates/shared/src/toml_config.rs` has an `activity_limit: usize` field
      defaulting to `default_feed_limit()` (50); `cargo test -p plinth-shared` still passes the
      existing `test_parse_partial_toml` / `test_parse_empty_toml`.
- [ ] Named test **`enabled::activity_feed_returns_valid_xml_with_entries`** in
      `crates/server/tests/activity_feed_search.rs` passes: asserts status `200`,
      `application/rss+xml` content type, `rss::Channel::read_from` parses the body, and a seeded
      entry's title appears as a channel `<item>`.
- [ ] The pgvector search query in `crates/server/src/actors/vector_search.rs` includes a path that
      queries `activity_items` with `1 - (embedding <=> $1) AS similarity`,
      `WHERE embedding IS NOT NULL AND published = true`, `ORDER BY embedding <=> $1 LIMIT $2`, and
      filters out rows with `similarity < min_similarity`.
- [ ] `GET /api/search?q=...` returns a result set that can contain activity hits (the response
      type generalizes beyond a single blog `SearchResult` — e.g. a `SearchHit` enum or an added
      `activity` array), merged and sorted by similarity descending, truncated to `limit`.
- [ ] Named test **`enabled::search_returns_seeded_activity_above_min_similarity`** passes: seeds
      an `activity_items` row with a known 384-dim embedding, exercises the activity search SQL with
      a matching query vector, and asserts the seeded item is returned with
      `similarity >= state.config.search.min_similarity`.
- [ ] Activity rows are **never** decoded via `row_to_blog_post`; they use the canonical
      `rows::activity_list_item` decoder (Phase 03).
- [ ] `cargo clippy --all-targets -- --deny warnings` reports **0 warnings** for the workspace.
- [ ] `cargo test --workspace --all-targets` passes (both `enabled` and `disabled` modules
      compile; the `disabled` module's `brick_activity_disabled_compiles` test exists for the
      `--no-default-features`-style gating proof).

## Files likely touched

Server:
- `crates/server/src/api/feeds.rs` — add `activity_feed` handler (mirror `blog_feed`).
- `crates/server/src/main.rs` — register `/feeds/activity.xml` on `feed_app`; (merge hot spot
  with Phase 04).
- `crates/server/src/actors/vector_search.rs` — add `search_activity_by_embedding` +
  `SearchActivity` message/handler.
- `crates/server/src/api/search.rs` — generalize `SearchResult` → `SearchHit` (or add `activity`
  array) and union activity hits into `search_articles`.
- `crates/server/src/bricks/activity/cache.rs` — **no edit required**; reuse Phase 03's existing
  `GetRankedActivity { limit, featured_only }` message (merge hot spot with Phase 04 if anything
  there changes).

Shared / config:
- `crates/shared/src/toml_config.rs` — `FeedsConfig.activity_limit`.
- `plinth.toml` — example `[feeds] activity_limit = 50`.

Tests:
- `crates/server/tests/activity_feed_search.rs` — new feed + search integration tests.
- `crates/server/tests/common/mod.rs` — optional `insert_activity_item` helper.

## Pitfalls

- **P1 — Forcing activity rows through `row_to_blog_post`.** *Symptom:* runtime
  `sqlx::Error` "column not found" / decode failure on `/api/search`. *Cause:* `row_to_blog_post`
  (vector_search.rs ~line 82) reads ~19 blog-specific columns. *Recovery:* use the dedicated
  `rows::activity_list_item` decoder and a separate activity query; never reuse the blog decoder.
- **P2 — Feed handler hitting the DB directly.** *Symptom:* feed render adds a synchronous DB
  round-trip and bypasses the cache/TTL design. *Cause:* copying the *query* instead of the
  *cache `ask`* from `blog_feed`. *Recovery:* the feed MUST `state.activity_cache.ask(...)`; only
  the cache actor's message handler touches the DB.
- **P3 — Dropping `min_similarity`.** *Symptom:* `/api/search` returns low-relevance activity
  noise that the test rejects. *Cause:* the blog search SQL does NOT apply `min_similarity` (only
  opinion-evolution does), so it is easy to omit. *Recovery:* filter
  `similarity < state.config.search.min_similarity` in `search_activity_by_embedding`, as in Step 5.
- **P4 — Search compiles only when `brick-blog` is on.** *Symptom:* build fails with
  `--no-default-features --features brick-activity` because `/search` is wired under
  `#[cfg(feature = "brick-blog")]` and references blog-only symbols. *Cause:* the existing
  `/search` route block is `brick-blog`-gated in `main.rs`. *Recovery:* keep the activity search
  contribution inside its own `#[cfg(feature = "brick-activity")]` block; do not move the blog
  `/search` registration; ensure the `SearchHit::Activity` variant and `SearchActivity` message are
  themselves `#[cfg(feature = "brick-activity")]`-gated so the no-blog build still type-checks. If
  `/search` should be reachable when only activity is enabled, add a second
  `#[cfg(all(feature = "brick-activity", not(feature = "brick-blog")))]` route block — otherwise
  document that search requires `brick-blog` (the `VectorSearch` actor lives in the blog feature
  path today). Pick one and state it in the test.
- **P5 — Expecting refresh to (re)embed.** *Symptom:* activity items added *before* this search
  wiring shipped never appear in search; refreshing them does not fix it. *Cause:* embeddings are
  computed only by the CLI at add-time (Phase 05); the refresh actor (Phase 04) deliberately does
  NOT re-embed (title/body rarely change). *Recovery:* document this as a known operational note —
  **entries added before search wiring need a `plinth activity` re-add to get embeddings.** The
  `WHERE embedding IS NOT NULL` clause silently skips un-embedded rows, which is correct behavior,
  not a bug.
- **P6 — `rss` crate `pub_date` not RFC-2822.** *Symptom:* feed validators reject the date. *Cause:*
  passing an ISO-8601/`to_rfc3339()` string. *Recovery:* use `.to_rfc2822()` on the chrono
  `DateTime<Utc>` (the reference date = `coalesce(merged_at, closed_at, created_at)`), matching
  `blog_feed`.
- **P7 — Rebase collision in `main.rs` / `cache.rs` / `lib.rs` with Phase 04.** *Symptom:*
  whichever lands second drops the other's route/field. *Cause:* both phases edit the same
  registration blocks. *Recovery:* `git pull --rebase` before starting; after rebase, grep for
  both phases' markers (`/feeds/activity.xml`, the refresh trigger, `activity_cache`) and confirm
  both survive; rebuild with `--features brick-activity`.
- **P8 — Embedding-model download in the CI sandbox.** *Symptom:* the search test hangs or fails
  in `nix flake check` (no network). *Cause:* constructing `VectorSearch` downloads the fastembed
  model. *Recovery:* use the SQL-level search test (Step 8, strategy 2) that binds a known
  `Vector` directly and never instantiates the model; gate any model-dependent assertion behind a
  `vector_search.is_none()` early return.

## Reference

- Design brief: this phase implements surfaces **4c (RSS/Atom feed at `/feeds/activity.xml`)** and
  **4d (search union over `activity_items`)** of the Forge Activity feature. Schema columns,
  endpoint paths, and config keys are inlined above so this file is standalone.
- **Phase 03 (server-brick-core) must land first** — it creates `bricks/activity/`,
  `0006_activity.sql`, `AppState.activity_cache`, the `rows::activity_list_item` decoder, and the
  `brick-activity` Cargo feature wiring. See `./03-server-brick-core.md` for sequencing only; do
  not rely on it for content here.
- **Phase 04 (lazy-refresh-actor)** shares `bricks/activity/{mod.rs,api.rs,cache.rs}`, `main.rs`
  route registration, and `lib.rs` (`AppState`) with this phase — rebase before starting. See
  `./04-lazy-refresh-actor.md` for sequencing only.
- **Phase 05 (cli-commands)** produces the embeddings this phase searches over; the search test
  seeds embeddings directly rather than depending on Phase 05. See `./05-cli-commands.md`.
- In-repo templates to copy: `blog_feed` / `projects_feed` in
  `crates/server/src/api/feeds.rs`; `search_by_embedding` + `SearchSimilarArticles` in
  `crates/server/src/actors/vector_search.rs`; `SearchResult` + `search_articles` in
  `crates/server/src/api/search.rs`; the integration-test harness in
  `crates/server/tests/portfolio_publish.rs`.
