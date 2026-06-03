# Phase 04 — Lazy stale-while-revalidate refresh in the cache actor

> **Recommended Codex model: GPT 5.5 high**
>
> This phase is pure concurrency-correctness engineering inside a Kameo actor: serve cached
> data synchronously while a *single-flighted* background refresh re-pulls forge metadata off
> the request path, never blocking render, never stampeding a rate-limited forge, and never
> losing a panic in a spawned task. The failure modes are subtle and silent — a too-small model
> will plausibly ship a refresh that (a) blocks the render path by `await`-ing the network
> inside the read handler, (b) lets N concurrent post-expiry reads each fire their own refresh
> (a thundering herd against a 60-req/hour GitHub limit), or (c) swallows a panic in a detached
> `tokio::spawn` so the `refreshing` flag is never cleared and refresh wedges forever. Getting
> the actor mailbox semantics, the single-flight latch, the off-path spawn, and the
> panic-recovery all simultaneously right requires careful reasoning about message ordering and
> task lifetimes — hence the high tier.

## Working tree

- cwd = `/data/nvme0/can/Projects/solo/plinth` (the plinth repo).
- **Phase 03 must land first.** This phase edits `crates/server/src/bricks/activity/cache.rs`,
  which Phase 03 creates (the `ActivityCache` actor skeleton, the `GetRankedActivity` /
  `GetActivityItem` ranked-read messages, the `activity_items` table, and the row decoder).
  If `crates/server/src/bricks/activity/cache.rs` does not exist, **stop** — rebase onto the
  branch that contains Phase 03 before doing anything else.
- **Serialize with Phase 07.** Phase 07 (feed + search union) also edits this brick
  (`bricks/activity/mod.rs`, `bricks/activity/api.rs`) and `crates/server/src/main.rs` route
  registration. Whichever of 04/07 lands second MUST `git pull --rebase` (or rebase its branch
  onto the other) BEFORE starting, to avoid clobbering the other's edits to those shared files.
  This phase's *new* file (`refresh.rs`) is conflict-free; the contention is only in
  `cache.rs` / `mod.rs` / `main.rs`. Before you begin, run
  `git fetch && git log --oneline -5` and confirm Phase 03 (and, if it landed first, Phase 07)
  are present.
- Phases 02 (the `plinth-forge` crate) and 03 (the activity brick + migration) are hard
  prerequisites — see the dependency note at the top of the brief: `04 depends on: 02, 03`.

## Goal

This phase succeeds when any **public read** of activity data (`GET /api/activity`,
`GET /api/activity/{id}`, and the feed/home-strip reads that go through the same actor) returns
the currently-cached, ranked entries **immediately and synchronously**, and — if any served
entry's `fetched_at` is older than the configured TTL (default 1 hour) — the `ActivityCache`
Kameo actor fires **exactly one** asynchronous background refresh (regardless of how many
concurrent reads observe the staleness), where that refresh re-pulls each entry's forge
metadata via `plinth-forge`, updates the DB row (`state`, `merged_at`, `closed_at`,
`additions`, `deletions`, `comments_count`, `repo_stars`, `fetched_at`) and the in-memory
cache, **without re-embedding**, and where a forge error during refresh leaves the prior data
intact, keeps the endpoint returning `200`, logs the failure, and applies a backoff so a
rate-limited forge is never thrashed. Refresh must never run on the request path.

## Why this matters now

