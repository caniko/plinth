# Plan: Forge Activity — curated cross-forge contribution showcase

> **Recommended Codex model for plan-set orchestration: GPT 5.5 high**
>
> Coordinating this set is a complex orchestration role: eight phases across four
> crates plus a new library crate, with two genuinely hard sub-problems (correct
> GitHub/Forgejo normalization and a race-free single-flight lazy refresh) and a
> tight cross-phase contract. A lighter model tends to lose the seam invariants
> (who owns which type, which message name, the public/admin auth boundary) when
> dispatching and merging. Per-phase execution is routed individually below —
> several phases are `5.5 medium`; do **not** uniformly route to `max`.

## Scope

A new plinth **brick** (`brick-activity`) that showcases the owner's curated
*external* contributions across **GitHub** and **Codeberg**, ranked by
**impact × recency**. It is distinct from the existing `portfolio` brick:
portfolio = the owner's own projects; activity = real PRs/issues the owner landed
on other people's repos. The owner curates entirely via the CLI by giving a
forge + repo + PR/issue number + an impact score (1–10); everything else is
fetched from the forge API and persisted in Postgres.

### Locked design decisions

1. **New `activity` brick**, feature-gated `brick-activity`, mirroring the
   portfolio brick's structure (migrations / admin / api / Kameo cache actor /
   frontend pages). Activity routes by **numeric id** (it has no slug).
2. **Lazy stale-while-revalidate refresh.** Public reads serve cached data
   immediately; if data is older than a TTL (default **1h**) the cache actor
   fires a **single-flighted** background forge re-fetch (no stampede, never
   blocks render, failures keep stale data + back off). Forge-fetch logic lives
   in a new **`plinth-forge`** crate shared by the CLI (add-time) and the server
   (refresh) — kept **out** of the WASM client.
3. **Configurable ranking** (`[ranking].strategy` ∈ `exponential` (default) /
   `linear` / `pure`), impact **1–10** (default 1), score computed in **SQL at
   read time** (no stored score) so it is always current.
4. **Four surfaces:** dedicated `/activity` list + `/activity/:id` detail page, a
   home-page top-N feature strip, an RSS feed at `/feeds/activity.xml`, and
   pgvector **search integration** (entries embedded via fastembed in the CLI).

### Current state

Greenfield feature. Plinth today has the blog / portfolio / todo bricks, the
Postgres + pgvector + sqlx stack, the Kameo actor system, the clap CLI
(`api_client` Bearer pattern + local fastembed), the Leptos SSR + WASM client,
and an mdBook docs site. No `activity_items` table, no `plinth-forge` crate, and
no forge-API client exist yet — every phase below adds new, mostly
conflict-disjoint surface.

## Phases

| Phase | File | Layout | Codex tier | Depends on | Touches | Can parallel with | Blocking? |
|------|------|--------|-----------|-----------|---------|-------------------|-----------|
| 01 | [Shared types + migration](./01-shared-types-and-migration.md) | single | 5.5 medium | — | `crates/shared`, `crates/server/migrations` | — | **yes — blocks all** |
| 02 | [plinth-forge crate](./02-forge-crate.md) | single | 5.5 high | 01 | `crates/forge` (new) | 03 | no |
| 03 | [Server brick core](./03-server-brick-core.md) | single | 5.5 medium | 01 | `crates/server/src/bricks/activity`, `main.rs`, `services` | 02 | partial (04/06/07 need it) |
| 04 | [Lazy refresh actor](./04-lazy-refresh-actor.md) | single | 5.5 high | 02, 03 | `bricks/activity/{cache,refresh}.rs`, `main.rs` | 05, 06 | no |
| 05 | [CLI commands](./05-cli-commands.md) | single | 5.5 medium | 01, 02, 03 | `crates/cli` | 04, 06, 07 | no |
| 06 | [Frontend surfaces](./06-frontend-surfaces.md) | single | 5.5 medium | 03 | `crates/client` | 04, 05, 07 | no |
| 07 | [Feed + search](./07-feed-and-search.md) | single | 5.5 medium | 03 (+05 for embedded data) | `bricks/activity/api.rs`, search service, `main.rs` | 05, 06 | no |
| 08 | [Nix + tests + docs](./08-nix-tests-docs/README.md) | **sub-layered** | merge 5.5 medium | 01–07 | flake / tests / docs | — | final |

Phase 08 fans out into three disjoint sub-layers — [nix packaging](./08-nix-tests-docs/sub-01-nix-packaging.md) (`5.5 medium`), [e2e tests](./08-nix-tests-docs/sub-02-e2e-tests.md) (`5.5 medium`), [docs](./08-nix-tests-docs/sub-03-docs.md) (`5.5 low`) — that touch `flake.nix` / `crates/server/tests` / `docs/src` respectively and can run in parallel, then a `5.5 medium` merge runs the full `nix flake check`.

## Parallelism layer (execution waves)

- **Wave 0 — `01`.** Foundation: the shared types + `0006_activity.sql` migration. Everything compiles against these, so it must land first.
- **Wave 1 — `02` ∥ `03`.** Disjoint files (`crates/forge` vs `crates/server/src/bricks/activity`); both depend only on Phase 01. The forge crate and the server brick can be built fully in parallel. (Phase 03 deliberately leaves a "Phase 04 seam" and does **not** import `plinth-forge`, so it does not wait on Phase 02.)
- **Wave 2 — `04` ∥ `05` ∥ `06` ∥ `07`.** All four unlock once Phase 03's brick + public API contract exist (Phase 04 also needs Phase 02). They touch mostly disjoint trees: `04` → the cache/refresh internals, `05` → `crates/cli`, `06` → `crates/client`, `07` → the brick's feed/search + search service.
- **Wave 3 — `08`.** After 01–07. Fan out the three sub-layers, then merge + `nix flake check`.

