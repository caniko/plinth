# Phase 08 / Sub-02 — End-to-end integration tests for the activity brick

> **Recommended Codex model: GPT 5.5 medium**
>
> The tests are concrete but span the whole feature: a real `AppState` with spawned Kameo actors, a mocked forge via `wiremock`, admin Bearer auth, the ranking order, the RSS feed XML, and the search union — exercised through axum's `oneshot` and `#[sqlx::test]` per-test databases. A low tier risks getting the `app_state` actor wiring wrong (it must mirror `portfolio_publish.rs` field-for-field, including the cfg-gated caches) or asserting on the wrong cosine-similarity column, producing flaky or false-green tests. Medium reliably reproduces the established test skeleton and writes correct DB/actor assertions. High is unnecessary — there is no novel design, only faithful pattern reuse.

## Working tree

`cwd = /data/nvme0/can/Projects/solo/plinth` (the plinth repo).

This sub-layer depends on Phases 01–07 having landed (the `activity_items` table/migration, the admin/public/feed routes, the cache + refresh actors, and the search union all exist). It also depends on **sub-01 of this phase** having added `wiremock` to `crates/forge`'s dev-deps **if** any forge-crate test is added — for the server-side test below, `wiremock` must be a dev-dep of `crates/server` (add it there too; see step 1).

It touches only test files: `crates/server/tests/forge_activity.rs` (new), `crates/server/tests/common/mod.rs` (extend), and optionally `crates/forge/tests/` (new). It edits no production code. No sibling sub-layer touches these files.

## Goal

This sub-layer succeeds when a new integration test file `crates/server/tests/forge_activity.rs` proves, against a sandbox Postgres, the full activity path end-to-end with a **mocked forge** (the PRIMARY mechanism is an injected `Arc<dyn ForgeClient>` counting fake; a wiremock-backed `ForgeRouter` is the secondary option): an admin POST upserts an activity row (Bearer-auth required), the public ranked endpoint returns it in score order, a stale entry triggers a single-flighted background refresh that re-pulls mocked forge metadata and updates the DB, the `/feeds/activity.xml` endpoint returns valid RSS containing the item, and the semantic-search endpoint's union surfaces the activity item — each as an independently named `#[sqlx::test]`, all passing under `cargo test -p plinth-server --test forge_activity` and therefore under `nix flake check`'s `plinth-test`.

## Why this matters now

Phases 02–07 are individually unit-tested at best, but no single test drives the whole chain. The lazy stale-while-revalidate refresh actor (Phase 04) is the highest-risk component — single-flighting, TTL expiry, "serve stale on failure", and "do not block render" are timing-sensitive behaviours that only an integration test with a controllable mocked forge can verify. The ranking SQL (Phase 03) computes score at read time and is easy to get subtly wrong (wrong `ORDER BY`, wrong age denominator); only an end-to-end assertion on returned order proves it. The search union (Phase 07) changes a previously single-source query; without a test, an activity item silently failing to appear in results is invisible. Deferring this leaves the feature shippable-looking but unverified.

## Out of scope

- Changing any production code. If a test reveals a bug, file it against the owning phase — do not fix it here (a test-only sub-layer must stay test-only to keep the merge disjoint).
- Re-testing the forge client's HTTP normalization at the unit level — that is Phase 02's inline `#[cfg(test)]` tests. This sub-layer tests integration (the server consuming the forge), using a mock, not the live forges.
- Frontend (Leptos) component tests — out of scope for the whole feature.
- Re-embedding behaviour beyond asserting the documented pitfall (refresh does NOT re-embed) holds: assert the embedding column is unchanged after a refresh.

## Plan

1. **Add server-side dev-dependencies.** Append to `/data/nvme0/can/Projects/solo/plinth/crates/server/Cargo.toml` `[dev-dependencies]` (the section already exists with `tower`):

   ```toml
   wiremock = "0.6"      # only for the SECONDARY (real ForgeRouter over loopback) refresh mode
   ```
   `tower` (with `util`) and `tokio` are already dev-deps; `kameo`, `axum`, `sqlx`, `serde_json` are available via the crate. `async-trait` is already a dep (used to impl `ForgeClient` on the `MockForge` fake). `wiremock` is transport-level, compatible with the locked `reqwest 0.12.28`. The PRIMARY refresh-test mechanism is the injected `Arc<dyn ForgeClient>` `MockForge` (no extra dep); `wiremock` is only required when you opt into the secondary mode.