Phase 03 ships the activity brick with persistence, admin upsert, ranked public reads, and the
cache actor — but its data is a static snapshot captured whenever the CLI last ran `activity
add` (Phase 05). Real contributions change state over time: a PR gets merged, an issue closes,
diff stats and star counts drift. Without this phase the `/activity` surfaces show
permanently-stale "open" PRs that were merged months ago, and the *ranking* (computed in SQL at
read time from `merged_at`/`closed_at`/`created_at`) ranks them on stale reference dates. This
phase is the freshness engine (locked decision #2). Deferring it means either (a) shipping
stale data indefinitely, or (b) a worse design that re-fetches on the request path and blocks
render or hammers the forge. It sits squarely in Wave 2 and is the only phase that owns the
freshness mechanism; every other surface (pages, feed, search) just reads the cache this phase
keeps fresh.

## Out of scope

- **Ranking math.** Do NOT change the score SQL (`exponential`/`linear`/`pure`) or the
  `ORDER BY score DESC, reference_date DESC` clause — that is Phase 03's `ranking.rs`. Refresh
  updates the *inputs* (`merged_at` etc.); the score recomputes itself at read time.
- **The `plinth-forge` crate internals.** Phase 02 owns the `ForgeClient` trait,
  `GitHubClient`, `CodebergClient`, the `FetchedActivity` DTO, and retry/backoff *inside* the
  HTTP layer. This phase *calls* `plinth-forge`; it does not modify it. (You MAY add a
  thin server-side retry/backoff wrapper around the call if Phase 02's client does not already
  surface a typed rate-limit error — see Pitfalls P4.)
- **CLI** (Phase 05), **frontend** (Phase 06), **feed + search union** (Phase 07). Do not touch
  `crates/cli`, `crates/client`, `crates/server/src/api/feeds.rs`, or
  `crates/server/src/actors/vector_search.rs`.
- **Re-embedding.** Refresh MUST NOT regenerate the `embedding vector(384)` column. Title/body
  rarely change and the server does not run fastembed (only the CLI does). Leave `embedding`
  untouched in the refresh UPDATE.
- **Admin handlers / upsert.** `bricks/activity/admin.rs` and the `services/db.rs` upsert are
  Phase 03's. Refresh writes a *narrow* UPDATE of forge-derived columns only, not the full
  upsert.
- **Schema changes.** The `activity_items` table already has `fetched_at TIMESTAMPTZ NOT NULL`
  (added by migration `0006_activity.sql` in Phase 03). Do NOT add a migration. If you need a
  per-row backoff timestamp, keep it **in actor memory**, not in a new column.

## Plan

> All snippets mirror the existing portfolio cache actor
> (`/data/nvme0/can/Projects/solo/plinth/crates/server/src/bricks/portfolio/cache.rs`) and the
> VectorSearch actor's `spawn_blocking` discipline
> (`/data/nvme0/can/Projects/solo/plinth/crates/server/src/actors/vector_search.rs`). Kameo is
> pinned at `0.19` (`Cargo.toml:38`). Actors derive `#[derive(Actor)]`; messages are
> `impl Message<M> for ActivityCache` with `type Reply` + `async fn handle`; an actor messages
> *itself* via `ctx.actor_ref()` and fires-and-forgets with `.tell(..).await` (no reply wait).

### Step 0 — Confirm the Phase 03 baseline you are extending

Read these (they MUST exist; if not, rebase per Working tree):

- `/data/nvme0/can/Projects/solo/plinth/crates/server/src/bricks/activity/cache.rs` — the
  `ActivityCache` actor with at minimum: `db: PlinthDb` (= `sqlx::PgPool`), an in-memory
  `ranked_list_cache` store of ranked items, a `cache_populated_at: Option<Instant>` (or
  equivalent), the
  `GetRankedActivity { limit, featured_only }` (list) and `GetActivityItem(id)` messages, and an
  `ActivityInvalidateCache` message. You will ADD fields and messages here.
- `/data/nvme0/can/Projects/solo/plinth/crates/server/src/services/rows.rs` — the
  `activity_item` / `activity_list_item` row decoder(s) (cfg-gated `brick-activity`). Refresh
  re-reads rows after the UPDATE through these.
- `/data/nvme0/can/Projects/solo/plinth/crates/shared/src/` — `Forge` enum
  `{GitHub, Codeberg}`, `ActivityKind {PullRequest, Issue}`, `ActivityState {Open, Closed,
  Merged}`, and the `FetchedActivity` DTO (from Phase 01/02). The refresh maps a
  `FetchedActivity` back onto an `activity_items` row.
- `/data/nvme0/can/Projects/solo/plinth/crates/server/src/lib.rs:34-52` — `AppState` already
  has `#[cfg(feature = "brick-activity")] pub activity_cache: ActorRef<...>` (Phase 03). No
  change needed unless missing.

### Step 1 — Add the TTL + forge config keys

The config split is two structs kept in sync
(`/data/nvme0/can/Projects/solo/plinth/crates/shared/src/toml_config.rs` for the server-side
`PlinthConfig`; `config.rs` for the client-safe `SiteConfig`). TTL and forge base URLs are
**server-only tuning** — they go ONLY in `toml_config.rs`, never in `to_site_config()`. Forge
tokens are **env-only secrets** (`GITHUB_TOKEN` / `CODEBERG_TOKEN`) and are NEVER toml keys.

Phase 03 already added a `[ranking]` section. **TTL belongs with the freshness model, not
ranking** — add it to the `[forge]` section, or, if no `[forge]` section exists yet, create it
here. In `/data/nvme0/can/Projects/solo/plinth/crates/shared/src/toml_config.rs`, mirror the
`SearchConfig` defaults idiom (every field has `#[serde(default = "fn")]` plus a hand-written
`impl Default` calling the same `default_*()` free fns). The canonical `ForgeConfig` carries the
TTL, the backoff, and the two overridable forge base URLs — and NO token fields:

```rust
/// [forge] section — freshness + base URLs for activity refresh.
/// Tokens are env-only (GITHUB_TOKEN / CODEBERG_TOKEN), never toml keys.
#[derive(Debug, Clone, Deserialize)]
pub struct ForgeConfig {
    /// Stale-while-revalidate TTL in seconds. Entries whose `fetched_at` is
    /// older than this trigger a single background refresh on the next read.
    #[serde(default = "default_refresh_ttl_secs")]
    pub refresh_ttl_secs: u64,
    /// Backoff after a failed refresh, in seconds. No refresh is attempted
    /// again until this window elapses (prevents thrashing a rate-limited forge).
    #[serde(default = "default_refresh_backoff_secs")]
    pub refresh_backoff_secs: u64,
    /// GitHub REST API base (overridable so tests can point at a mock server).
    #[serde(default = "default_github_base_url")]
    pub github_base_url: String,
    /// Codeberg/Forgejo API base (overridable so tests can point at a mock server).
    #[serde(default = "default_codeberg_base_url")]
    pub codeberg_base_url: String,
}

impl Default for ForgeConfig {
    fn default() -> Self {
        Self {
            refresh_ttl_secs: default_refresh_ttl_secs(),
            refresh_backoff_secs: default_refresh_backoff_secs(),
            github_base_url: default_github_base_url(),
            codeberg_base_url: default_codeberg_base_url(),
        }
    }
}

fn default_refresh_ttl_secs() -> u64 { 3600 }       // 1 hour (locked default)
fn default_refresh_backoff_secs() -> u64 { 900 }    // 15 minutes
fn default_github_base_url() -> String { "https://api.github.com".to_string() }
fn default_codeberg_base_url() -> String { "https://codeberg.org/api/v1".to_string() }
```

Wire it into `PlinthConfig` (the top-level struct, near `search`/`content`/`feeds`):

```rust
#[serde(default)]
pub forge: ForgeConfig,
```

(If Phase 02/03 already added `ForgeConfig` with the base URLs, just ADD the two
`refresh_ttl_secs` / `refresh_backoff_secs` fields + their defaults — do not duplicate the
struct, and do NOT add token fields.) Tokens are read at spawn time directly from the
environment (`std::env::var("GITHUB_TOKEN")` / `std::env::var("CODEBERG_TOKEN")` — see Step 4),
NOT via `apply_env_overrides()` onto a config field.

Add to `/data/nvme0/can/Projects/solo/plinth/plinth.toml`:

```toml
[forge]
refresh_ttl_secs = 3600
refresh_backoff_secs = 900
github_base_url = "https://api.github.com"
codeberg_base_url = "https://codeberg.org/api/v1"
# tokens via GITHUB_TOKEN / CODEBERG_TOKEN env vars, not here
```

`ForgeConfig` is read at runtime via `state.config.forge.*` (`AppState.config: PlinthConfig`,
`crates/server/src/lib.rs:40`). It is threaded into the actor at spawn time (Step 4).

### Step 2 — Create `bricks/activity/refresh.rs` (the off-path refresh worker)

This is a NEW, conflict-free file:
`/data/nvme0/can/Projects/solo/plinth/crates/server/src/bricks/activity/refresh.rs`. It holds a
**free async function** that does the actual forge re-pull + DB UPDATE. Keeping it free
(not a method on the actor) is what lets us `tokio::spawn` it off the actor's mailbox so the
refresh runs concurrently with — never serialized behind — subsequent reads.

```rust
//! Background refresh worker for the activity cache.
//!
//! Runs OFF the actor mailbox (spawned as a detached task) so it never blocks
//! reads. Re-pulls forge metadata for each item via `plinth-forge`, writes a
//! NARROW update of the forge-derived columns (NOT a full upsert, and NEVER the
//! embedding), and returns the fresh, re-ranked rows for the actor to swap in.

use std::sync::Arc;
use crate::PlinthDb;
use plinth_forge::{ActivityRef, ForgeClient, ForgeError};   // from Phase 02
use plinth_shared::{ActivityListItem, Forge, ActivityKind};
use plinth_shared::toml_config::RankingConfig;              // Phase 03's ranking config
use tracing::{info, warn, instrument};

/// Identifies one row to refresh by its natural forge key.
#[derive(Clone)]
pub struct RefreshTarget {
    pub id: i64,
    pub forge: Forge,
    pub repo_owner: String,
    pub repo_name: String,
    pub kind: ActivityKind,
    pub number: i32,
}

/// Outcome of a single refresh sweep, reported back to the actor.
pub enum RefreshOutcome {
    /// All targets refreshed; the actor should swap the re-ranked list into
    /// `ranked_list_cache`, clear the `refreshing` flag, and mark the cache
    /// freshly populated. These are the SQL-ranked `ActivityListItem`s (carrying
    /// the computed `score`) — the exact shape `GetRankedActivity` replies with.
    Refreshed { items: Vec<ActivityListItem> },
    /// The sweep failed (forge/network/DB). The actor MUST keep its prior data,
    /// clear the `refreshing` flag, and start a backoff window.
    Failed { reason: String },
}

/// Re-pull every target from its forge and write a narrow UPDATE per row.
///
/// `client` is the canonical `Arc<dyn ForgeClient>` (production: a `ForgeRouter`
/// that dispatches by `ActivityRef::forge`; tests: a mock). The ONLY fetch
/// entrypoint is `client.fetch(&ActivityRef)` — there is no `fetch_one`.
/// Returns the freshly-read rows on success.
#[instrument(skip(db, client, ranking, targets), fields(n = targets.len()))]
pub async fn run_refresh(
    db: PlinthDb,
    client: Arc<dyn ForgeClient + Send + Sync>,
    ranking: RankingConfig,
    targets: Vec<RefreshTarget>,
) -> RefreshOutcome {
    let mut rate_limited = false;
    for t in &targets {
        // Build the canonical ActivityRef; the router dispatches by `forge`.
        let r = ActivityRef {
            forge: t.forge,
            owner: t.repo_owner.clone(),
            repo: t.repo_name.clone(),
            kind: t.kind,
            number: t.number,
        };
        let fetched = match client.fetch(&r).await {
            Ok(f) => f,
            // 404/410 — deleted upstream: do not fail the whole sweep, skip.
            Err(ForgeError::NotFound { .. }) => {
                warn!(id = t.id, "activity refresh: upstream gone (404/410), keeping last-known");
                continue;
            }
            // Rate-limited: STOP the sweep, signal backoff. Do NOT keep hammering.
            Err(ForgeError::RateLimited { .. }) => {
                warn!(id = t.id, "activity refresh: rate limited, backing off");
                rate_limited = true;
                break;
            }
            Err(e) => {
                // Network/parse/http: log, abort sweep, keep stale data + backoff.
                return RefreshOutcome::Failed { reason: format!("forge fetch failed: {e}") };
            }
        };

        // NARROW update: forge-derived columns + fetched_at. NEVER embedding,
        // NEVER title/body (rarely change; would require re-embedding).
        let res = sqlx::query(
            r#"
            UPDATE activity_items
            SET state = $2,
                merged_at = $3,
                closed_at = $4,
                additions = $5,
                deletions = $6,
                comments_count = $7,
                repo_stars = $8,
                labels = $9,
                fetched_at = now()
            WHERE id = $1
            "#,
        )
        .bind(t.id)
        .bind(fetched.state.as_str())   // 'open' | 'closed' | 'merged'
        .bind(fetched.merged_at)
        .bind(fetched.closed_at)
        .bind(fetched.additions)
        .bind(fetched.deletions)
        .bind(fetched.comments_count)
        .bind(fetched.repo_stars)
        .bind(&fetched.labels)
        .execute(&db)
        .await;

        if let Err(e) = res {
            return RefreshOutcome::Failed { reason: format!("refresh UPDATE failed: {e}") };
        }
    }

    if rate_limited {
        // We made partial progress but hit a limit; treat as a soft-fail so the
        // actor backs off, but the partial DB writes already landed.
        return RefreshOutcome::Failed { reason: "rate limited mid-sweep".into() };
    }

    // Re-read the fresh, RE-RANKED list so the actor can swap it in. Reuse the
    // exact ranked query the actor uses for GetRankedActivity (Phase 03's
    // ranking.rs score expression). Do NOT re-rank here in Rust.
    match reread_ranked(&db, &ranking).await {
        Ok(items) => {
            info!(n = targets.len(), "activity refresh complete");
            RefreshOutcome::Refreshed { items }
        }
        Err(e) => RefreshOutcome::Failed { reason: format!("re-read after refresh failed: {e}") },
    }
}

/// Re-run the ranked SELECT (identical to the actor's GetRankedActivity query) by
/// calling the SHARED, free, `pub` helper Phase 03 exposes in `ranking.rs`
/// (`query_ranked_list`) — so the score expression + ORDER BY live in exactly one
/// place and never drift from what the cache serves. `featured_only = false`,
/// `limit = None`: re-read the FULL ranked set so the actor can swap in the whole
/// list. Returns `ActivityListItem`s (decoded via `rows::activity_list_item`).
async fn reread_ranked(
    db: &PlinthDb,
    ranking: &RankingConfig,
) -> Result<Vec<ActivityListItem>, sqlx::Error> {
    crate::bricks::activity::ranking::query_ranked_list(db, ranking, false, None).await
}
```

> The forge surface is canonical (Phase 02 owns it; see `crates/forge/src/lib.rs`): the trait is
> `trait ForgeClient: Send + Sync` with the SINGLE entrypoint `async fn fetch(&self, r:
> &ActivityRef) -> ForgeResult<FetchedActivity>` (there is NO `fetch_one`/`fetch_pull_request`/
> `fetch_issue`). `ActivityRef { forge, owner, repo, kind, number }` carries `kind`, and PR-vs-issue
> routing is INTERNAL to each client — you just pass `kind` through on the ref. `ForgeError` is a
> `thiserror` enum with all-struct variants (`NotFound { forge, url, status }`,
> `RateLimited { forge, retry_after }`, `Http { forge, status, body }`, `Network(String)`,
> `Decode(String)`), so error matches use STRUCT patterns
> (`Err(ForgeError::NotFound { .. })` / `Err(ForgeError::RateLimited { .. })`). Per the forge-API
> facts: GitHub PRs MUST be fetched via `/pulls/{n}` (the `/issues/{n}` payload lacks
> `merged`/`additions`/`deletions`); Phase 02's client already handles this internally by `kind`.

### Step 3 — Extend `ActivityCache` with single-flight + TTL-on-read (`cache.rs`)

Edit `/data/nvme0/can/Projects/solo/plinth/crates/server/src/bricks/activity/cache.rs`. Add the
freshness state and the single-flight latch to the actor struct:

```rust
use std::sync::Arc;
use std::time::{Duration, Instant};
use kameo::Actor;
use kameo::message::{Context, Message};
use kameo::actor::ActorRef;
use plinth_forge::ForgeClient;
use crate::bricks::activity::refresh::{self, RefreshOutcome, RefreshTarget};

#[derive(Actor)]
pub struct ActivityCache {
    db: PlinthDb,   // = sqlx::PgPool (crates/server/src/lib.rs)
    /// Phase 03's in-memory ranked store (canonical field name).
    ranked_list_cache: Vec<ActivityListItem>,
    // ... plus Phase 03's per-id map for GetActivityItem ...
    cache_populated_at: Option<Instant>,
    /// SINGLE-FLIGHT LATCH: true while a background refresh is in flight.
    /// Guarantees only one refresh runs at a time — no stampede.
    refreshing: bool,
    /// Set when a refresh fails; suppresses new refreshes until it elapses.
    backoff_until: Option<Instant>,
    /// TTL after which served data is considered stale (from [forge] config).
    ttl: Duration,
    backoff: Duration,
    /// The canonical single forge client, built once at spawn (production: a
    /// `ForgeRouter` that dispatches by forge; tests: an injected mock).
    forge_client: Arc<dyn ForgeClient + Send + Sync>,
}
```

Add the staleness check + the trigger. Because the actor processes messages one at a time, the
`refreshing` flag is checked-and-set **inside a single message handler with no `.await` between
the check and the set** — that is what makes the single-flight race-free without any extra lock:

```rust
impl ActivityCache {
    /// True if any *currently cached* entry's fetched_at is older than the TTL.
    /// Cheap: compares against the actor's own population timestamp. (If you
    /// track per-row fetched_at, compute max-age over the cached rows instead.)
    fn is_stale(&self) -> bool {
        self.cache_populated_at
            .is_some_and(|t| t.elapsed() > self.ttl)
    }

    fn in_backoff(&self) -> bool {
        self.backoff_until.is_some_and(|t| Instant::now() < t)
    }

    /// Fire a single-flighted background refresh if stale, not already
    /// refreshing, and not in a backoff window. Returns immediately; the
    /// refresh runs OFF the mailbox via tokio::spawn and reports back with a
    /// RefreshDone message.
    ///
    /// Takes a CONCRETE `me: ActorRef<ActivityCache>` (obtained in the handler
    /// via `ctx.actor_ref().clone()`), NOT a generic `Context<Self, impl Send>`
    /// — both read handlers have different `Reply` types, so a concrete actor
    /// ref sidesteps the generic-reply borrow-checker friction entirely.
    fn maybe_trigger_refresh(&mut self, me: ActorRef<ActivityCache>) {
        if !self.is_stale() || self.refreshing || self.in_backoff() {
            return;
        }
        self.refreshing = true;                 // latch BEFORE any await — race-free
        let db = self.db.clone();
        let forge_client = Arc::clone(&self.forge_client);
        let ranking = self.ranking.clone();     // Phase 03's RankingConfig field
        // Build targets from the current cache (id + natural forge key).
        let targets: Vec<RefreshTarget> = self.refresh_targets();

        tokio::spawn(async move {
            // PANIC-GUARD: an inner tokio::spawn(...).await converts a panic or
            // cancel in the refresh path into RefreshOutcome::Failed, so a
            // RefreshDone is ALWAYS delivered and the `refreshing` latch is
            // cleared in ALL cases (the actor would otherwise wedge forever).
            let outcome = match tokio::spawn(async move {
                refresh::run_refresh(db, forge_client, ranking, targets).await
            })
            .await
            {
                Ok(o) => o,
                Err(join_err) => RefreshOutcome::Failed {
                    reason: format!("refresh task panicked/aborted: {join_err}"),
                },
            };
            // tell() = fire-and-forget; we don't await a reply.
            let _ = me.tell(RefreshDone(outcome)).await;
        });
    }
}
```

Call `self.maybe_trigger_refresh(me)` at the **end** of each read handler, AFTER the cached
data has been cloned for the reply — so the reply value is already computed and the trigger
cannot delay it. Reuse Phase 03's EXACT message `GetRankedActivity { limit, featured_only }`
verbatim (do NOT invent a `GetActivityItems`):

```rust
impl Message<GetRankedActivity> for ActivityCache {
    // Keep Phase 03's canonical reply type verbatim — the live portfolio cache
    // idiom is Result<_, String>, which callers flatten via `.map_err(...)?`.
    // (Phase 04 only ADDS refresh behaviour; it does NOT change the reply type.)
    type Reply = Result<Vec<ActivityListItem>, String>;
    async fn handle(&mut self, msg: GetRankedActivity, ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        // 1. Serve cached/ranked data IMMEDIATELY (Phase 03's existing logic).
        let items = self.read_ranked(msg.limit, msg.featured_only);   // the clone to reply with
        // 2. Grab a concrete actor ref BEFORE the trigger (sidesteps generic-reply friction).
        let me = ctx.actor_ref().clone();
        // 3. THEN consider a background refresh — never blocks this reply.
        self.maybe_trigger_refresh(me);
        Ok(items)
    }
}
```

The `GetActivityItem(i64)` handler (`Reply = Result<Option<ActivityItem>, String>`, per Phase 03)
ends the same way: `let me = ctx.actor_ref().clone(); self.maybe_trigger_refresh(me);` before
returning its `Ok(item)` reply.

> Because `maybe_trigger_refresh` takes a concrete `ActorRef<ActivityCache>` (not `&Context`),
> it is callable identically from both `GetRankedActivity` (`Reply = Vec<ActivityListItem>`) and
> `GetActivityItem(i64)` (`Reply = Option<ActivityItem>`) with no generic-reply borrow-checker
> friction. Obtain the ref with `ctx.actor_ref().clone()` at the tail of each handler.

Add the `RefreshDone` message — this is how the spawned task hands the result back onto the
mailbox so the cache swap happens *inside* the actor (no shared-mutable-state, no lock):

```rust
pub struct RefreshDone(pub RefreshOutcome);

impl Message<RefreshDone> for ActivityCache {
    type Reply = ();
    async fn handle(&mut self, msg: RefreshDone, _ctx: &mut Context<Self, Self::Reply>) -> Self::Reply {
        self.refreshing = false;                 // ALWAYS clear the latch
        match msg.0 {
            RefreshOutcome::Refreshed { items } => {
                self.ranked_list_cache = items;  // swap in the re-ranked list
                self.cache_populated_at = Some(Instant::now());
                self.backoff_until = None;
            }
            RefreshOutcome::Failed { reason } => {
                tracing::warn!(%reason, "activity refresh failed; keeping stale data + backing off");
                self.backoff_until = Some(Instant::now() + self.backoff);
                // DO NOT clear or mutate the cache — stale data stays served.
            }
        }
    }
}
```

Build `refresh_targets()` over Phase 03's existing store shape: it reads `id`, `forge`,
`repo_owner`, `repo_name`, `kind`, `number` from each cached row (the per-id/natural-key map).
On success, the swap is a plain `self.ranked_list_cache = items;` (the re-ranked
`Vec<ActivityListItem>` from `reread_ranked`) — no separate `swap_in` helper is required.

### Step 4 — Build the forge clients at spawn and thread config in (`main.rs`)

Edit `/data/nvme0/can/Projects/solo/plinth/crates/server/src/main.rs` where Phase 03 spawns the
actor (mirror the portfolio spawn at the `#[cfg(feature = "brick-portfolio")]` block). Phase 04
EXTENDS Phase 03's constructor from `ActivityCache::new(db, ranking)` to the canonical
four-argument form `ActivityCache::new(db, ranking: RankingConfig, forge: ForgeConfig,
forge_client: Arc<dyn ForgeClient + Send + Sync>)`. Production builds a `ForgeRouter` from
`ForgeConfig` + **env tokens** (using `with_base_url` so the base is overridable), then wraps it
as `Arc<dyn ForgeClient>`:

```rust
#[cfg(feature = "brick-activity")]
let activity_cache = {
    use std::sync::Arc;
    use plinth_server::bricks::activity::cache::ActivityCache;
    use plinth_forge::{ForgeClient, ForgeRouter, GitHubClient, CodebergClient};

    let forge = config.forge.clone();   // ForgeConfig: ttl/backoff + base URLs
    // Tokens are ENV-ONLY (never toml keys).
    let github_token = std::env::var("GITHUB_TOKEN").ok();
    let codeberg_token = std::env::var("CODEBERG_TOKEN").ok();

    // with_base_url so the base is overridable (prod uses the canonical defaults).
    let router = ForgeRouter {
        github: GitHubClient::with_base_url(forge.github_base_url.clone(), github_token),
        codeberg: CodebergClient::with_base_url(forge.codeberg_base_url.clone(), codeberg_token),
    };
    let forge_client: Arc<dyn ForgeClient + Send + Sync> = Arc::new(router);

    ActivityCache::spawn(ActivityCache::new(
        db.clone(),
        config.ranking.clone(),   // RankingConfig (Phase 03)
        forge,                    // ForgeConfig (ttl/backoff/base-urls)
        forge_client,             // Arc<dyn ForgeClient>
    ))
};
```

`ActivityCache::new` gains the `forge: ForgeConfig` and `forge_client: Arc<dyn ForgeClient +
Send + Sync>` params (the actor reads `ttl`/`backoff` from `forge`). Keep the existing
`AppState { ... activity_cache, ... }` field assignment (Phase 03 added it). **You MUST update
every `ActivityCache::new` call site** — both this `main.rs` spawn AND Phase 03's integration
test `crates/server/tests/activity_brick.rs` (pass a mock `Arc<dyn ForgeClient>` there). The
serialize-note with Phase 07 applies to `main.rs`.

### Step 5 — Make `refresh` a module of the brick (`mod.rs`)

Edit `/data/nvme0/can/Projects/solo/plinth/crates/server/src/bricks/activity/mod.rs` and add
`pub mod refresh;` alongside the existing `pub mod cache; pub mod admin; pub mod api; pub mod
migrations;`. (This is one of the files contended with Phase 07 — rebase first per Working
tree.)

### Step 6 — Tests (this phase MUST add them; named below)

Add an integration test file
`/data/nvme0/can/Projects/solo/plinth/crates/server/tests/activity_refresh.rs`, mirroring the
structure of `crates/server/tests/portfolio_publish.rs` (feature-gated `mod enabled`/`mod
disabled`, an `app_state(pool)` builder, `#[sqlx::test(migrations = "./migrations")]`). The
**PRIMARY** way to drive these tests is to inject a **mock `Arc<dyn ForgeClient>`** — a test
double implementing the canonical trait (`async fn fetch(&self, r: &ActivityRef) ->
ForgeResult<FetchedActivity>`) that returns canned `FetchedActivity` / `ForgeError` and counts
calls via an `Arc<AtomicUsize>`. This needs no network at all and is the cleanest way to assert
single-flight and the error/backoff paths. (A secondary HTTP-level option is to point a real
`ForgeRouter` at a `wiremock` server via `with_base_url`; if you take it, add `wiremock = "0.6"`
to `crates/server/Cargo.toml` `[dev-dependencies]` — but the mock-client route is preferred.)

Three required tests (the brief mandates exactly these):

1. **`fresh_data_fires_no_refresh`** — Seed `activity_items` with `fetched_at = now()` (fresh).
   Spawn `ActivityCache` with a counting mock `Arc<dyn ForgeClient>` and TTL = 3600s. Call
   `state.activity_cache.ask(GetRankedActivity { limit: None, featured_only: false }).await`
   once. Assert the reply is `200`-equivalent (`Vec<ActivityListItem>`, N items) AND the mock
   client's `fetch` counter is `0` (no refresh fired). Give the spawned-task a moment
   (`tokio::task::yield_now().await` a few times, or a short `tokio::time::sleep`) before
   asserting the counter, to prove nothing was queued.

2. **`stale_data_single_flight_under_concurrency`** — Seed rows with `fetched_at = now() -
   2h` (stale; TTL = 3600s). Spawn the actor with a counting mock `Arc<dyn ForgeClient>` whose
   `fetch` is artificially slow (e.g. `tokio::time::sleep(50ms)` then return canned data). Fire
   **N = 20 concurrent reads**: `let handles: Vec<_> = (0..20).map(|_| {
   let c = state.activity_cache.clone(); tokio::spawn(async move {
   c.ask(GetRankedActivity { limit: None, featured_only: false }).await }) }).collect();` then
   `join_all`. Assert every read returned `Ok` (served immediately, never blocked). Then await
   the in-flight refresh and assert the mock client's **fetch-call counter == number of seeded
   rows EXACTLY ONCE** (i.e. exactly one *sweep* ran, not 20). The cleanest invariant: count
   *sweeps* via an `AtomicUsize` incremented once per `run_refresh` entry — assert it equals `1`.

3. **`refresh_error_keeps_prior_data_and_200s`** — Seed two stale rows with known states (e.g.
   `state = 'open'`). Spawn the actor with a mock `Arc<dyn ForgeClient>` whose `fetch` returns a
   `ForgeError` (e.g. `ForgeError::RateLimited { .. }` or `ForgeError::Network(..)`). Fire one
   `ask(GetRankedActivity { limit: None, featured_only: false })` read: assert it returns `Ok`
   with the **original** rows (state still `'open'`) — the endpoint still succeeds. Await the
   failed refresh, then fire a SECOND read immediately and assert the mock client was NOT called
   again (backoff suppresses re-fire) AND the data is still the original stale rows (unchanged).
   Optionally assert via `sqlx::query_scalar` that the DB rows were not corrupted.

For an end-to-end HTTP variant (optional but recommended), build the axum router as
`portfolio_publish.rs` does and `app.oneshot(GET /api/activity)` to assert `StatusCode::OK`
under the error case.

### Step 7 — Verify the full workspace

```bash
cargo test -p plinth-server --test activity_refresh        # the three named tests
cargo clippy --workspace --all-targets -- --deny warnings  # flake's plinth-clippy gate
cargo fmt --all -- --check                                  # flake's plinth-fmt gate
```

`cargo test --workspace --all-targets` is what `nix flake check`'s `plinth-test` runs against a
sandbox Postgres; your `#[sqlx::test(migrations = "./migrations")]` tests get `DATABASE_URL`
for free there. wiremock binds loopback inside the sandbox (allowed); never make real outbound
HTTP in tests (the sandbox has no network).

## Acceptance criteria

- [ ] `cargo test -p plinth-server --test activity_refresh` passes with all three named tests
      present: `fresh_data_fires_no_refresh`, `stale_data_single_flight_under_concurrency`,
      `refresh_error_keeps_prior_data_and_200s`.
- [ ] `fresh_data_fires_no_refresh`: with `fetched_at = now()` and TTL 3600s, after one
      `ask(GetRankedActivity { limit: None, featured_only: false })` the mock forge client's
      `fetch` counter is **exactly 0**.
- [ ] `stale_data_single_flight_under_concurrency`: with stale rows and **20 concurrent**
      `ask(GetRankedActivity { limit: None, featured_only: false })` reads, every read returns
      `Ok` immediately and the refresh-sweep counter is **exactly 1** (single-flight proven under
      contention).
- [ ] `refresh_error_keeps_prior_data_and_200s`: a forge error during refresh leaves the
      cached rows byte-identical to before (same `state`), the read returns `Ok` (the
      equivalent HTTP path returns `StatusCode::OK`/`200`), and a second immediate read does
      **not** re-invoke the forge client (backoff active).
- [ ] `GET /api/activity` returns `200` even when the configured forge is unreachable
      (verified by the error test, optionally via `app.oneshot`).
- [ ] The read handlers (`GetRankedActivity`, `GetActivityItem`) compute and return their reply
      **before** triggering refresh; no `.await` on a forge/network call exists on the read
      reply path (verify by inspection: `run_refresh` is only ever reached through
      `tokio::spawn`, never directly awaited inside a `Message::handle` read handler).
- [ ] The refresh UPDATE statement touches only `state`, `merged_at`, `closed_at`, `additions`,
      `deletions`, `comments_count`, `repo_stars`, `labels`, `fetched_at` — and **never**
      `embedding`, `title`, `body`, or `impact` (verify by inspecting the SQL in `refresh.rs`).
- [ ] `config.forge.refresh_ttl_secs` defaults to `3600`, `refresh_backoff_secs` to `900`,
      `github_base_url` to `"https://api.github.com"`, and `codeberg_base_url` to
      `"https://codeberg.org/api/v1"`; all are overridable from `[forge]` in `plinth.toml`.
      Tokens are read at spawn time from the `GITHUB_TOKEN`/`CODEBERG_TOKEN` env vars only
      (there are NO token fields on `ForgeConfig`). Add a `toml_config.rs` unit test asserting
      the defaults parse from empty TOML, mirroring the existing `test_parse_empty_toml`.
- [ ] `cargo clippy --workspace --all-targets -- --deny warnings` reports **0 warnings**.
- [ ] `cargo fmt --all -- --check` is clean.
- [ ] The spawned refresh task is wrapped so a panic inside `run_refresh` is caught (via the
      inner `tokio::spawn(...).await` JoinError branch) and converted to `RefreshOutcome::Failed`,
      guaranteeing the `refreshing` latch is always cleared by a `RefreshDone` message (verify by
      inspection; optionally a test that injects a `panic!()` in the mock client and asserts a
      later read can still trigger a fresh refresh).

## Files likely touched

- **New:** `crates/server/src/bricks/activity/refresh.rs` (the off-path refresh worker; the
  only conflict-free new file).
- **New:** `crates/server/tests/activity_refresh.rs` (the three named integration tests).
- **Edit:** `crates/server/src/bricks/activity/cache.rs` (add `refreshing`/`backoff_until`/
  `ttl`/`backoff`/`forge_client` fields, `is_stale`/`in_backoff`/`maybe_trigger_refresh`, the
  `RefreshDone` message; call the trigger at the tail of each read handler. **EXTEND the
  constructor** to `ActivityCache::new(db, ranking: RankingConfig, forge: ForgeConfig,
  forge_client: Arc<dyn ForgeClient + Send + Sync>)`). *Contended with Phase 07 — rebase first.*
- **Edit:** `crates/server/src/bricks/activity/mod.rs` (`pub mod refresh;`). *Contended with
  Phase 07.*
- **Edit:** `crates/server/src/main.rs` (build a `ForgeRouter` from `ForgeConfig` + env tokens
  via `with_base_url`, wrap as `Arc<dyn ForgeClient>`, pass the canonical four-arg
  `ActivityCache::new(db, ranking, forge, forge_client)`). *Contended with Phase 07 route
  registration — rebase first.*
- **Edit:** `crates/server/tests/activity_brick.rs` (Phase 03's integration test) — update its
  `ActivityCache::new` call site to the new four-arg ctor, injecting a mock
  `Arc<dyn ForgeClient>`. **Every `ActivityCache::new` call site MUST be updated in lockstep.**
- **Edit:** `crates/shared/src/toml_config.rs` (add/extend `ForgeConfig` with
  `refresh_ttl_secs`/`refresh_backoff_secs`/`github_base_url`/`codeberg_base_url`; wire into
  `PlinthConfig`. No token fields — tokens are env-only and read at spawn time, not here).
- **Edit:** `crates/server/Cargo.toml` (`[dev-dependencies] wiremock = "0.6"` if using the HTTP
  mock route).
- **Edit:** `plinth.toml` (the `[forge]` example section).

## Pitfalls

- **P1 — Stampede (the headline risk).** Symptom: under load right after TTL expiry, N reads
  each spawn a refresh; GitHub's 60-req/hour unauthenticated limit is exhausted in one burst.
  Cause: setting `refreshing = true` *after* an `.await`, or checking it in the spawned task
  instead of the handler. Recovery: the check-and-set MUST be a synchronous pair inside the
  message handler (no `.await` between them) — Kameo processes one message at a time, so this is
  atomic. The `stale_data_single_flight_under_concurrency` test proves it.
- **P2 — Render-blocking refresh.** Symptom: `/activity` page TTFB spikes by the forge round-trip
  latency once per TTL. Cause: calling `run_refresh(...).await` directly inside the read handler
  (or `.ask()`-ing a refresh message and awaiting it). Recovery: refresh is reached ONLY through
  `tokio::spawn`; the read handler returns its cloned reply first, then calls
  `maybe_trigger_refresh` which spawns and returns instantly.
- **P3 — Lost panic wedges the latch forever.** Symptom: one refresh panics, `RefreshDone` is
  never sent, `refreshing` stays `true`, and the cache never refreshes again for the process
  lifetime. Cause: a bare `tokio::spawn` whose task panics is silently dropped. Recovery: wrap
  the refresh body in an inner `tokio::spawn(...).await` and map `Err(JoinError)` →
  `RefreshOutcome::Failed`, so a `RefreshDone` is always delivered and the latch always clears.
- **P4 — Rate-limit thrash.** Symptom: a forge returns 429/403; the next read immediately
  refires the refresh; repeat. Cause: no backoff. Recovery: on `RefreshOutcome::Failed`, set
  `backoff_until = now + refresh_backoff_secs`; `maybe_trigger_refresh` returns early while
  `in_backoff()`. Note Codeberg returns **no** `X-RateLimit-*` headers — treat 429 reactively;
  GitHub exposes `x-ratelimit-reset` (Phase 02's client may surface it; if so honor it, else use
  the fixed backoff).
- **P5 — Re-embedding on refresh.** Symptom: refresh fails or is slow because it tries to run
  fastembed (which the server does not have) or it overwrites a good embedding with NULL.
  Cause: including `embedding` in the UPDATE. Recovery: the refresh UPDATE column list excludes
  `embedding`/`title`/`body` entirely.
- **P6 — TTL measured against the wrong clock.** Symptom: refresh never fires, or fires on every
  read. Cause: comparing `fetched_at` (a `TIMESTAMPTZ` wall clock from the DB) against
  `Instant` (a monotonic clock) — they are not comparable. Recovery: either gate staleness on
  the actor's own `cache_populated_at: Instant` (monotonic, simplest), OR compute staleness from
  the row `fetched_at` using `chrono::Utc::now() - fetched_at` (wall clock) — never mix the two.
- **P7 — `ctx.actor_ref()` lifetime / generic-reply friction.** Symptom: borrow-checker errors
  threading `ctx` into a generic `maybe_trigger_refresh`. Cause: the two read handlers have
  different `Reply` types (`Vec<ActivityListItem>` for `GetRankedActivity`, `Option<ActivityItem>`
  for `GetActivityItem`). Recovery (canonical): `maybe_trigger_refresh` takes a CONCRETE
  `me: ActorRef<ActivityCache>` argument, obtained in each handler via `ctx.actor_ref().clone()` —
  NOT a generic `Context<Self, impl Send>`. This sidesteps the generic-reply friction entirely.
- **P8 — Re-ranking drift after refresh.** Symptom: the refreshed cache orders items differently
  from `GET /api/activity` (which Phase 03 ranks in SQL). Cause: re-ranking in Rust inside
  `run_refresh`. Recovery: `reread_ranked` calls Phase 03's exact ranked SELECT helper — do not
  duplicate or reimplement the `ORDER BY score DESC, reference_date DESC` logic.
- **P9 — A stale `ActivityCache::new` call site won't compile.** Symptom: after extending the
  constructor to the four-arg `ActivityCache::new(db, ranking, forge, forge_client)`, the
  workspace fails to build at an un-updated caller. Cause: missing a call site. Recovery: update
  ALL of them in the same commit — the `main.rs` spawn AND Phase 03's integration test
  `crates/server/tests/activity_brick.rs` (inject a mock `Arc<dyn ForgeClient>` there). Grep for
  `ActivityCache::new(` across the workspace before you finish.

## Risk profile

The blast radius is contained to the activity brick and config, but the *correctness* surface
is high because the bugs are concurrency bugs that pass a naive smoke test and only bite under
production load: a stampede only shows up when many visitors hit a freshly-expired cache (and it
manifests as forge rate-limit bans, not a crash); a render-block only shows up as latency once
per TTL; a lost-panic latch only shows up as "the data never updates again" hours later. None of
these throw. The data-loss risk is low (refresh is a narrow UPDATE that never touches embeddings
or identity columns, and failures keep stale data), but a careless UPDATE that included
`embedding` would silently degrade semantic search. The forge-dependency risk is external and
unbounded (rate limits, outages) — mitigated by single-flight + backoff + keep-stale-on-error.
Highest residual risk: the single-flight latch correctness and the panic-recovery wrapper; both
are covered by named tests, and both are inspectable in code review.

## Strategy

1. **Land config + the worker function first (Steps 1–2), in isolation.** `refresh.rs` is a
   free function with no actor coupling — write it, then unit-test its UPDATE against a seeded
   DB row with a mock client *before* wiring the actor. This de-risks the SQL and the
   `FetchedActivity` → row mapping independently of the concurrency.
2. **Add the latch + trigger (Step 3) and prove single-flight with the mock client.** Use a
   *sweep counter* (`AtomicUsize` incremented once at `run_refresh` entry) as the single-flight
   invariant — it is the cleanest thing to assert under N concurrent reads. Write
   `stale_data_single_flight_under_concurrency` immediately and iterate until it is green.
3. **Add panic recovery (P3) and the backoff (P4) last**, each with a dedicated assertion. The
   panic-recovery is the easiest correctness property to forget and the hardest to notice
   missing in prod, so make it the final, explicitly-tested gate.
4. **Throughout, keep the read path side-effect-light:** the only mutation a read does is
   flipping `refreshing` and spawning a task — never DB writes, never network awaits. Re-read
   the two read handlers at the end and confirm no `.await` on forge/DB-write paths exists
   between receiving the message and returning the reply.
5. **Rebase discipline:** because `cache.rs`/`mod.rs`/`main.rs` are shared with Phase 07, do the
   `cache.rs` edits in small, reviewable chunks and re-run `cargo build -p plinth-server` after
   each, so a mid-phase rebase against Phase 07 is a small merge, not a rewrite.

## Rollback drill

This phase is **additive and feature-gated** (`brick-activity`), so rollback is clean:

1. **Disable refresh without reverting code:** set `refresh_ttl_secs` to a very large value
   (e.g. `315360000`, ~10 years) in `[forge]` / via env — staleness never triggers, the actor
   behaves exactly like the Phase 03 static-snapshot cache. This is the zero-deploy kill switch;
   document it in the `[forge]` toml comment.
2. **Revert the code:** `git revert` the commit(s) for this phase. The only cross-file edits are
   the `cache.rs` field/handler additions + constructor extension, `mod.rs`'s `pub mod refresh;`,
   the `main.rs` spawn args, the `crates/server/tests/activity_brick.rs` ctor call-site update,
   and the `toml_config.rs` `ForgeConfig` fields. Reverting restores Phase 03's two-arg
   `ActivityCache::new(db, ranking)` signature — make sure the `main.rs` spawn line AND the Phase
   03 test call site revert in lockstep (all in the same commit). `refresh.rs` and
   `activity_refresh.rs` are new files; `git revert` removes them.
3. **If only the refresh is misbehaving in prod (e.g. thrashing despite backoff):** ship a
   one-line change setting `refreshing` semantics to never trigger — but prefer the config kill
   switch (step 1) which needs no rebuild.
4. **Verify rollback:** `cargo test -p plinth-server` (the brick still builds and the Phase 03
   read tests pass), `GET /api/activity` still `200`s with the last-known snapshot.

No migration was added, so there is **nothing to roll back at the schema level** — the
`activity_items` table (incl. `fetched_at`) is owned by Phase 03 and untouched here.

## Failure modes and recoveries

- **F1 — Stampede / thundering herd.** *Symptom:* forge rate-limit bans; logs show many
  near-simultaneous `activity refresh complete` entries after a TTL boundary. *Cause:* the
  `refreshing` check-and-set straddles an `.await`, or the latch lives in the spawned task
  instead of the actor. *Recovery:* move the `if self.refreshing { return } self.refreshing =
  true;` pair to be the first synchronous statements in `maybe_trigger_refresh` with no `.await`
  between them; rely on Kameo's one-message-at-a-time mailbox for atomicity. Regression-guarded
  by `stale_data_single_flight_under_concurrency` (sweep counter must equal 1 under 20 reads).
- **F2 — Render-blocking refresh.** *Symptom:* `/activity` TTFB jumps by forge round-trip
  latency once per TTL window; under forge slowness the page hangs. *Cause:* awaiting
  `run_refresh` (or an `.ask()` refresh message) on the read reply path. *Recovery:* refresh is
  invoked ONLY via `tokio::spawn` from `maybe_trigger_refresh`, called *after* the reply value
  is computed; the read handler never awaits anything forge- or DB-write-related. Verified by
  inspection + the concurrency test (all 20 reads return promptly while the slow refresh is
  still in flight).
- **F3 — Rate-limit thrash.** *Symptom:* repeated 429s in logs; each read refires a refresh.
  *Cause:* no backoff after a failed refresh. *Recovery:* `RefreshOutcome::Failed` sets
  `backoff_until = now + refresh_backoff_secs`; `maybe_trigger_refresh` early-returns while
  `in_backoff()`. Honor `Retry-After`/`x-ratelimit-reset` if Phase 02 surfaces it; otherwise the
  fixed 15-min backoff applies. Guarded by `refresh_error_keeps_prior_data_and_200s` (second
  read does not re-invoke the client).
- **F4 — Lost panic in the spawned task.** *Symptom:* refresh stops happening permanently after
  some hours; `refreshing` is stuck `true`; no error in logs (the panic was swallowed by the
  detached task). *Cause:* bare `tokio::spawn` with a panicking body. *Recovery:* wrap the
  refresh in an inner `tokio::spawn(...).await`; on `Err(JoinError)` (panic/cancel) deliver
  `RefreshDone(RefreshOutcome::Failed { .. })` so the actor always clears the latch and starts a
  backoff. Optionally add a test injecting `panic!()` in the mock client and asserting a later
  read can trigger a fresh refresh (latch was cleared).
- **F5 — Stuck latch from a dropped actor / never-delivered RefreshDone.** *Symptom:* same as
  F4 but the actor was stopped/restarted mid-refresh. *Cause:* the `me.tell(RefreshDone)` target
  is gone. *Recovery:* `tell` failing is harmless (the actor is gone); on a fresh actor the
  latch starts `false`. If you later add actor restart-in-place, ensure `refreshing` resets to
  `false` in the actor's `on_start`/constructor (it does, via `ActivityCache::new`).
- **F6 — Stale-data UPDATE clobbers good data on partial failure.** *Symptom:* after a flaky
  forge response, some rows show NULL/garbage forge fields. *Cause:* mapping a malformed
  `FetchedActivity` into the UPDATE, or updating before validating. *Recovery:* the UPDATE binds
  only well-typed fields from a successfully-parsed `FetchedActivity`; a parse/network error
  aborts the sweep (`RefreshOutcome::Failed`) *before* writing that row, and 404/410 rows are
  skipped (`continue`) rather than nulled. Stale data is always preferable to corrupt data.

## Reference

- **Design brief — locked decision #2** (lazy stale-while-revalidate, single-flight, TTL=1h,
  off-path, keep-stale-on-error, backoff, no-re-embed) is fully inlined above; this file is the
  authoritative spec for the freshness mechanism. Schema fields (`fetched_at`, `state`,
  `merged_at`, `closed_at`, `additions`, `deletions`, `comments_count`, `repo_stars`, `labels`,
  `embedding`) and endpoints (`GET /api/activity`, `GET /api/activity/{id}`) are restated inline
  so this phase is standalone.
- **Sequencing only (no content needed from them):**
  - `./03-server-brick-core.md` — **must land first**; creates `bricks/activity/cache.rs`, the
    ranked-read query helper this phase's `reread_ranked` calls, the `activity_items` migration
    (incl. `fetched_at`), the row decoder, and the `AppState.activity_cache` field.
  - `./02-forge-crate.md` — **must land first**; provides `plinth_forge::{ForgeClient,
    GitHubClient, CodebergClient, FetchedActivity, ForgeError}` and the `Forge` enum. Adapt the
    exact names from `crates/forge/src/lib.rs`.
  - `./07-feed-and-search.md` — also edits `bricks/activity/{mod.rs,api.rs}` and `main.rs` route
    registration; whichever of 04/07 lands second must rebase first (see Working tree).
- **In-repo patterns to copy (paths, not content gates):**
  - `/data/nvme0/can/Projects/solo/plinth/crates/server/src/bricks/portfolio/cache.rs` — the
    `#[derive(Actor)]` + `Message`/`Reply`/`handle` + `Instant`-based TTL idiom this phase
    extends.
  - `/data/nvme0/can/Projects/solo/plinth/crates/server/src/actors/vector_search.rs` — the
    `spawn_blocking`/off-runtime discipline and `Arc<…>`-shared-resource pattern.
  - `/data/nvme0/can/Projects/solo/plinth/crates/server/tests/portfolio_publish.rs` — the
    `#[sqlx::test(migrations = "./migrations")]` + `app_state(pool)` + actor `.ask()` test
    skeleton to mirror for `activity_refresh.rs`.
  - `/data/nvme0/can/Projects/solo/plinth/crates/shared/src/toml_config.rs` — the
    `SearchConfig`/`impl Default`/`default_*()` config idiom for `ForgeConfig`.
