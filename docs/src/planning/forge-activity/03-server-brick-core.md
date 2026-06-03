# Phase 03 — Server brick: persistence, admin, public read, ranking

> **Recommended Codex model: GPT 5.5 medium**
>
> This phase is mostly mechanical pattern-mirroring of the existing `portfolio` brick (the same `mod.rs`/`admin.rs`/`api.rs`/`cache.rs` layout, the same Kameo actor skeleton, the same `main.rs` cfg-gated route wiring), so it does not need the deepest model. The one genuinely novel piece — the read-time ranking SQL with three switchable strategies and an exact `ON CONFLICT` upsert on a five-column natural key — is fully specified inline here (formulas, bind order, column list), so a medium model has everything it needs. A too-small model would get the load-bearing details wrong: it would try to make the `Brick` trait's `public_routes`/`admin_routes` methods load-bearing (they are dead code — wiring is hand-written in `main.rs`), it would put real DDL in `migrations.rs` (DDL lives in the embedded `.sql` file), it would forget to insert into the `schema_migrations` ledger, or it would emit ranking SQL that divides by zero / parameter-injects the strategy. Those are the failure modes the inline spec below forecloses.

## Working tree

cwd = `/data/nvme0/can/Projects/solo/plinth` (the plinth repo).

Serialization notes — this brick directory is touched by later phases too:

- **Phase 04 (lazy-refresh-actor)** will edit `crates/server/src/bricks/activity/cache.rs` and add `refresh.rs`, and will touch `crates/server/src/bricks/activity/{mod.rs,api.rs}` plus `main.rs`. Leave the explicit TTL/refresh seam described in step 7 so Phase 04 has a clean insertion point.
- **Phase 07 (feed-and-search)** will edit `crates/server/src/bricks/activity/api.rs` (or add a feed handler in `crates/server/src/api/feeds.rs`), the search service, and `main.rs` route registration.
- Both 04 and 07 also touch `crates/server/src/main.rs` route registration. If you are landing this phase and either of those has merged ahead of you, `git pull --rebase` first and re-resolve the `#[cfg(feature = "brick-activity")]` blocks in `main.rs`. Since this phase (03) is in Wave 1, you will normally land **before** 04/07; just keep your `main.rs` edits confined to clearly-delimited `#[cfg(feature = "brick-activity")]` blocks so the later phases append cleanly.

This phase assumes **Phase 01** has already landed: `plinth-shared` exposes the activity DTOs (`ActivityItem`, `ActivityListItem`, `PublishActivityRequest`, `Forge`, `ActivityKind`, `ActivityState`, `RankingStrategy`) behind the shared `brick-activity` feature, and migration `0006_activity.sql` creating the `activity_items` table exists in `crates/server/migrations/`. If those are missing, stop — this phase cannot proceed without them (see Reference).

## Goal

This phase succeeds when an operator can `POST /api/admin/activity` (Bearer-authed) to upsert a contribution by its natural key, the row lands in the `activity_items` table, and an unauthenticated `GET /api/activity` immediately returns that row ranked by the configured strategy — with `GET /api/activity/{id}` returning the single item and `DELETE`/`PATCH` admin endpoints working — all served through a Kameo `ActivityCache` actor that loads from the DB and answers a "ranked list" message, with the three ranking strategies (`exponential` default, `linear`, `pure`) computed in SQL at read time and driven by `[ranking]` config. The lazy TTL stale-while-revalidate refresh is explicitly **not** built here (Phase 04); a clearly-marked seam is left for it.

## Why this matters now

This is the server-side spine of the activity brick. Everything downstream consumes the persistence + read contract defined here: the CLI (Phase 05) POSTs to `/api/admin/activity`; the frontend (Phase 06) reads `GET /api/activity` and `/api/activity/{id}`; the feed and search union (Phase 07) read from the `ActivityCache` actor and the `activity_items` table; the lazy refresh actor (Phase 04) extends the very `cache.rs` written here. If the upsert natural key, the endpoint shapes, or the ranking score expression are wrong, every later phase inherits the bug. Landing this in Wave 1 alongside the forge crate (Phase 02) unblocks the entire downstream fan-out (04/05/06/07).

## Out of scope

- **Forge fetching / refresh** — no `reqwest`, no `plinth-forge` dependency, no network calls in this phase. The cache actor loads from the DB only. The TTL check + single-flight stale-while-revalidate refresh is **Phase 04**. Leave the seam (step 7); do not implement it.
- **CLI** — no `crates/cli` changes. `PublishActivityRequest` is consumed here only as the admin handler's JSON body type. (Phase 05.)
- **Frontend** — no `crates/client` changes, no `#[server]` fns, no routes in `app.rs`. (Phase 06.)
- **RSS/Atom feed** — do not add `/feeds/activity.xml` or an `activity_feed` handler. (Phase 07.)
- **Search union** — do not edit `crates/server/src/actors/vector_search.rs` or `crates/server/src/api/search.rs`. Activity embeddings are written to the `embedding` column by the upsert (passed through from the request) but **not** queried here. (Phase 07.)
- **Embedding generation** — the server does not run fastembed. The admin handler accepts `embedding: Option<Vec<f32>>` from the request and binds it; it does not compute it.
- **Shared DTOs and the migration SQL file** — those are Phase 01. This phase consumes them; do not redefine them. (If you must touch `0006_activity.sql` to fix a bug, that is acceptable, but the schema is locked — see the schema block below.)

## Plan

All paths below are repo-relative to `/data/nvme0/can/Projects/solo/plinth`.

### 0. Confirm Phase 01 artifacts exist

Verify before writing any code:

```bash
ls crates/server/migrations/0006_activity.sql
rg -n "pub struct ActivityItem" crates/shared/src
rg -n "pub struct PublishActivityRequest" crates/shared/src
rg -n "pub enum RankingStrategy" crates/shared/src
rg -n "brick-activity" crates/shared/Cargo.toml
```

The locked schema that `0006_activity.sql` must have created (for reference — do **not** recreate it; this is the contract your SQL binds against):

```sql
CREATE TABLE activity_items (
    id BIGSERIAL PRIMARY KEY,
    forge TEXT NOT NULL,                         -- 'github' | 'codeberg'
    repo_owner TEXT NOT NULL,
    repo_name TEXT NOT NULL,
    kind TEXT NOT NULL,                          -- 'pr' | 'issue'
    number INTEGER NOT NULL,
    url TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    body TEXT,
    state TEXT NOT NULL,                         -- 'open' | 'closed' | 'merged'
    created_at TIMESTAMPTZ NOT NULL,
    closed_at TIMESTAMPTZ,
    merged_at TIMESTAMPTZ,
    impact SMALLINT NOT NULL DEFAULT 1 CHECK (impact BETWEEN 1 AND 10),
    additions INTEGER,
    deletions INTEGER,
    comments_count INTEGER,
    labels TEXT[] DEFAULT '{}',
    repo_stars INTEGER,
    embedding vector(384),
    fetched_at TIMESTAMPTZ NOT NULL,             -- snapshot/refresh time; drives the TTL (Phase 04)
    featured BOOLEAN NOT NULL DEFAULT false,
    published BOOLEAN NOT NULL DEFAULT true,
    content_hash TEXT,
    UNIQUE (forge, repo_owner, repo_name, kind, number)
);
CREATE INDEX activity_items_embedding_hnsw_idx
    ON activity_items USING hnsw (embedding vector_cosine_ops);
```

The natural-key upsert target is `UNIQUE (forge, repo_owner, repo_name, kind, number)`. The reference date for ranking is `coalesce(merged_at, closed_at, created_at)`.