### Serialization hazards (the only non-disjoint edits)

- **`04` and `07` both edit `crates/server/src/bricks/activity` and the `main.rs` route registration.** They can be authored in parallel, but whichever **lands second must rebase first**. This is flagged in both phase docs' "Working tree".
- **Phase 04 changes the `ActivityCache::new` constructor** (adds `forge: ForgeConfig` + `forge_client: Arc<dyn ForgeClient>`). Phase 04 owns updating *all* call sites it breaks — the `main.rs` spawn **and** Phase 03's integration test (`crates/server/tests/activity_brick.rs`).
- **Phase 07's search test needs embedded data**, which the CLI produces (Phase 05). The feed half of Phase 07 is independent; only the search-similarity test gates on Phase 05.

## Whole-set acceptance criteria

- [ ] `nix flake check` is green with `brick-activity` enabled: the new `crates/forge` crate builds, the workspace builds (server SSR + WASM client), clippy + fmt pass, and all named tests pass against the sandbox Postgres.
- [ ] A curated entry round-trips end to end: `plinth activity add --forge codeberg --repo <owner/name> --pr <n> --impact 8` fetches forge metadata, embeds it, and persists it; `GET /api/activity` returns it ranked; `/activity` renders it; `/feeds/activity.xml` includes it; `/api/search` surfaces it above `min_similarity`.
- [ ] Ranking honors `[ranking].strategy`: two seeded rows order correctly under each of `exponential`, `linear`, `pure` (named ranking tests).
- [ ] Freshness works: a fresh cache fires **no** refresh; a stale cache fires **exactly one** refresh under N concurrent reads (single-flight); a forge error during refresh keeps prior data and the endpoint still returns 200 (named refresh tests, driven by an injected mock `ForgeClient`).
- [ ] The public/admin auth boundary holds: admin routes require a Bearer token; `GET /api/activity`, `/api/activity/{id}`, and `/feeds/activity.xml` are unauthenticated.
- [ ] `plinth-forge` (and thus `reqwest`) is **absent** from the WASM client dependency tree (`cargo tree` check).

## Global constraints (apply to every phase)

- **Single source of truth for the contract.** Shared types are owned by Phase 01, the forge API by Phase 02, the brick handlers/messages by Phase 03. Do not redefine them elsewhere — consume them. The reconciled canonical shapes are summarized below.
- **WASM safety.** `plinth-shared` stays pure (no `reqwest`/`sqlx`/`pgvector`); `plinth-forge` is server+CLI only.
- **The server stamps `fetched_at`** (`chrono::Utc::now()`) on insert and on refresh — `PublishActivityRequest` carries no `fetched_at`.
- **Refresh never re-embeds.** It updates forge-derived columns only; an entry added before search wiring needs a CLI re-add to gain an embedding (documented pitfall).
- **Tokens are env-only:** `GITHUB_TOKEN` / `CODEBERG_TOKEN` (never TOML). Public data works unauthenticated but rate-limited.

## Canonical contract (quick reference)

- **Shared (Phase 01):** `Forge{GitHub,Codeberg}`, `ActivityKind{PullRequest,Issue}`, `ActivityState{Open,Closed,Merged}`, `RankingStrategy{Exponential,Linear,Pure}` (each enum has `as_str` + `FromStr`); `FetchedActivity`, `PublishActivityRequest` (no `fetched_at`; `published: bool`), `ActivityItem` (`id: i64`), `ActivityListItem` (`id: i64`, computed `score: f64`, `reference_date()` helper, no stored ref-date).
- **Forge (Phase 02):** `trait ForgeClient { async fn fetch(&self, r: &ActivityRef) }` — the *only* entrypoint; `ActivityRef{forge,owner,repo,kind,number}`; `ForgeRouter` dispatches by forge; `ForgeError` all-struct variants. Production wires `Arc<dyn ForgeClient>`; tests inject a mock.
- **Server (Phase 03):** handlers `publish_activity_item` / `delete_activity_handler` / `patch_activity_handler` / `list_activity_items` / `get_activity_item`; cache `ActivityCache` (field `ranked_list_cache`); messages `GetRankedActivity{limit,featured_only}` + `GetActivityItem(i64)` (reply `Result<_, String>`); shared `pub` ranked read `ranking::query_ranked_list(db, ranking, featured_only, limit)`; `Path<i64>` for delete/patch.
- **Config:** `[ranking]` strategy/half_life_days(365)/window_days(730); `[forge]` refresh_ttl_secs(3600)/refresh_backoff_secs(900)/github_base_url/codeberg_base_url; `[feeds].activity_limit`.

## Reference

- Originating ideation + the four locked decisions: this plan's design conversation.
- Mirror these existing bricks while implementing: `crates/server/src/bricks/portfolio/*` (brick + cache actor + admin/api), `crates/server/src/bricks/blog/*` (embeddings + HNSW), `crates/cli/src/commands/{publish,portfolio}.rs` (CLI + fastembed + ApiClient), `crates/client/src/pages/portfolio*.rs` (Leptos pages).
- Sibling plan set for shape reference: [`postgres-migration`](../postgres-migration/).

*Run the phases yourself (a fresh agent session per phase, fanning out per the waves above). When done, prompt `verify` to audit the acceptance criteria against the repo.*
