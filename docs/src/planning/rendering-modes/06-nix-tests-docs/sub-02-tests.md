# Phase 06 · Sub-02 — End-to-end tests per rendering mode

> **Recommended Codex model: GPT 5.5 medium**
>
> Moderate complexity, leaf role. The test harness already exists
> (`crates/server/tests/common/mod.rs` boots sandbox Postgres); the work is writing
> assertions that pin each rendering mode's *observable* behavior — static = no
> per-request SQL, streaming = incremental chunks, islands = selective hydration —
> which requires understanding what each mode actually changes at the HTTP/HTML
> level. Trivial-tier would write shallow "returns 200" tests that don't actually
> distinguish the modes; medium is the floor.

## Working tree

`/data/nvme0/can/Projects/solo/plinth` — after Phases 01–05. Adds files under
`crates/server/tests/`; does not modify rendering code.

## Goal

A named e2e test suite proves each route's assigned rendering mode behaves as
specified, so a regression (a static route quietly going dynamic, streaming
collapsing to buffered, an island failing to hydrate, the `todo!` panic returning)
fails CI rather than shipping.

## Why this matters now

The plan's whole-set acceptance criteria assert mode-specific behaviors that the
current suite doesn't check — it covers the activity brick and feeds, not rendering
strategy. Each mode has a distinct, testable signature; without tests pinning them,
the modes are unverifiable and the next refactor silently regresses them.

## Out of scope

- Unit-testing Leptos internals — assert observable HTTP/HTML behavior.
- Browser-driven hydration tests requiring a full headless browser, unless the repo
  already has that harness; prefer asserting on served HTML/payload manifests and
  the WASM chunk set.
- Re-testing data correctness already covered by the brick/activity suites.

## Plan

1. **Reuse the sandbox-Postgres harness** in `crates/server/tests/common/mod.rs`
   (the pattern `activity_brick.rs` / `activity_feed_search.rs` use).
2. **`rendering_data_path` test** — the gap regression guard: seed a blog post, a
   portfolio item, a todo; GET `/posts`, `/projects`, `/todos`; assert HTTP 200 and
   the seeded titles appear in the SSR HTML (i.e. no `todo!("phase 03")` panic, no
   "Could not load").
3. **`static_routes_no_per_request_sql` test** — request `/about` (or a published
   `/posts/<slug>`) twice; assert the second response does no DB query (instrument a
   query counter in the test pool, or assert the response is served from the static
   cache without hitting the brick query path). Then publish an edit via the admin
   API and assert the next request reflects it (invalidation works).
4. **`home_streams_out_of_order` test** — inject an artificial delay into the
   activity loader (mock/feature-gated slow path); request `/`; assert the shell +
   intro + blog/portfolio bytes arrive before the activity strip bytes (assert on
   streamed chunk ordering / first-byte timing, not just final HTML).
5. **`islands_selective_hydration` test** — assert the served HTML for a content-only
   route (`/about`) references only the island runtime + the toggle/menu island
   chunks, not a full-page hydration bundle; assert a static route's body markup is
   present in the initial HTML (server-rendered, not JS-injected). If a headless
   browser is available, assert the toggle island responds to a click; otherwise
   assert the island boundary markers are present in the HTML.
6. **`dynamic_routes_fresh` test** — `/activity` reflects a newly added item without a
   rebuild (request-time SSR); `/todos` reflects an admin edit immediately.
7. **Name every test** so acceptance criteria can reference targets, e.g.
   `cargo test -p plinth-server --test rendering_modes`.

## Acceptance criteria

- [ ] `cargo test -p plinth-server --test rendering_modes` (or the chosen test file)
      passes against sandbox Postgres with all bricks enabled.
- [ ] `rendering_data_path` fails if any `todo!("phase 03")` panic path is
      reintroduced (verified by temporarily reverting one stub-fill and seeing red).
- [ ] `static_routes_no_per_request_sql` proves no DB query on a cached static route
      AND regeneration after publish.
- [ ] `home_streams_out_of_order` proves the slow activity section does not delay the
      other sections' bytes.
- [ ] `islands_selective_hydration` proves a content-only page does not ship a
      full-page hydration bundle.
- [ ] All new tests are named and run under `nix flake check` (via the `plinth-test`
      check).

## Files likely touched

- `crates/server/tests/rendering_modes.rs` (new; or split per-mode files).
- `crates/server/tests/common/mod.rs` (only to add a shared helper — e.g. a
  query-counter pool wrapper or a slow-loader injection hook — if not already present).

## Pitfalls

- **Asserting on final HTML for streaming.** Final HTML looks identical whether or
  not it streamed. The streaming test must assert on *chunk arrival order / timing*,
  which requires reading the response body incrementally, not `.text().await`.
- **Counting SQL for static routes.** If the route is served from the static cache,
  there is no query to count — assert the *absence* of a query (counter unchanged),
  which means the test pool must be the one the server uses. Wire the counter through
  the harness, not a separate pool.
- **Headless-browser dependency creep.** Don't add a Playwright/chromedriver dep just
  for hydration assertions if the repo lacks it; assert on HTML island markers and
  the served chunk manifest instead, and note the limitation.
- **Flaky timing.** The streaming test's injected delay must be deterministic
  (a controllable barrier/channel), not a wall-clock `sleep` race.

## Reference

- Harness + named-test conventions: `crates/server/tests/common/mod.rs`,
  `crates/server/tests/activity_brick.rs`, `crates/server/tests/activity_feed_search.rs`.
- Modes under test: [../01-ssr-data-path.md](../01-ssr-data-path.md) ..
  [../05-csr-profile.md](../05-csr-profile.md).