2. **Extend `crates/server/tests/common/mod.rs`** with an activity row-insert helper, mirroring `insert_blog_post` (which is at the top of that file and inserts via raw `sqlx::query_scalar` with `.bind(...)`). Add (keep the file's `#![allow(dead_code)]`):

   ```rust
   #[cfg(feature = "brick-activity")]
   #[allow(clippy::too_many_arguments)]
   pub async fn insert_activity(
       pool: &PgPool,
       forge: &str,        // "github" | "codeberg"
       owner: &str,
       repo: &str,
       kind: &str,         // "pr" | "issue"
       number: i32,
       impact: i16,        // 1..=10
       reference_date: chrono::DateTime<chrono::Utc>, // becomes merged_at/created_at
       fetched_at: chrono::DateTime<chrono::Utc>,     // drives the TTL
   ) -> Result<i64, sqlx::Error> {
       let url = format!("https://{forge}.example/{owner}/{repo}/{kind}/{number}");
       sqlx::query_scalar(
           r#"
           INSERT INTO activity_items (
               forge, repo_owner, repo_name, kind, number, url, title, body,
               state, created_at, merged_at, impact, fetched_at, published
           )
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'merged',$9,$9,$10,$11,true)
           RETURNING id
           "#,
       )
       .bind(forge)
       .bind(owner)
       .bind(repo)
       .bind(kind)
       .bind(number)
       .bind(&url)
       .bind(format!("{kind} #{number} on {owner}/{repo}"))
       .bind(format!("Body for {owner}/{repo}#{number}"))
       .bind(reference_date)   // created_at
       .bind(impact)
       .bind(fetched_at)
       .fetch_one(pool)
       .await
   }
   ```

   (Adjust column names to the exact migration `0006_activity.sql` schema: `forge, repo_owner, repo_name, kind, number, url UNIQUE, title, body, state, created_at, closed_at, merged_at, impact SMALLINT, additions, deletions, comments_count, labels TEXT[], repo_stars, embedding vector(384), fetched_at, featured, published, content_hash`.)

3. **Create `crates/server/tests/forge_activity.rs`** using the `portfolio_publish.rs` skeleton verbatim where possible. The skeleton's load-bearing pieces, inlined:

   - **Module gate (top + bottom):**
     ```rust
     #[cfg(feature = "brick-activity")]
     mod enabled {
         // ... all tests ...
     }

     #[cfg(not(feature = "brick-activity"))]
     mod disabled {
         use axum::{Router, body::Body, http::{Request, StatusCode}};
         use tower::ServiceExt;
         #[tokio::test]
         async fn activity_admin_route_absent_without_feature() {
             let app: Router = Router::new();
             let response = app
                 .oneshot(Request::builder().method("POST").uri("/api/admin/activity")
                     .body(Body::empty()).unwrap())
                 .await.unwrap();
             assert_eq!(response.status(), StatusCode::NOT_FOUND);
         }
     }
     ```

   - **`fn app_state(pool: PgPool, forge_client: Arc<dyn ForgeClient + Send + Sync>) -> AppState`** — copy `portfolio_publish.rs`'s `app_state` field-for-field (it constructs the real `plinth_server::AppState`: `leptos_options`, `core_cache: CoreCache::spawn(...)`, `db: pool.clone()`, `immich_config: None`, `http_client: reqwest::Client::builder().build().expect(...)`, `config`, `site_config`, and the cfg-gated `blog_cache`/`vector_search`/`portfolio_cache`/`todo_cache`). **Add the activity cache field** using the canonical Phase-04 four-arg constructor `ActivityCache::new(db, ranking, forge, forge_client)`:
     ```rust
     #[cfg(feature = "brick-activity")]
     activity_cache: kameo::spawn(
         plinth_server::bricks::activity::cache::ActivityCache::new(
             pool.clone(),
             config.ranking.clone(),   // RankingConfig (strategy=Exponential, half_life_days=365, window_days=730)
             config.forge.clone(),     // ForgeConfig (refresh_ttl_secs=3600, refresh_backoff_secs=900, base urls)
             forge_client,             // injected Arc<dyn ForgeClient + Send + Sync>
         ),
     ),
     ```
     **The PRIMARY injection point is the mocked `forge_client`.** Tests pass an `Arc<dyn ForgeClient + Send + Sync>` — a counting fake implementing the canonical trait method `async fn fetch(&self, r: &ActivityRef) -> ForgeResult<FetchedActivity>` (see the `MockForge` helper below) — so the refresh path never touches the network and is fully deterministic. Use the wiremock `MockServer` only when you want a real `ForgeRouter` built via `with_base_url` to exercise HTTP normalization; in that secondary mode set `config.forge.github_base_url = server.uri()` / `config.forge.codeberg_base_url = server.uri()` and pass `Arc::new(ForgeRouter { github: GitHubClient::with_base_url(server.uri(), None), codeberg: CodebergClient::with_base_url(server.uri(), None) })` as the `forge_client`. Do NOT rely on `config.forge.github_base_url` as the only mechanism, and do NOT use any one-arg `ActivityCache::new`.

   - **`struct MockForge` (the PRIMARY refresh driver)** — a counting fake `Arc<dyn ForgeClient>`:
     ```rust
     use std::sync::Arc;
     use std::sync::atomic::{AtomicUsize, Ordering};
     use plinth_forge::{ActivityRef, FetchedActivity, ForgeClient, ForgeError, ForgeResult};

     struct MockForge {
         calls: AtomicUsize,
         response: ForgeResult<FetchedActivity>,   // clone-per-call; or build fresh in fetch()
     }
     #[async_trait::async_trait]
     impl ForgeClient for MockForge {
         async fn fetch(&self, r: &ActivityRef) -> ForgeResult<FetchedActivity> {
             self.calls.fetch_add(1, Ordering::SeqCst);
             // return canned FetchedActivity for r, or Err(ForgeError::Http { .. }) / NotFound { .. }
             self.response.clone()
         }
     }
     ```
     `MockForge.calls` replaces `wiremock`'s `received_requests()` for the single-flight assertion. Inject it via `Arc::new(MockForge { .. })`. For the failure test, have `fetch` return `Err(ForgeError::Http { forge: Forge::GitHub, status: 500, body: "..".into() })` or `Err(ForgeError::NotFound { .. })`; for delay-based single-flight, add a `tokio::time::sleep` inside `fetch`.

   - **`fn test_app(state: AppState) -> Router`** — mirror `portfolio_publish.rs`: build an admin sub-router with `middleware::from_fn_with_state(Some("test_secret".to_string()), plinth_server::api::admin::auth_middleware)`, merge the public + feed routes, `.with_state(state)`:
     ```rust
     fn test_app(state: AppState) -> Router {
         use plinth_server::bricks::activity::{admin, api};
         let admin_router = Router::new()
             .route("/api/admin/activity", post(admin::publish_activity_item))
             .route(
                 "/api/admin/activity/{id}",
                 delete(admin::delete_activity_handler).patch(admin::patch_activity_handler),
             )
             .layer(middleware::from_fn_with_state(
                 Some("test_secret".to_string()),
                 plinth_server::api::admin::auth_middleware,
             ));
         Router::new()
             .route("/api/activity", get(api::list_activity_items))
             .route("/api/activity/{id}", get(api::get_activity_item))
             .route("/feeds/activity.xml", get(plinth_server::api::feeds::activity_feed))
             .route("/api/search", get(plinth_server::api::search::search_articles))
             .merge(admin_router)
             .with_state(state)
     }
     ```
     (These are the canonical handler names defined by Phases 03/04/07: `admin::publish_activity_item`, `admin::delete_activity_handler`, `admin::patch_activity_handler`, `api::list_activity_items`, `api::get_activity_item`. The `{id}` paths take `Path<i64>`.)

   - **Request helper `post_json`** — mirror `post_manifest`: build a `Request`, set `CONTENT_TYPE`, optional `AUTHORIZATION: Bearer {token}`, body = `serde_json::to_vec(&request)`, then `app.oneshot(request).await` (tower `ServiceExt`).

4. **Write the named tests** (each `#[sqlx::test(migrations = "./migrations")] async fn ...(pool: PgPool)`):

   - **`admin_upsert_requires_bearer_token`** — POST `/api/admin/activity` with no token ⇒ `401 UNAUTHORIZED`; with `Bearer test_secret` ⇒ `200`. (Mirrors `posting_without_bearer_token_returns_401`.)

   - **`admin_upsert_creates_and_upserts_by_natural_key`** — POST a `PublishActivityRequest` (forge=github, owner/repo, kind=pr, number=1, impact=5, embedding=`Some(vec![0.1f32; 384])`); assert `200`, then `SELECT COUNT(*) FROM activity_items WHERE url = $1` = 1. POST again with the same natural key (`forge, repo_owner, repo_name, kind, number`) and a higher impact; assert still 1 row and `impact` updated (proves `ON CONFLICT` upsert).

   - **`public_list_returns_ranked_order`** — `insert_activity` three rows with controlled `(impact, reference_date)` so the exponential default produces a known order (e.g. impact 10 today > impact 10 a year ago > impact 1 today); GET `/api/activity`, parse JSON, assert the `id`/`url` order matches the expected `score DESC, reference_date DESC`. Then GET `/api/activity?featured=true` after marking one `featured=true` and assert filtering.

   - **`refresh_on_stale_read_updates_db_from_mocked_forge`** — build a `MockForge` whose `fetch(&self, r: &ActivityRef)` returns a canned `FetchedActivity` with `additions: Some(99)`, `state: ActivityState::Closed`, `merged_at: Some(...)`, etc.; inject it as the `forge_client`. Insert one activity row with `fetched_at` older than the TTL (default 3600s → use `Utc::now() - chrono::Duration::hours(2)`). GET `/api/activity` (serves stale immediately — assert `200` and the *stale* additions value). Then **wait for the single-flighted background refresh** to complete: poll `state.activity_cache.ask(GetRankedActivity { limit: None, featured_only: false }).await` / `SELECT additions FROM activity_items WHERE ...` in a bounded loop (e.g. up to 5 s with 50 ms sleeps using `tokio::time::sleep`) until `additions == 99`; assert it converges. Assert `MockForge.calls.load(Ordering::SeqCst) >= 1`. (Secondary mode: build a `ForgeRouter` over a `wiremock::MockServer` with `with_base_url` and assert `server.received_requests()` instead.)

   - **`refresh_failure_keeps_stale_data`** — make `MockForge::fetch` return `Err(ForgeError::Http { forge: Forge::GitHub, status: 500, body: "..".into() })` (or `Err(ForgeError::NotFound { .. })`). Insert a stale row with known values. GET `/api/activity` ⇒ `200` with the stale values; after a bounded wait, the row is **unchanged** (no panic, no wipe) — proves "refresh failure must not break the page; keep stale data". (Secondary mode: mount a wiremock endpoint returning `500`/`404`.)

   - **`refresh_is_single_flighted`** — make `MockForge::fetch` `tokio::time::sleep(Duration::from_millis(300))` before returning success (so the first refresh is still in flight when the rest arrive). Insert a stale row. Fire N concurrent GETs `/api/activity` (e.g. spawn 10 tasks via `JoinSet`). After convergence, assert `MockForge.calls.load(Ordering::SeqCst) == 1` (the single-flight latch in `maybe_trigger_refresh` prevented a stampede). (Secondary mode: `ResponseTemplate::set_delay` + `server.received_requests()` == 1.)

   - **`activity_feed_returns_valid_rss`** — `insert_activity` one row; GET `/feeds/activity.xml`; assert status `200`, `Content-Type` contains `application/rss+xml`, body parses (string-contains `<rss`, `<item>`, the item title, and the forge URL as `<link>`). Optionally parse with the `rss` crate's `Channel::read_from` to assert one item.

   - **`search_union_includes_activity_items`** — insert one blog post (via `common::insert_blog_post`) and one activity row, both with non-null `embedding` (use the same constant 384-dim vector so cosine similarity is comparable; or insert activity with an embedding identical to the query). GET `/api/search?q=...&limit=10`; assert the response array contains an entry whose `kind`/identifier corresponds to the activity item (per Phase 07's generalized `SearchResult` shape). If the search actor is `None` in `app_state` (mirroring `vector_search: None` in portfolio tests), this test must instead construct a real `VectorSearch` actor or query the union SQL directly — match whatever Phase 07 wired. Document which.

   - **`refresh_does_not_reembed`** — insert a stale row with a known embedding (a recognizable 384-dim vector); have `MockForge::fetch` return changed metadata; trigger a refresh as above (poll via `ask(GetRankedActivity { limit: None, featured_only: false })`); after convergence assert metadata changed but `SELECT embedding FROM activity_items WHERE ...` is byte-identical to the inserted vector (proves the documented pitfall: refresh re-pulls metadata, never re-embeds).

5. **Add the `futures` dev-dep if needed** for the concurrent test (`futures = "0.3"` under `[dev-dependencies]`), or use `tokio::task::JoinSet` (already available via tokio) to avoid a new dep — prefer `JoinSet`.

6. **Local verification:**
   ```bash
   export DATABASE_URL="postgres://localhost/plinth_dev"   # or your dev DB
   cargo test -p plinth-server --test forge_activity -- --nocapture
   cargo test -p plinth-server --test forge_activity --no-default-features --features brick-blog,brick-portfolio,brick-todo  # the `disabled` module compiles + passes
   ```
   The PRIMARY mock path uses an in-process `Arc<dyn ForgeClient>` and touches no socket at all; in the secondary mode the wiremock server binds loopback, which is permitted in the Nix sandbox. Either way no real network is used.

## Acceptance criteria

- [ ] New file `crates/server/tests/forge_activity.rs` exists with an `#[cfg(feature = "brick-activity")] mod enabled` and an `#[cfg(not(...))] mod disabled` (mirroring `portfolio_publish.rs`).
- [ ] `cargo test -p plinth-server --test forge_activity` exits 0 with all of these named tests run and passing: `admin_upsert_requires_bearer_token`, `admin_upsert_creates_and_upserts_by_natural_key`, `public_list_returns_ranked_order`, `refresh_on_stale_read_updates_db_from_mocked_forge`, `refresh_failure_keeps_stale_data`, `refresh_is_single_flighted`, `activity_feed_returns_valid_rss`, `search_union_includes_activity_items`, `refresh_does_not_reembed`.
- [ ] `refresh_is_single_flighted` asserts exactly one upstream fetch was made across ≥10 concurrent reads (`MockForge.calls == 1`; or `server.received_requests()` == 1 in the secondary wiremock mode).
- [ ] `activity_feed_returns_valid_rss` asserts `Content-Type` contains `application/rss+xml` and the body contains `<item>` plus the item title.
- [ ] The `disabled` module asserts `POST /api/admin/activity` ⇒ `404` when `brick-activity` is off, and compiles under `--no-default-features` + the other three bricks.
- [ ] `crates/server/Cargo.toml` `[dev-dependencies]` contains `wiremock = "0.6"`.
- [ ] `cargo clippy -p plinth-server --tests -- --deny warnings` is clean for the new file (so `nix flake check`'s `plinth-clippy --all-targets` stays green).

## Files likely touched

- `/data/nvme0/can/Projects/solo/plinth/crates/server/tests/forge_activity.rs` — new, the whole end-to-end suite.
- `/data/nvme0/can/Projects/solo/plinth/crates/server/tests/common/mod.rs` — add `insert_activity` helper (cfg-gated).
- `/data/nvme0/can/Projects/solo/plinth/crates/server/Cargo.toml` — `[dev-dependencies]` add `wiremock` (and `futures` only if not using `JoinSet`).
- (Optional) `/data/nvme0/can/Projects/solo/plinth/crates/forge/tests/` — forge-crate integration tests if Phase 02 left gaps; use `#[sqlx::test(migrations = "../server/migrations")]` (path relative to the forge crate) for any DB-backed forge test, and `wiremock` (added by sub-01).

## Pitfalls

- **Symptom:** `app_state` does not compile — missing/extra struct field.
  **Cause:** `AppState` uses manual `#[cfg]`-gated fields (no `FromRef` derive); the field set must match the active feature set exactly.
  **Recovery:** copy `portfolio_publish.rs`'s `app_state` verbatim and add only the cfg-gated `activity_cache` field. Keep `vector_search: None` unless the search test needs a real actor.

- **Symptom:** the refresh test never observes the updated value (test hangs to timeout).
  **Cause:** the refresh is async/background and "must not block render", so the GET returns before the DB is updated; asserting immediately after the GET races the refresh.
  **Recovery:** poll in a bounded loop with `tokio::time::sleep` until convergence (or until a deadline, then `panic!`). Never assert the fresh value synchronously right after the first GET.

- **Symptom:** `refresh_is_single_flighted` sees more than one upstream fetch.
  **Cause:** the concurrent GETs were not actually concurrent (awaited sequentially), or `MockForge::fetch` has no delay so each refresh completes before the next read arrives, defeating the single-flight latch.
  **Recovery:** add a `tokio::time::sleep` inside `MockForge::fetch` (secondary mode: `ResponseTemplate::set_delay`); fire the reads with `JoinSet`/`tokio::spawn` and join all; only then read `MockForge.calls` (secondary mode: `received_requests()`).

- **Symptom:** real network call to GitHub/Codeberg ⇒ test hangs/fails in the Nix sandbox.
  **Cause:** a real `ForgeRouter` was wired as the `forge_client` instead of the injected `MockForge`, so the refresh path hit the live forge.
  **Recovery:** pass `Arc::new(MockForge { .. })` as the fourth arg of `ActivityCache::new(db, ranking, forge, forge_client)` — this is the PRIMARY, deterministic path and never touches the network. Only fall back to a real `ForgeRouter` if you specifically want to exercise HTTP normalization, and then build it with `GitHubClient::with_base_url(server.uri(), None)` / `CodebergClient::with_base_url(server.uri(), None)` and set `config.forge.github_base_url`/`config.forge.codeberg_base_url` to `server.uri()` before constructing `AppState`.

- **Symptom:** `search_union_includes_activity_items` returns nothing.
  **Cause:** activity row inserted without an `embedding`, or `vector_search: None` so the union actor never runs, or the inserted embedding is too dissimilar to the query embedding.
  **Recovery:** insert a non-null `embedding`; build a real `VectorSearch`/activity search actor if Phase 07 requires it; use a query whose embedding matches the inserted vector (or insert the vector equal to the query embedding for a guaranteed top hit).

- **Symptom:** (secondary wiremock mode only) `wiremock` JSON body shape rejected by the forge client deserializer.
  **Cause:** GitHub `/issues/{n}` vs `/pulls/{n}` differ — a PR must be served from the `/pulls/{n}` endpoint with `merged`, `additions`, `deletions`; `/issues/{n}` lacks them.
  **Recovery:** for a PR mock, mount the `/pulls/{number}` path with the full PR JSON (`merged`, `merged_at`, `additions`, `deletions`, `comments`, `labels`) plus the repo endpoint returning `stargazers_count`. For an issue, omit `additions`/`deletions`. Match Phase 02's `FetchedActivity` field names. (The PRIMARY `MockForge` path returns a `FetchedActivity` struct directly and never goes through HTTP deserialization, so it sidesteps this entirely.)

## Reference

- Skeleton: `crates/server/tests/portfolio_publish.rs` (the `app_state`/`test_app`/`post_manifest`/`#[sqlx::test]` pattern) and `crates/server/tests/common/mod.rs` (`insert_blog_post` helper shape) — both summarized inline above.
- `#[sqlx::test(migrations = "./migrations")]` resolves the path relative to `crates/server`; it creates a per-test database and applies migrations including `0006_activity.sql`.
- Sibling sub-layers (CONTEXT only): [sub-01-nix-packaging.md](./sub-01-nix-packaging.md) adds `wiremock` to `crates/forge` and gets the crate into the sandbox; this sub-layer additionally adds `wiremock` to `crates/server`. The merge agent (see [README.md](./README.md)) runs the full `nix flake check`.