### 1. Add the `brick-activity` Cargo feature to the server crate

File: `crates/server/Cargo.toml`. The existing brick features chain to client + shared (see `brick-portfolio` around lines 72–75). Add `brick-activity` mirroring exactly:

```toml
# in [features], next to the other brick-* lines
brick-activity = ["plinth-client/brick-activity", "plinth-shared/brick-activity"]
```

Add `brick-activity` to the server `default` features list (alongside `brick-blog`, `brick-portfolio`, `brick-todo`):

```toml
default = ["ssr", "brick-blog", "brick-portfolio", "brick-todo", "brick-activity"]
```

Also add it to the workspace-level cargo-leptos feature lists in the **root** `Cargo.toml` (around lines 132 and 138) so the leptos build compiles it in:

```toml
bin-features = ["ssr", "brick-blog", "brick-portfolio", "brick-todo", "brick-activity"]
lib-features = ["hydrate", "brick-blog", "brick-portfolio", "brick-todo", "brick-activity"]
```

(Phase 01 should already have added `brick-activity = []` to `crates/shared/Cargo.toml` and `brick-activity = ["plinth-shared/brick-activity"]` to `crates/client/Cargo.toml`. If `plinth-client/brick-activity` does not exist yet, add `brick-activity = ["plinth-shared/brick-activity"]` to `crates/client/Cargo.toml`'s `[features]` and its `default`, mirroring `brick-portfolio` there — otherwise the server feature chain will not resolve.)

### 2. Register the brick module + registry push

File: `crates/server/src/bricks/mod.rs`. Mirror the portfolio gating exactly. Add the module declaration alongside the others:

```rust
#[cfg(feature = "brick-activity")]  pub mod activity;
```

And add the registry push inside `enabled_bricks()`:

```rust
#[cfg(feature = "brick-activity")] bricks.push(Box::new(activity::ActivityBrick));
```

(Reminder from the pattern: `enabled_bricks()` is only consumed by the migration runner to count expected `(brick, version)` tuples. The trait's route methods are NOT called at runtime — routes are wired by hand in `main.rs`, step 8.)

### 3. `bricks/activity/mod.rs` — `Brick` impl

File: `crates/server/src/bricks/activity/mod.rs`. Mirror `crates/server/src/bricks/portfolio/mod.rs` exactly:

```rust
pub mod admin;
pub mod api;
pub mod cache;
pub mod migrations;
pub mod ranking;

use super::{Brick, BrickMigration};

pub struct ActivityBrick;

impl Brick for ActivityBrick {
    fn name(&self) -> &'static str { "activity" }
    fn migrations(&self) -> Vec<BrickMigration> { migrations::activity_migrations() }
}
```

### 4. `bricks/activity/migrations.rs` — metadata tuple only

File: `crates/server/src/bricks/activity/migrations.rs`. The `up` field is always `""` (real DDL is in the embedded `.sql`; the runner ignores `up`). Mirror `crates/server/src/bricks/portfolio/migrations.rs`:

```rust
use super::BrickMigration;

pub fn activity_migrations() -> Vec<BrickMigration> {
    vec![BrickMigration {
        brick: "activity",
        version: 1,
        name: "initial_activity_schema",
        up: "",
    }]
}
```

Confirm `0006_activity.sql` (Phase 01) ends with the matching ledger insert so `migration_status()` reporting agrees:

```sql
INSERT INTO schema_migrations (brick, version, name)
VALUES ('activity', 1, 'initial_activity_schema');
```

If that `INSERT` is missing from `0006_activity.sql`, add it (it is required — there are two parallel ledgers: sqlx's `_sqlx_migrations` drives execution; `schema_migrations` drives status reporting).

### 5. `bricks/activity/ranking.rs` — the read-time score SQL (the novel piece)

File: `crates/server/src/bricks/activity/ranking.rs`. This module produces a SQL **fragment** for the score, plus the `ORDER BY` clause, driven by the `[ranking]` config. The score is computed at read time — there is **no stored score column**. Definitions:

- `reference_date = coalesce(merged_at, closed_at, created_at)`
- `age_days = extract(epoch from (now() - reference_date)) / 86400.0`

The three strategies (formulas locked):

- **exponential** (default): `impact * power(0.5, age_days / $half_life_days)` — `half_life_days` default `365`.
- **linear**: `impact * greatest(0.0, 1.0 - age_days / $window_days)` — `window_days` default `730`.
- **pure**: `impact` — recency is only a tiebreaker.

All strategies `ORDER BY score DESC, reference_date DESC`.

The strategy and its parameters come from `[ranking]` config and are threaded into the query. **Never** interpolate the strategy name as a string into the SQL via formatting in a way that could inject user input — the strategy comes from a typed `RankingStrategy` enum (from `plinth-shared`, Phase 01), so a `match` on the enum selecting a fixed `&'static str` fragment is safe and idiomatic. Numeric params (`half_life_days`, `window_days`) must be **bound** (`$N`), not formatted.

Read the `[ranking]` config struct first to learn its exact field names — Phase 01 may have added it to `crates/shared/src/toml_config.rs`. If it has NOT been added yet, add it here (this phase owns the `[ranking]` config keys). The struct, mirroring the existing `SearchConfig`/`ContentConfig` serde-defaults pattern in `crates/shared/src/toml_config.rs`:

```rust
/// [ranking] section — activity ranking strategy + params.
#[derive(Debug, Clone, Deserialize)]
pub struct RankingConfig {
    #[serde(default = "default_ranking_strategy")]
    pub strategy: plinth_shared::RankingStrategy,   // {Exponential, Linear, Pure}
    #[serde(default = "default_half_life_days")]
    pub half_life_days: f64,
    #[serde(default = "default_window_days")]
    pub window_days: f64,
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            strategy: default_ranking_strategy(),
            half_life_days: default_half_life_days(),
            window_days: default_window_days(),
        }
    }
}

fn default_ranking_strategy() -> plinth_shared::RankingStrategy {
    plinth_shared::RankingStrategy::Exponential
}
fn default_half_life_days() -> f64 { 365.0 }
fn default_window_days() -> f64 { 730.0 }
```

Then wire `#[serde(default)] pub ranking: RankingConfig,` into the top-level `PlinthConfig` struct (around `crates/shared/src/toml_config.rs:476`, the section list that already includes `search`, `content`, `feeds`). `PlinthConfig` derives `Default` and every section impls `Default`, so partial/empty TOML still parses (proven by `test_parse_partial_toml` / `test_parse_empty_toml`). `RankingStrategy` must `#[derive(Deserialize)]` with `#[serde(rename_all = "lowercase")]` so `strategy = "exponential"` parses — verify Phase 01 did this; if not, this phase adds it.

The ranking module itself (the score fragment + bind plumbing). The cleanest shape that avoids both SQL injection and divide-by-zero (clamp params to a positive floor):

```rust
use plinth_shared::RankingStrategy;

/// Returns a SQL scalar expression computing the ranking score, using $score_param
/// as the first bound parameter for half_life/window (pure ignores it).
/// `ref_expr` is the reference-date expression to reuse.
pub const REF_DATE_SQL: &str =
    "coalesce(merged_at, closed_at, created_at)";

/// age_days SQL given the reference-date expression.
pub fn age_days_sql() -> String {
    format!("(extract(epoch from (now() - {REF_DATE_SQL})) / 86400.0)")
}

/// The score expression for a strategy. `$N` is the bound numeric param
/// (half_life_days for exponential, window_days for linear; unused for pure).
pub fn score_sql(strategy: RankingStrategy, param_placeholder: &str) -> String {
    let age = age_days_sql();
    match strategy {
        RankingStrategy::Exponential =>
            format!("(impact::float8 * power(0.5, {age} / greatest({param_placeholder}, 0.000001)))"),
        RankingStrategy::Linear =>
            format!("(impact::float8 * greatest(0.0, 1.0 - {age} / greatest({param_placeholder}, 0.000001)))"),
        RankingStrategy::Pure =>
            "(impact::float8)".to_string(),
    }
}

/// The numeric param to bind for a strategy (None for pure -> bind a dummy or skip).
pub fn score_param(strategy: RankingStrategy, half_life_days: f64, window_days: f64) -> f64 {
    match strategy {
        RankingStrategy::Exponential => half_life_days,
        RankingStrategy::Linear => window_days,
        RankingStrategy::Pure => 1.0, // unused; bound to keep $N positions stable
    }
}
```

Note on `greatest({param}, 0.000001)`: this guards divide-by-zero if `half_life_days`/`window_days` is misconfigured to `0`. The `impact::float8` cast is required because `impact` is `SMALLINT` and `power(...)` returns `double precision`; mixing without the cast risks integer truncation in the multiplication.

`ORDER BY` is the same for all three: `ORDER BY <score_expr> DESC, <REF_DATE_SQL> DESC` (the score expr is re-inlined in the `ORDER BY`, since Postgres allows ordering by an aliased select column — alias it `score` in the `SELECT` and `ORDER BY score DESC, ` + the reference-date expr).

### 6. `bricks/activity/cache.rs` — the Kameo `ActivityCache` actor (NO TTL refresh)

File: `crates/server/src/bricks/activity/cache.rs`. Mirror the skeleton of `crates/server/src/bricks/portfolio/cache.rs` (same `Actor` derive, same `new(db)`, same `Instant`-based cache fields), but with activity-specific messages and the ranked query. **Important**: this phase deliberately does NOT implement the lazy TTL stale-while-revalidate refresh — that is Phase 04. Keep a simple invalidation-on-write cache (like portfolio's), and leave the explicit seam in step 7.

Imports + state (mirror portfolio's TTL skeleton; the in-memory list cache is keyed by the active ranking strategy because the ordering depends on it):

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

use kameo::Actor;
use kameo::message::{Context, Message};

use plinth_shared::{ActivityItem, ActivityListItem};
use plinth_shared::toml_config::RankingConfig;   // CANON: RankingConfig lives in toml_config.rs
use crate::PlinthDb;
use crate::services::rows;

const CACHE_TTL: Duration = Duration::from_secs(5 * 60);
const MAX_ITEM_CACHE_SIZE: usize = 500;

#[derive(Actor)]
pub struct ActivityCache {
    db: PlinthDb,
    // Ranking config snapshot so the ranked query is reproducible inside the actor.
    ranking: RankingConfig,   // { strategy, half_life_days, window_days }
    // Per-id item cache (for GET /api/activity/{id}).
    items: HashMap<i64, ActivityItem>,
    // Ranked list cache (the public list); invalidated on any write.
    ranked_list_cache: Option<Vec<ActivityListItem>>,
    cache_populated_at: Option<Instant>,
    // === Phase 04 seam: refreshing flag + last_refresh_attempt live here. ===
    // Do NOT add refresh logic in this phase; Phase 04 adds:
    //   refreshing: bool,
    //   last_refresh_attempt: Option<Instant>,
    //   forge: ForgeConfig,
    //   forge_client: Arc<dyn ForgeClient + Send + Sync>,
    // and a single-flighted stale-while-revalidate path keyed off fetched_at + TTL.
}

impl ActivityCache {
    // PHASE 03 ctor — NO forge here. Phase 04 EXTENDS this signature to
    //   ActivityCache::new(db, ranking, forge, forge_client)
    // and updates every call site, INCLUDING this phase's integration test
    // (crates/server/tests/activity_brick.rs). Keep this seam clean.
    pub fn new(db: PlinthDb, ranking: RankingConfig) -> Self {
        Self {
            db, ranking,
            items: HashMap::new(),
            ranked_list_cache: None,
            cache_populated_at: None,
        }
    }
    fn is_expired(&self) -> bool {
        self.cache_populated_at.is_some_and(|t| t.elapsed() > CACHE_TTL)
    }
    fn clear_all(&mut self) {
        self.items.clear();
        self.ranked_list_cache = None;
        self.cache_populated_at = None;
    }
    fn touch(&mut self) {
        if self.cache_populated_at.is_none() { self.cache_populated_at = Some(Instant::now()); }
    }
    fn expire_if_stale(&mut self) { if self.is_expired() { self.clear_all(); } }
}
```

Messages (three, mirroring portfolio's `GetAll*` / `Get*` / `InvalidateCache`):

```rust
/// Ranked public list. `limit`/`featured_only` come from query params.
pub struct GetRankedActivity {
    pub limit: Option<i64>,
    pub featured_only: bool,
}
impl Message<GetRankedActivity> for ActivityCache {
    type Reply = Result<Vec<ActivityListItem>, String>;
    async fn handle(&mut self, msg: GetRankedActivity, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        self.expire_if_stale();
        // Cache only the unfiltered full list; apply featured/limit on top so the
        // cache key stays simple. (Phase 04 will replace this with TTL+refresh.)
        if msg.featured_only || self.ranked_list_cache.is_none() {
            let list = self.query_ranked(msg.featured_only, msg.limit).await?;
            if !msg.featured_only {
                self.ranked_list_cache = Some(list.clone());
                self.touch();
            }
            return Ok(apply_limit(list, msg.limit));
        }
        let cached = self.ranked_list_cache.clone().unwrap_or_default();
        Ok(apply_limit(cached, msg.limit))
    }
}

pub struct GetActivityItem(pub i64);
impl Message<GetActivityItem> for ActivityCache {
    type Reply = Result<Option<ActivityItem>, String>;
    async fn handle(&mut self, msg: GetActivityItem, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        self.expire_if_stale();
        let id = msg.0;
        if let Some(item) = self.items.get(&id) { return Ok(Some(item.clone())); }
        let row = sqlx::query("SELECT * FROM activity_items WHERE id = $1 AND published = true LIMIT 1")
            .bind(id).fetch_optional(&self.db).await
            .map_err(|e| format!("Database error: {e}"))?;
        let item = row.map(rows::activity_item).transpose()
            .map_err(|e| format!("Database error: {e}"))?;
        if let Some(ref item) = item {
            if self.items.len() < MAX_ITEM_CACHE_SIZE {
                self.items.insert(id, item.clone());
                self.touch();
            }
        }
        Ok(item)
    }
}

pub struct ActivityInvalidateCache;
impl Message<ActivityInvalidateCache> for ActivityCache {
    type Reply = ();
    async fn handle(&mut self, _msg: ActivityInvalidateCache, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        self.clear_all();
    }
}

fn apply_limit(mut list: Vec<ActivityListItem>, limit: Option<i64>) -> Vec<ActivityListItem> {
    if let Some(n) = limit {
        let n = n.max(0) as usize;
        list.truncate(n);
    }
    list
}
```

The ranked SELECT lives in a **shared, free `pub` helper** in `ranking.rs` so that BOTH the cache actor (`query_ranked`) AND Phase 04's background refresh worker (`reread_ranked`) call the exact same query — the score expression and `ORDER BY` exist in exactly one place. Bind order: `$1` = the numeric ranking param. The `score` column is computed and also drives `ORDER BY`. `ActivityListItem` carries a computed `score` field (Phase 01).

In `crates/server/src/bricks/activity/ranking.rs` (alongside `score_sql` / `REF_DATE_SQL` / `score_param`):

```rust
use plinth_shared::{ActivityListItem, toml_config::RankingConfig};
use crate::{PlinthDb, services::rows};

/// Canonical ranked-list read. Free + `pub` so the cache actor AND the Phase 04
/// refresh worker share ONE SELECT (one score expression, one ORDER BY).
/// `featured_only` filters to featured rows; `limit` adds a SQL `LIMIT` when set
/// (the cache actor passes `None` and truncates per-request via `apply_limit`).
pub async fn query_ranked_list(
    db: &PlinthDb,
    ranking: &RankingConfig,
    featured_only: bool,
    limit: Option<i64>,
) -> Result<Vec<ActivityListItem>, sqlx::Error> {
    let strategy = ranking.strategy;
    let score_expr = score_sql(strategy, "$1");
    let ref_date = REF_DATE_SQL;
    let where_featured = if featured_only { "AND featured = true" } else { "" };
    let limit_clause = if limit.is_some() { "LIMIT $2" } else { "" };
    // Column projection is EXACTLY the canonical ActivityListItem shape (no
    // comments_count, repo_stars, fetched_at, body, or embedding here) plus the
    // SQL-computed score. Do not SELECT * — project the columns rows::activity_list_item decodes.
    let sql = format!(
        r#"
        SELECT
            id, forge, repo_owner, repo_name, kind, number, url, title,
            state, created_at, closed_at, merged_at, impact,
            labels, featured,
            {score_expr} AS score
        FROM activity_items
        WHERE published = true {where_featured}
        ORDER BY score DESC, {ref_date} DESC
        {limit_clause}
        "#
    );
    let param = score_param(strategy, ranking.half_life_days, ranking.window_days);
    let mut q = sqlx::query(&sql).bind(param);
    if let Some(n) = limit { q = q.bind(n.max(0)); }
    let rows = q.fetch_all(db).await?;
    rows.into_iter().map(rows::activity_list_item).collect()
}
```

The cache actor's method is then a thin delegate that keeps the repo-standard `Result<_, String>` reply (matching the live portfolio cache idiom):

```rust
async fn query_ranked(&self, featured_only: bool, limit: Option<i64>)
    -> Result<Vec<ActivityListItem>, String> {
    crate::bricks::activity::ranking::query_ranked_list(&self.db, &self.ranking, featured_only, limit)
        .await
        .map_err(|e| format!("Database error: {e}"))
}
```

Note: for the cache actor, `query_ranked` is called with `limit = None` (the full list is cached) and per-request limits are applied in `apply_limit`, so the cached list stays limit-agnostic. The shared `query_ranked_list` still accepts an optional SQL `LIMIT` for direct, non-caching callers; just do not cache a limited result.

### 7. The Phase-04 seam (explicit marker)

In `cache.rs`, immediately above the `ActivityCache` struct and again inside `GetRankedActivity::handle`, leave a comment block exactly like this so Phase 04 has an unambiguous insertion point:

```rust
// ============================================================================
// PHASE 04 SEAM: lazy stale-while-revalidate refresh.
// This phase (03) intentionally serves only DB-backed cached data. Phase 04
// adds: a `refreshing: bool` single-flight guard + `last_refresh_attempt`,
// a per-served-entry `fetched_at` TTL check (default 1h from [refresh]/[ranking]
// config), and a non-blocking background refresh that re-pulls forge metadata
// via plinth-forge and updates DB + cache. DO NOT block reads on refresh.
// ============================================================================
```

Do not implement any of it here. For the re-read after a successful refresh, Phase 04 calls the shared `pub` helper defined in section 6 — `bricks::activity::ranking::query_ranked_list(db, ranking, false, None)` — so the ranked SELECT is never duplicated.

### 8. `services/db.rs` — the upsert + `services/rows.rs` decoders

#### 8a. Upsert (file: `crates/server/src/services/db.rs`)

Add `upsert_activity_item`, gated `#[cfg(feature = "brick-activity")]`, mirroring `upsert_portfolio_item` but with the five-column natural-key `ON CONFLICT` and the `embedding` bound via `vector_or_none` (the existing pgvector adapter in this same file). The conflict target is the natural key, NOT `url` (both are unique; the brief specifies upsert by the natural key). Column order and bind order MUST match.

**The upsert takes the `PublishActivityRequest` directly**, plus a server-stamped `fetched_at: DateTime<Utc>` — because the canonical `ActivityItem` carries neither the `embedding` vector (write-only via the request) nor a construct-time id, while `PublishActivityRequest` carries the embedding and all natural-key fields. The SERVER (not the request) supplies `fetched_at`:

```rust
#[cfg(feature = "brick-activity")]
pub async fn upsert_activity_item(
    db: &PlinthDb,
    request: &plinth_shared::PublishActivityRequest,
    fetched_at: chrono::DateTime<chrono::Utc>,   // server-stamped (chrono::Utc::now())
) -> Result<i64, sqlx::Error> {
    use crate::services::db::vector_or_none; // same module; or call directly if private-in-mod
    let embedding = vector_or_none(request.embedding.clone());
    sqlx::query_scalar(
        r#"
        INSERT INTO activity_items (
            forge, repo_owner, repo_name, kind, number, url, title, body, state,
            created_at, closed_at, merged_at, impact, additions, deletions,
            comments_count, labels, repo_stars, embedding, fetched_at,
            featured, published, content_hash
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23)
        ON CONFLICT (forge, repo_owner, repo_name, kind, number) DO UPDATE SET
            url = EXCLUDED.url,
            title = EXCLUDED.title,
            body = EXCLUDED.body,
            state = EXCLUDED.state,
            created_at = EXCLUDED.created_at,
            closed_at = EXCLUDED.closed_at,
            merged_at = EXCLUDED.merged_at,
            impact = EXCLUDED.impact,
            additions = EXCLUDED.additions,
            deletions = EXCLUDED.deletions,
            comments_count = EXCLUDED.comments_count,
            labels = EXCLUDED.labels,
            repo_stars = EXCLUDED.repo_stars,
            embedding = COALESCE(EXCLUDED.embedding, activity_items.embedding),
            fetched_at = EXCLUDED.fetched_at,
            featured = EXCLUDED.featured,
            published = EXCLUDED.published,
            content_hash = EXCLUDED.content_hash
        RETURNING id
        "#,
    )
    .bind(request.forge.as_str())       // 'github' | 'codeberg'  (Forge::as_str from shared)
    .bind(&request.repo_owner)
    .bind(&request.repo_name)
    .bind(request.kind.as_str())        // 'pr' | 'issue'
    .bind(request.number)               // i32 / INTEGER
    .bind(&request.url)
    .bind(&request.title)
    .bind(&request.body)                // Option<String>
    .bind(request.state.as_str())       // 'open' | 'closed' | 'merged'
    .bind(request.created_at)
    .bind(request.closed_at)            // Option<DateTime<Utc>>
    .bind(request.merged_at)            // Option<DateTime<Utc>>
    .bind(request.impact)               // i16 / SMALLINT
    .bind(request.additions)            // Option<i32>
    .bind(request.deletions)            // Option<i32>
    .bind(request.comments_count)       // Option<i32>
    .bind(&request.labels)              // Vec<String> -> TEXT[]
    .bind(request.repo_stars)           // Option<i32>
    .bind(embedding)                    // Option<pgvector::Vector> -> vector(384)
    .bind(fetched_at)                   // SERVER-stamped, NOT request.fetched_at (no such field)
    .bind(request.featured)
    .bind(request.published)            // plain bool (serde default = true), NOT Option
    .bind(&request.content_hash)        // Option<String>
    .fetch_one(db)
    .await
}
```

Note the `embedding = COALESCE(EXCLUDED.embedding, activity_items.embedding)` on conflict: this preserves a previously-stored embedding if a later upsert (e.g. a refresh in Phase 04) supplies `None`. This is the pitfall the brief flags — refresh does not re-embed, so it must not wipe the embedding.

Add `DELETE` and `PATCH` helpers too (the admin handlers call these):

```rust
#[cfg(feature = "brick-activity")]
pub async fn delete_activity_item(db: &PlinthDb, id: i64) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("DELETE FROM activity_items WHERE id = $1")
        .bind(id).execute(db).await?;
    Ok(res.rows_affected())
}

#[cfg(feature = "brick-activity")]
pub async fn patch_activity_item(
    db: &PlinthDb,
    id: i64,
    impact: Option<i16>,
    featured: Option<bool>,
    published: Option<bool>,
) -> Result<bool, sqlx::Error> {
    // COALESCE keeps existing values when a field is None.
    let res = sqlx::query(
        r#"
        UPDATE activity_items SET
            impact = COALESCE($2, impact),
            featured = COALESCE($3, featured),
            published = COALESCE($4, published)
        WHERE id = $1
        "#,
    )
    .bind(id).bind(impact).bind(featured).bind(published)
    .execute(db).await?;
    Ok(res.rows_affected() > 0)
}
```

#### 8b. Row decoders (file: `crates/server/src/services/rows.rs`)

Add two `#[cfg(feature = "brick-activity")]`-gated decoders mirroring the portfolio/blog ones (`pub fn <type>(row: PgRow) -> Result<T, sqlx::Error>`). Both follow the CANON decoder rules:

- `id` is a plain `i64` via `row.try_get::<i64, _>("id")?` — **NOT** `Some(...)`, and **NOT** the `id(table, value)` flexible-string helper (activity ids are numeric end-to-end; there is no slug).
- `forge`/`kind`/`state` decode the TEXT column into a `String` then `.parse()` into the `Forge`/`ActivityKind`/`ActivityState` enums via the `FromStr` impls Phase 01 provides: `row.try_get::<String, _>(col)?.parse()?` (the `?` adapts `ParseEnumError` into `sqlx::Error::Decode`).

- `activity_item(row: PgRow) -> Result<ActivityItem, sqlx::Error>` — decodes the full row from a `SELECT *`. Per CANON A, `ActivityItem` does **NOT** carry the `embedding` vector (write-only via the request; used only by Phase 07 search), so the decoder does not read it.
- `activity_list_item(row: PgRow) -> Result<ActivityListItem, sqlx::Error>` — decodes EXACTLY the canonical `ActivityListItem` columns projected by `query_ranked` (step 6) **including the computed `score` column** (`try_get::<f64, _>("score")`). `ActivityListItem` does NOT carry `body`/`embedding`/`comments_count`/`repo_stars`/`fetched_at` (list view); `reference_date` is **not** a column — it is derived via the `reference_date()` helper.

Decoder skeleton to mirror (from the blog decoder pattern):

```rust
#[cfg(feature = "brick-activity")]
pub fn activity_item(row: sqlx::postgres::PgRow) -> Result<plinth_shared::ActivityItem, sqlx::Error> {
    use sqlx::Row;
    Ok(plinth_shared::ActivityItem {
        id: row.try_get::<i64, _>("id")?,                          // plain i64, NOT Some(...)
        forge: row.try_get::<String, _>("forge")?.parse()?,        // FromStr (Phase 01)
        repo_owner: row.try_get("repo_owner")?,
        repo_name: row.try_get("repo_name")?,
        kind: row.try_get::<String, _>("kind")?.parse()?,          // FromStr (Phase 01)
        number: row.try_get("number")?,
        url: row.try_get("url")?,
        title: row.try_get("title")?,
        body: row.try_get("body")?,
        state: row.try_get::<String, _>("state")?.parse()?,        // FromStr (Phase 01)
        created_at: row.try_get("created_at")?,
        closed_at: row.try_get("closed_at")?,
        merged_at: row.try_get("merged_at")?,
        impact: row.try_get("impact")?,
        additions: row.try_get("additions")?,
        deletions: row.try_get("deletions")?,
        comments_count: row.try_get("comments_count")?,
        labels: row.try_get("labels")?,
        repo_stars: row.try_get("repo_stars")?,
        fetched_at: row.try_get("fetched_at")?,
        featured: row.try_get("featured")?,
        published: row.try_get("published")?,
        content_hash: row.try_get("content_hash")?,
    })
}

#[cfg(feature = "brick-activity")]
pub fn activity_list_item(row: sqlx::postgres::PgRow) -> Result<plinth_shared::ActivityListItem, sqlx::Error> {
    use sqlx::Row;
    Ok(plinth_shared::ActivityListItem {
        id: row.try_get::<i64, _>("id")?,                          // plain i64, NOT Some(...)
        forge: row.try_get::<String, _>("forge")?.parse()?,        // FromStr (Phase 01)
        repo_owner: row.try_get("repo_owner")?,
        repo_name: row.try_get("repo_name")?,
        kind: row.try_get::<String, _>("kind")?.parse()?,          // FromStr (Phase 01)
        number: row.try_get("number")?,
        url: row.try_get("url")?,
        title: row.try_get("title")?,
        state: row.try_get::<String, _>("state")?.parse()?,        // FromStr (Phase 01)
        created_at: row.try_get("created_at")?,
        closed_at: row.try_get("closed_at")?,
        merged_at: row.try_get("merged_at")?,
        impact: row.try_get("impact")?,
        labels: row.try_get("labels")?,
        featured: row.try_get("featured")?,
        score: row.try_get::<f64, _>("score")?,                    // SQL-computed
    })
}
```

(`?` on `.parse()` relies on the Phase-01 `FromStr` impls whose `Err = ParseEnumError` converts into `sqlx::Error::Decode`; ensure `ParseEnumError: std::error::Error + Send + Sync + 'static` so the `From` for `sqlx::Error` applies. The `try_get::<i64, _>("id")` form keeps ids numeric — do not wrap in `Some(...)` or route through the `id(table, value)` string helper.)

### 9. `bricks/activity/admin.rs` — admin handlers

File: `crates/server/src/bricks/activity/admin.rs`. Mirror `crates/server/src/bricks/portfolio/admin.rs` (validate via the shared `validate_activity_fields` → stamp `fetched_at = chrono::Utc::now()` → call the `services/db.rs` fn with the request → invalidate the cache actor with `.ask(ActivityInvalidateCache)`). Imports follow the portfolio handler (`axum::extract::{State, Path}`, `axum::Json`, `crate::AppState`, `crate::error::PlinthError`, the `services::db` fns, the cache message). Note the canonical contract: `PublishActivityRequest` has **NO** `fetched_at` field (the server stamps it) and its `published` is a plain `bool` (serde default `true`, **not** `Option`). The upsert handler:

```rust
use axum::extract::{Path, State};
use axum::Json;
use tracing::warn;

use plinth_shared::{validate_activity_fields, PublishActivityRequest};
use crate::AppState;
use crate::error::PlinthError;
use crate::services::db::{upsert_activity_item, delete_activity_item, patch_activity_item};
use super::cache::ActivityInvalidateCache;

/// POST /api/admin/activity — upsert by natural key.
pub async fn publish_activity_item(
    State(state): State<AppState>,
    Json(request): Json<PublishActivityRequest>,
) -> Result<Json<serde_json::Value>, PlinthError> {
    // Validation mirrors CANON A: impact 1..=10, owner/name non-empty, number > 0.
    // Reuse the shared validator (Phase 01) so server + CLI agree.
    validate_activity_fields(
        request.impact,
        &request.repo_owner,
        &request.repo_name,
        request.number,
    )
    .map_err(|e| PlinthError::validation(e.to_string()))?;

    // The SERVER stamps fetched_at on every insert/refresh — the request has NO
    // fetched_at field. `published` is a plain bool on the request (serde default = true).
    let fetched_at = chrono::Utc::now();
    let title = request.title.clone();

    // The upsert consumes the request directly (it carries the embedding, which
    // ActivityItem does not) plus the server-stamped fetched_at.
    let id = upsert_activity_item(&state.db, &request, fetched_at).await?;

    if let Err(e) = state.activity_cache.ask(ActivityInvalidateCache).await {
        warn!("Activity cache invalidation failed: {e}");
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "id": id,
        "message": format!("Activity '{title}' published"),
    })))
}

/// DELETE /api/admin/activity/{id}
pub async fn delete_activity_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, PlinthError> {
    let n = delete_activity_item(&state.db, id).await?;
    if n == 0 {
        return Err(PlinthError::not_found(format!("activity item {id} not found")));
    }
    if let Err(e) = state.activity_cache.ask(ActivityInvalidateCache).await {
        warn!("Activity cache invalidation failed: {e}");
    }
    Ok(Json(serde_json::json!({ "success": true, "deleted": id })))
}

/// PATCH /api/admin/activity/{id}
#[derive(serde::Deserialize)]
pub struct PatchActivityBody {
    pub impact: Option<i16>,
    pub featured: Option<bool>,
    pub published: Option<bool>,
}

pub async fn patch_activity_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<PatchActivityBody>,
) -> Result<Json<serde_json::Value>, PlinthError> {
    if let Some(impact) = body.impact {
        if !(1..=10).contains(&impact) {
            return Err(PlinthError::validation("impact must be between 1 and 10"));
        }
    }
    let updated = patch_activity_item(&state.db, id, body.impact, body.featured, body.published).await?;
    if !updated {
        return Err(PlinthError::not_found(format!("activity item {id} not found")));
    }
    if let Err(e) = state.activity_cache.ask(ActivityInvalidateCache).await {
        warn!("Activity cache invalidation failed: {e}");
    }
    Ok(Json(serde_json::json!({ "success": true, "updated": id })))
}
```

Check the exact `PlinthError` constructor names in `crate::error` (search `rg "pub fn validation" crates/server/src/error.rs` and `rg "not_found|fn actor" crates/server/src/error.rs`). Portfolio uses `PlinthError::validation(...)` and `PlinthError::actor(...)`; use whatever exists for not-found (it may be `PlinthError::not_found` or you may return an `Err` with a 404 status — match the existing convention). The `From<sqlx::Error> for PlinthError` impl lets you `?` the db calls (portfolio relies on it).

### 10. `bricks/activity/api.rs` — public read handlers

File: `crates/server/src/bricks/activity/api.rs`. Mirror `crates/server/src/bricks/portfolio/api.rs` — thin handlers that `.ask()` the cache actor. Add a query-param extractor for `limit`/`featured`:

```rust
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;

use plinth_shared::{ActivityItem, ActivityListItem};
use crate::AppState;
use crate::error::PlinthError;
use super::cache::{GetActivityItem, GetRankedActivity};

#[derive(Deserialize, Default)]
pub struct ActivityListQuery {
    pub limit: Option<i64>,
    #[serde(default)]
    pub featured: bool,
}

/// GET /api/activity — ranked list (query: limit, featured)
pub async fn list_activity_items(
    State(state): State<AppState>,
    Query(q): Query<ActivityListQuery>,
) -> Result<Json<Vec<ActivityListItem>>, PlinthError> {
    let items = state.activity_cache
        .ask(GetRankedActivity { limit: q.limit, featured_only: q.featured })
        .await
        .map_err(PlinthError::actor)?;
    Ok(Json(items))
}

/// GET /api/activity/{id}
pub async fn get_activity_item(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Option<ActivityItem>>, PlinthError> {
    let item = state.activity_cache.ask(GetActivityItem(id)).await
        .map_err(PlinthError::actor)?;
    Ok(Json(item))
}
```

### 11. `lib.rs` — add the cfg-gated `AppState` field

File: `crates/server/src/lib.rs` (around lines 34–52, the `AppState` struct). `FromRef` derive is deliberately avoided; fields are hand-`#[cfg]`-gated. Add, next to `portfolio_cache`:

```rust
#[cfg(feature = "brick-activity")]
pub activity_cache: ActorRef<bricks::activity::cache::ActivityCache>,
```

### 12. `main.rs` — spawn the actor + wire routes

File: `crates/server/src/main.rs`. Four edits, each in a `#[cfg(feature = "brick-activity")]` block mirroring the portfolio blocks exactly (keep blocks self-contained so Phase 04/07 append cleanly).

**a. Spawn the actor** (near where `portfolio_cache` is spawned), threading the `[ranking]` config:

```rust
#[cfg(feature = "brick-activity")]
let activity_cache = {
    use plinth_server::bricks::activity::cache::ActivityCache;
    // PHASE 03 ctor: (db, ranking). Phase 04 EXTENDS this to add forge + forge_client
    // and rewires this spawn (build a ForgeRouter from [forge] + env tokens).
    ActivityCache::spawn(ActivityCache::new(
        db.clone(),
        config.ranking.clone(),   // PlinthConfig.ranking : RankingConfig (step 5)
    ))
};
```

**b. Admin router** (mirror the `#[cfg(feature = "brick-portfolio")]` admin block):

```rust
#[cfg(feature = "brick-activity")]
{
    admin_router = admin_router
        .route("/admin/activity", post(bricks::activity::admin::publish_activity_item))
        .route("/admin/activity/{id}", delete(bricks::activity::admin::delete_activity_handler)
                                          .patch(bricks::activity::admin::patch_activity_handler));
}
```

(`patch` is `axum::routing::patch`; ensure it is imported in `main.rs`'s `use axum::routing::{...}` line — add `patch` if absent.)

**c. Public router** (mirror the portfolio public block):

```rust
#[cfg(feature = "brick-activity")]
{
    public_api_router = public_api_router
        .route("/activity", get(bricks::activity::api::list_activity_items))
        .route("/activity/{id}", get(bricks::activity::api::get_activity_item));
}
```

**d. `AppState { ... }` literal** — add the field to the struct construction:

```rust
#[cfg(feature = "brick-activity")]
activity_cache,
```

Do **not** add a feed route or sitemap entry here — those are Phase 07.

### 13. Integration test

File: `crates/server/tests/activity_brick.rs`. Mirror `crates/server/tests/portfolio_publish.rs` exactly: `#[cfg(feature = "brick-activity")] mod enabled { ... }` + `#[cfg(not(...))] mod disabled { ... }`, an `app_state(pool) -> AppState` builder (spawning `CoreCache`, the activity cache, all other cfg-gated caches), a `test_app(state) -> Router` that mounts admin (with `auth_middleware` + `Some("test_secret")`) and public routers, and a `post_json`/`get` request helper using `tower::ServiceExt::oneshot`. Use `#[sqlx::test(migrations = "./migrations")]` (path resolves relative to `crates/server`).

The activity cache MUST be spawned with the **Phase 03 ctor** `ActivityCache::new(db, ranking)`, where `ranking` is a `RankingConfig`. Parameterize the builder by `RankingStrategy` so the ranking tests (4–6) can pick the strategy:

```rust
use plinth_shared::RankingStrategy;
use plinth_shared::toml_config::RankingConfig;
use plinth_server::bricks::activity::cache::ActivityCache;

fn app_state_with(pool: PgPool, strategy: RankingStrategy) -> AppState {
    let ranking = RankingConfig {
        strategy,
        half_life_days: 365.0,
        window_days: 730.0,
    };
    let activity_cache = ActivityCache::spawn(ActivityCache::new(pool.clone(), ranking));
    // ... spawn CoreCache + every other cfg-gated cache, build AppState { activity_cache, .. } ...
}

// Default-strategy builder for the non-ranking tests.
fn app_state(pool: PgPool) -> AppState { app_state_with(pool, RankingStrategy::Exponential) }
```

> **Phase 04 note:** Phase 04 EXTENDS the ctor to `ActivityCache::new(db, ranking, forge, forge_client)` and WILL update this exact test builder to pass a `ForgeConfig` plus a mock `Arc<dyn ForgeClient>`. Leave this call site as the clean Phase-03 two-arg form; do not pre-add the forge args here.

Write these named tests (each `async fn ...(pool: PgPool)`):

1. `admin_upsert_then_public_get_returns_it` — POST `/api/admin/activity` with a valid `PublishActivityRequest` JSON + `Authorization: Bearer test_secret`; assert `200 OK`. Then GET `/api/activity` (no auth); assert `200 OK` and the body (`Vec<ActivityListItem>`) contains the upserted item (match on `url`). Also assert the DB row exists: `SELECT COUNT(*) FROM activity_items WHERE url = $1` returns `1`.
2. `admin_upsert_is_idempotent_on_natural_key` — POST the same natural key twice with different `impact`; assert one row (`COUNT(*) = 1`) and the second `impact` wins.
3. `public_get_by_id_returns_item_and_404_semantics` — after an upsert, GET `/api/activity/{id}` returns `200` with the item; GET `/api/activity/999999` returns `200` with JSON `null` (the handler returns `Option`; a missing id is `Ok(None)` → `null`, not a 404 — assert the body is `null`).
4. `ranking_orders_two_rows_exponential` — seed two rows directly via `common` helpers (or the upsert): row A `impact=10`, `merged_at = now() - 800 days`; row B `impact=3`, `merged_at = now() - 1 day`. With `RankingStrategy::Exponential` (`half_life=365`), B must rank before A (A's score ≈ `10 * 0.5^(800/365) ≈ 2.2`; B ≈ `3 * 0.5^(1/365) ≈ 2.99`). Assert the ranked list's first element is B.
5. `ranking_orders_two_rows_linear` — same two rows, `RankingStrategy::Linear` (`window=730`): A's age (800d) exceeds the window → score `0`; B ≈ `3 * (1 - 1/730) ≈ 2.99`. Assert B first, and A's `score` is `0.0`.
6. `ranking_orders_two_rows_pure` — same two rows, `RankingStrategy::Pure`: score = impact, so A (`impact=10`) ranks first; tiebreak is reference_date DESC (irrelevant here). Assert A first.
7. `admin_requires_bearer_token` — POST `/api/admin/activity` with NO `Authorization` header; assert `401 UNAUTHORIZED`.
8. `patch_updates_impact_and_featured` — upsert, then PATCH `/api/admin/activity/{id}` with `{"impact": 7, "featured": true}` + Bearer; assert `200`; assert DB `impact=7 AND featured=true`.
9. `delete_removes_row` — upsert, DELETE `/api/admin/activity/{id}` + Bearer; assert `200`; assert `COUNT(*) = 0`; DELETE again → `404` (or whatever not-found status the handler returns — assert it matches the handler).

For the ranking tests (4–6), the cleanest approach is to build a fresh `app_state` per strategy by parameterizing the `app_state` helper to take a `RankingStrategy` (the actor snapshots strategy at spawn). Seed rows with `sqlx::query("INSERT INTO activity_items (...) VALUES (...)")` using explicit `now() - interval '800 days'` so dates are deterministic.

### 14. Build + lint gates

```bash
cargo build -p plinth-server --features brick-activity
cargo clippy -p plinth-server --features brick-activity --all-targets -- -D warnings
cargo test  -p plinth-server --features brick-activity --test activity_brick
# Also confirm the workspace default build still compiles activity in (it's in default now):
cargo clippy --workspace --all-targets -- -D warnings
```

`nix flake check` runs clippy with `--deny warnings` over the workspace and `cargo test --workspace --all-targets` against a sandbox Postgres (with `pgvector`); the new test runs there automatically once it compiles and the feature is in the default set.

## Acceptance criteria

- [ ] `crates/server/src/bricks/activity/` contains `mod.rs`, `migrations.rs`, `admin.rs`, `api.rs`, `cache.rs`, `ranking.rs`, all under `#[cfg(feature = "brick-activity")]` gating via `bricks/mod.rs`.
- [ ] `cargo build -p plinth-server --features brick-activity` succeeds.
- [ ] `cargo clippy -p plinth-server --features brick-activity --all-targets -- -D warnings` produces **zero** warnings.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` is clean (activity is in the default feature set).
- [ ] `POST /api/admin/activity` with a valid body + `Authorization: Bearer test_secret` returns **200**; the same request with no Authorization header returns **401**.
- [ ] `DELETE /api/admin/activity/{id}` with Bearer returns **200** on an existing id and **404** (or the handler's documented not-found status) on a missing id.
- [ ] `PATCH /api/admin/activity/{id}` with `{"impact":7,"featured":true}` + Bearer returns **200** and the DB row reflects `impact=7, featured=true`.
- [ ] `GET /api/activity` (unauthenticated) returns **200** with a JSON array of `ActivityListItem` (each carrying a `score`), and includes a just-upserted item.
- [ ] `GET /api/activity/{id}` returns **200** with the item for a valid id and **200** + JSON `null` for a missing id.
- [ ] Integration test file `crates/server/tests/activity_brick.rs` exists and these named tests pass under `cargo test -p plinth-server --features brick-activity --test activity_brick`: `admin_upsert_then_public_get_returns_it`, `admin_upsert_is_idempotent_on_natural_key`, `public_get_by_id_returns_item_and_404_semantics`, `ranking_orders_two_rows_exponential`, `ranking_orders_two_rows_linear`, `ranking_orders_two_rows_pure`, `admin_requires_bearer_token`, `patch_updates_impact_and_featured`, `delete_removes_row`.
- [ ] The ranking tests prove ordering: exponential → recent-low-impact (B) before old-high-impact (A); linear → B before A with A's `score == 0.0`; pure → A before B.
- [ ] `cache.rs` contains the literal `PHASE 04 SEAM` comment block and **no** refresh / `reqwest` / `plinth-forge` references (verify: `rg "plinth-forge|reqwest|refreshing" crates/server/src/bricks/activity` returns nothing).
- [ ] `[ranking]` config (`strategy`, `half_life_days`, `window_days`) parses from `plinth.toml` with the documented defaults (`exponential`, `365`, `730`); `cargo test -p plinth-shared` config tests still pass (empty/partial TOML).
- [ ] The brick is registered: `rg "ActivityBrick" crates/server/src/bricks/mod.rs` shows both the `pub mod activity;` and the `enabled_bricks()` push; `rg "activity_cache" crates/server/src/main.rs crates/server/src/lib.rs` shows the spawn, the route blocks, and the `AppState` field.

## Files likely touched

New files (under `crates/server/`):
- `src/bricks/activity/mod.rs`
- `src/bricks/activity/migrations.rs`
- `src/bricks/activity/ranking.rs`
- `src/bricks/activity/cache.rs`
- `src/bricks/activity/admin.rs`
- `src/bricks/activity/api.rs`
- `tests/activity_brick.rs`

Edited files:
- `crates/server/src/bricks/mod.rs` — `pub mod activity;` + `enabled_bricks()` push (cfg-gated).
- `crates/server/src/services/db.rs` — `upsert_activity_item`, `delete_activity_item`, `patch_activity_item` (cfg-gated).
- `crates/server/src/services/rows.rs` — `activity_item`, `activity_list_item` decoders (cfg-gated).
- `crates/server/src/lib.rs` — `AppState.activity_cache` field (cfg-gated).
- `crates/server/src/main.rs` — actor spawn + admin/public route blocks + `AppState` literal field (cfg-gated).
- `crates/server/Cargo.toml` — `brick-activity` feature + add to `default`.
- `Cargo.toml` (root) — `bin-features` / `lib-features` include `brick-activity`.
- `crates/shared/src/toml_config.rs` — `RankingConfig` + `PlinthConfig.ranking` (if Phase 01 did not add it).
- `crates/client/Cargo.toml` — `brick-activity` feature (only if Phase 01 left it missing; required for the server feature chain `plinth-client/brick-activity` to resolve).
- `plinth.toml` (optional) — example `[ranking]` section.

NOT touched (other phases): `crates/forge/**` (02), `crates/cli/**` (05), `crates/client/src/**` (06), `crates/server/src/api/feeds.rs` + `actors/vector_search.rs` + `api/search.rs` (07).

## Pitfalls

- **Treating the `Brick` trait methods as load-bearing.** Symptom: routes 404 even though `public_routes()` is implemented. Cause: the trait's `public_routes`/`admin_routes`/`feed_routes` have **no callers** — `enabled_bricks()` is used only by the migration counter. Recovery: wire routes by hand in `main.rs` `#[cfg]` blocks (step 12), exactly like portfolio.
- **Putting DDL in `migrations.rs`.** Symptom: table never created / `up` ignored. Cause: the runner runs only the embedded `0006_activity.sql` and ignores `BrickMigration.up`. Recovery: `migrations.rs` carries only the metadata tuple; the DDL is Phase 01's `.sql`.
- **Missing `schema_migrations` ledger insert.** Symptom: `migration_status()` reports a missing/mismatched activity migration even though the table exists. Cause: the `.sql` file did not `INSERT INTO schema_migrations`. Recovery: ensure `0006_activity.sql` ends with the `('activity', 1, 'initial_activity_schema')` insert (step 4).
- **Wrong `ON CONFLICT` target.** Symptom: duplicate rows for the same PR, or upsert fails with "no unique constraint matching". Cause: conflicting on `url` instead of the natural key, or a typo in the five columns. Recovery: conflict on `(forge, repo_owner, repo_name, kind, number)` — the brief's upsert key — matching the table's `UNIQUE (...)` constraint.
- **Integer truncation in ranking SQL.** Symptom: every exponential/linear score is `0` or wrong. Cause: `impact` is `SMALLINT`; multiplying by a `power(...)` double without casting truncates. Recovery: cast `impact::float8` (step 5).
- **Divide-by-zero on misconfigured half-life/window.** Symptom: `division by zero` SQL error when `half_life_days = 0`. Cause: raw `age / $1`. Recovery: `greatest($1, 0.000001)` floor (step 5).
- **SQL injection / dynamic strategy string.** Symptom: clippy/security concern or malformed SQL. Cause: formatting a user-supplied strategy string into SQL. Recovery: the strategy is a typed `RankingStrategy` enum → `match` to a fixed `&'static str` fragment; numeric params are **bound** (`$1`), never formatted.
- **`vector(384)` bind mismatch.** Symptom: `expected vector, got ...` or dimension errors. Cause: binding `Vec<f32>` directly or a wrong-length vector. Recovery: convert with the existing `vector_or_none` (`Option<Vec<f32>> -> Option<pgvector::Vector>`) in `services/db.rs`; the column is fixed at 384 dims. Refresh must not wipe it — use `COALESCE(EXCLUDED.embedding, activity_items.embedding)` (step 8a).
- **`AppState` `FromRef` derive.** Symptom: compile error adding the field. Cause: `AppState` does NOT derive `FromRef` (it can't handle `#[cfg]` fields); fields are hand-gated. Recovery: just add the `#[cfg(feature = "brick-activity")] pub activity_cache: ...` field; do not add a derive.
- **`patch` route method not imported.** Symptom: `cannot find function patch in this scope`. Cause: `main.rs` imports `get/post/put/delete` but not `patch`. Recovery: add `patch` to the `use axum::routing::{...}` import.
- **Forgetting `bin-features`/`lib-features`.** Symptom: brick compiles in `cargo build` but the deployed leptos binary/WASM does not include it. Cause: cargo-leptos uses the root `Cargo.toml` feature lists, not crate defaults. Recovery: add `brick-activity` to both lists (step 1).
- **`#[sqlx::test]` migrations path.** Symptom: test fails with "relation activity_items does not exist". Cause: wrong `migrations = ...` path. Recovery: in `crates/server/tests/`, use `#[sqlx::test(migrations = "./migrations")]` (relative to the server crate root).
- **Caching a `featured`/`limit`-filtered list as the full list.** Symptom: a `?featured=true` request poisons the cache so later unfiltered requests return only featured items. Cause: storing a filtered query result in `ranked_list_cache`. Recovery: cache only the unfiltered list; apply `featured`/`limit` on top (step 6).

## Reference

Sequencing context only (do **not** read these for execution content — everything needed is inlined above):

- **Phase 01 (shared-types-and-migration)** must have landed first: it provides the `plinth-shared` DTOs (`ActivityItem`, `ActivityListItem` with `score`, `PublishActivityRequest`, `Forge`, `ActivityKind`, `ActivityState`, `RankingStrategy`) and `crates/server/migrations/0006_activity.sql`. See `./01-shared-types-and-migration.md`. Verify with step 0.
- **Phase 04 (lazy-refresh-actor)** extends `cache.rs` (the seam in step 7) and `api.rs`/`main.rs`; it adds the TTL + single-flight stale-while-revalidate refresh and `plinth-forge` usage. See `./04-lazy-refresh-actor.md`. Do not implement any of it here.
- **Phase 07 (feed-and-search)** adds `/feeds/activity.xml` and the pgvector search union over `activity_items` (reading the `embedding` written here). See `./07-feed-and-search.md`. Out of scope here.
- Design brief: the locked schema, endpoints, ranking formulas, and config keys are reproduced inline above; the brief is the source of truth if any inlined detail is ambiguous.

Pattern sources in the live repo to copy (read these in the repo, not a sibling phase doc):
- `crates/server/src/bricks/portfolio/{mod.rs,admin.rs,api.rs,cache.rs,migrations.rs}` — the exact brick shape to mirror.
- `crates/server/src/services/db.rs` (`upsert_portfolio_item`, `vector_or_none`) and `crates/server/src/services/rows.rs` (decoders).
- `crates/server/src/main.rs` (portfolio `#[cfg]` spawn + route blocks), `crates/server/src/lib.rs` (`AppState` fields).
- `crates/server/tests/portfolio_publish.rs` (the integration-test harness: `app_state`, `test_app`, `oneshot`, auth-failure test).
- `crates/shared/src/toml_config.rs` (`SearchConfig`/`ContentConfig` serde-defaults pattern for `RankingConfig`).
