# Plan: Rendering modes — per-route CSR / SSR / SSG / Streaming / Islands

> **Recommended Codex model for plan-set orchestration: GPT 5.5 high**
>
> Coordinating this set is a complex orchestration role: six phases that move the
> Leptos client between rendering models (full-hydration SSR → per-route static
> generation → out-of-order streaming → app-wide islands) while one shared seam —
> the route table in `crates/client/src/app.rs` and the `cargo-leptos` feature
> matrix — is edited by three of them. The hard invariants are easy to lose when
> dispatching: which routes are `SsrMode::Static` vs streamed, that islands mode
> is an **app-wide** switch (not per-route) that composes with `SsrMode` rather
> than replacing it, and that the CSR profile must talk to the REST API while the
> SSR build talks to server functions. A lighter model tends to conflate islands
> with per-route hydration or let static-route regeneration drift from the publish
> path. Per-phase routing is individual below — do **not** uniformly route to
> `max`; only Phase 04 earns it.

## Scope

Plinth's Leptos client currently runs in exactly **one** rendering model: full
server-side rendering with whole-page hydration, built by `cargo-leptos`
(`bin-features = ["ssr", …]`, `lib-features = ["hydrate", …]`). Every page fetches
data through Leptos server functions (`#[server]`) consumed via `Resource` +
`<Suspense>`.

This plan makes the rendering strategy **explicit and per-surface**, introducing
the five modes the app should actually use:

- **SSR + Hydration** — the current default; retained for dynamic, always-fresh
  surfaces (`/activity`, `/activity/:id`, `/todos*`).
- **SSG (static generation)** — `SsrMode::Static` for content that only changes on
  publish: `/posts*`, `/series*`, `/projects*`, `/about`, `/support`.
- **Streaming SSR** — out-of-order streaming for the **home page** (`/`), whose
  shell + each section (`intro`, blog strip, projects strip, activity strip) can
  paint independently as their `Resource`s resolve.
- **Islands / partial hydration** — app-wide `experimental-islands`; only the two
  genuinely interactive widgets (`ThemeToggle`, the `Header` mobile-menu) hydrate,
  so static pages ship near-zero WASM.
- **CSR** — a real client-only build target (today's `csr` feature is inert and
  overridden by `cargo-leptos`), backed by the REST JSON API for static-host /
  preview / offline deployments.

### Why this plan also unblocks two prior plans

The Leptos data path is half-built. The **activity** server functions in
`crates/client/src/api.rs` are implemented against Postgres; the **blog,
portfolio, todo, series, and site-content** server functions are still
`todo!("phase 03")` stubs that **panic at runtime** (12 sites). "phase 03" refers
to the [`postgres-migration`](../postgres-migration/) plan's Phase 03 (query
rewrite). The server's own `services/db.rs` and brick caches were migrated to
`sqlx`/Postgres, but that migration never reached the client-facing server-fn
layer — so every non-activity page renders a panic.

Phase 01 of this plan fills those stubs (it is the prerequisite for choosing any
per-route mode, since a route cannot be SSG/streamed if its data loader panics).
Filling them **completes `postgres-migration`**, and `forge-activity` is already
fully shipped. So on a clean `verify` of this plan, **both prior plans
auto-retire** — see "Retirement" below.

### Current state (verified 2026-06-03)

- Live build: `cargo leptos build`; server `required-features = ["ssr"]`, client
  compiled with `hydrate`. `default = ["csr", …]` on the client crate is **inert**
  (cargo-leptos overrides it via `lib-features`).
- SSR wiring: `generate_route_list(App)` + `leptos_routes_with_context(...)` +
  `shell(...)` + `file_and_error_handler` fallback in `crates/server/src/main.rs`.
- Server functions: `GetActivityList` / `GetActivityItemById` implemented
  (`crates/client/src/api.rs:86-265`); the other 12 are `todo!("phase 03")`.
- Interactive surface: `ThemeToggle` (signal + `localStorage`) and `Header`'s
  `menu_open` mobile toggle are the **only** client-reactive widgets. All page
  bodies are read-only display.
- Postgres backend (`services/db.rs`, brick caches, migrations `0001`–`0006`) is
  fully on `sqlx`/Postgres; the single-flight activity refresh actor is live.

## Phases

| Phase | File | Layout | Codex tier | Depends on | Touches | Can parallel with | Blocking? |
|------|------|--------|-----------|-----------|---------|-------------------|-----------|
| 01 | [SSR data path](./01-ssr-data-path.md) | single | 5.5 high | — | `crates/client/src/api.rs` (+ server query helpers) | — | **yes — blocks all** |
| 02 | [SSG static routes](./02-ssg-static-routes.md) | single | 5.5 high | 01 | `crates/client/src/app.rs`, `crates/server/src/main.rs`, brick admin invalidation | 03, 05 | partial (04 needs it) |
| 03 | [Streaming home](./03-streaming-home.md) | single | 5.5 medium | 01 | `crates/client/src/pages/home.rs`, `app.rs` (`/` route) | 02, 05 | no |
| 04 | [Islands / partial hydration](./04-islands.md) | single | **5.5 max** | 01 (after 02, 03) | `app.rs`, `components/*`, `lib.rs`, `Cargo.toml`, cargo-leptos metadata | — | no |
| 05 | [CSR build profile](./05-csr-profile.md) | single | 5.5 medium | 01 | `crates/client/src/api.rs` (data-source split), `Cargo.toml`, `flake.nix` | 02, 03 | no |
| 06 | [Nix, tests, docs + retire](./06-nix-tests-docs/README.md) | **sub-layered** | merge 5.5 medium | 01–05 | flake / tests / docs | — | final |

Phase 06 fans out into three disjoint sub-layers —
[nix](./06-nix-tests-docs/sub-01-nix.md) (`5.5 medium`),
[tests](./06-nix-tests-docs/sub-02-tests.md) (`5.5 medium`),
[docs + retirement](./06-nix-tests-docs/sub-03-docs-retire.md) (`5.5 low`) — then a
`5.5 medium` merge runs the full `nix flake check`.

## Parallelism layer (execution waves)

- **Wave 0 — `01`.** Fill the server-function data path. Every other phase assigns
  a rendering mode to a route, which is meaningless while that route's loader
  panics. Must land first.
- **Wave 1 — `02` ∥ `03` ∥ `05`.** All three depend only on Phase 01.
  - `02` (SSG) and `03` (streaming home) both edit the route table in
    `crates/client/src/app.rs` (`app_routes()`): `02` adds `ssr=SsrMode::Static`
    attributes to many routes, `03` adjusts the `/` route. They can be authored in
    parallel, but **whichever lands second must rebase `app.rs` first** (flagged in
    both phase docs).
  - `05` (CSR) is mostly disjoint (it splits the data layer in `api.rs` and adds a
    build profile) but shares `Cargo.toml` / `flake.nix` with later phases —
    coordinate, don't race, on those two files.
- **Wave 2 — `04`.** Islands is an **app-wide** switch and edits `app.rs`,
  `components/*`, `lib.rs`, and the cargo-leptos `Cargo.toml` block broadly. Start
  it only **after** `02` and `03` have landed so the route taxonomy is stable;
  otherwise the islands conversion fights ongoing route edits.
- **Wave 3 — `06`.** After 01–05. Fan out nix / tests / docs, then merge and run
  `nix flake check`. The docs sub-layer performs the retirement bookkeeping.

### Serialization hazards (the only non-disjoint edits)

- **`crates/client/src/app.rs` is edited by `02`, `03`, and `04`.** Treat it as a
  serialization point: land `02` → `03` → `04` in that order, rebasing `app.rs`
  before each.
- **`Cargo.toml` (workspace + client features, `[[workspace.metadata.leptos]]`)
  is edited by `04`, `05`, and `06/sub-01`.** `04` toggles `experimental-islands`
  and `islands = true`; `05` adds the CSR profile; `06/sub-01` reconciles the nix
  build flags. Whichever lands second rebases.
- **`crates/client/src/api.rs` is created by `01` and re-shaped by `05`** (the
  server-fn vs REST data-source split). `01` lands first by construction.
- **`flake.nix` is edited by `05` and `06/sub-01`.** `05` adds the CSR package
  output; `06/sub-01` adds islands/SSG build flags. Coordinate on the package set.

## Whole-set acceptance criteria

- [ ] `rg 'todo!\("phase 03"\)' crates/` returns **zero** hits; every page renders
      real data under SSR (no runtime panic on `/posts`, `/projects`, `/todos`,
      `/series`).
- [ ] Each route resolves to exactly one documented rendering mode, recorded in a
      single source of truth (a `RENDER_MODES` table / doc page) that matches the
      `ssr=` attributes in `app.rs`.
- [ ] Static routes (`/posts*`, `/series*`, `/projects*`, `/about`, `/support`)
      are served from pre-rendered HTML and execute **no per-request database
      query** for an unchanged page; publishing an article regenerates/invalidates
      the relevant static route.
- [ ] The home page (`/`) streams: the shell + `intro` paint before the slowest
      section (activity) resolves (out-of-order streaming verified by a test that
      asserts incremental chunk delivery).
- [ ] In the islands build, static pages ship **no per-page WASM beyond the island
      runtime**; the `ThemeToggle` and mobile menu still work after load; a
      static-only page (e.g. `/about`) hydrates zero islands.
- [ ] A CSR build (`plinth-csr` output) boots without the Rust SSR server, fetches
      content from `/api/*` REST endpoints, and renders every page client-side.
- [ ] `nix flake check` is green for the default (SSR+hydrate / islands) build and
      the CSR build; clippy + fmt pass; named tests pass against sandbox Postgres.
- [ ] `crates/client` (WASM) still excludes `sqlx`/`reqwest-server` deps —
      `cargo tree -p plinth-client --target wasm32-unknown-unknown` stays clean.

## Global constraints (apply to every phase)

- **WASM safety is non-negotiable.** Server-function bodies are `#[cfg(feature =
  "ssr")]`-gated; the WASM/`hydrate`/`csr` builds must never pull `sqlx`,
  `pgvector`, `fastembed`, or the `plinth-forge` crate. Mirror the existing
  activity server-fn pattern (`crates/client/src/api.rs:86-121`): real body under
  `ssr`, `unreachable!()` otherwise.
- **One source of truth for rendering mode.** The `ssr=` attribute in `app.rs` and
  the documented mode table must agree. Do not encode a route's mode in two places
  that can drift.
- **Islands compose with `SsrMode`; they do not replace it.** A `SsrMode::Static`
  route can contain islands; a streamed route can contain islands. Islands mode
  only changes hydration **granularity** (which sub-trees ship WASM), not how the
  HTML is generated.
- **Verify the exact Leptos 0.8 static-rendering API before coding it.** The
  `SsrMode::Static` / `StaticRoute` / `prerender_params` surface has shifted across
  0.8 point releases. Check the version pinned in `Cargo.lock` and the installed
  `leptos_router` rustdoc, not memory, before writing route attributes.
- **Server functions stay the SSR/hydrate data source; the REST API stays the CSR
  data source.** Phase 05 introduces the split deliberately; do not let CSR depend
  on server-fn endpoints (there is no Leptos server in a pure-CSR deployment).

## Retirement (auto-retire trigger)

This plan **supersedes the unfinished tail of two prior plans** and absorbs their
remaining gap:

- [`postgres-migration`](../postgres-migration/) — server side shipped (migrations
  `0001`–`0006`, `sqlx`/`PgPool`, brick query rewrite). Its only outstanding work
  is the client server-fn query rewrite, which is exactly Phase 01 here.
- [`forge-activity`](../forge-activity/) — fully shipped (verified 2026-06-03); its
  durable docs already live at `docs/src/api/activity.md` and
  `docs/src/guides/activity.md`.

Both remain published until this plan lands. On a **clean `verify`** of this plan
(Phase 01 fills the stubs; Phases 02–06 land their acceptance criteria), Phase
06/sub-03 retires both prior plans: fold any residual durable knowledge into stable
docs, remove their `SUMMARY.md` entries and directories, then this plan's own
`verify` retires it in turn. Until then, do **not** delete the prior plans — they
carry the only record of the unfinished migration tail.

## Reference

- Originating request: retire shipped planning docs, fill gaps to enable
  auto-retirement, and introduce per-route rendering modes (design conversation
  2026-06-03).
- Current rendering wiring to mirror/extend: `crates/server/src/main.rs`
  (`generate_route_list` / `leptos_routes_with_context` / `shell` / fallback),
  `crates/client/src/app.rs` (`app_routes()`), `crates/client/src/api.rs`
  (server-fn pattern, activity reference impl), `Cargo.toml`
  `[[workspace.metadata.leptos]]`, `flake.nix` (`cargo leptos build`).
- Sibling plan sets for shape reference:
  [`forge-activity`](../forge-activity/), [`postgres-migration`](../postgres-migration/).

*Run the phases yourself (a fresh agent session per phase, fanning out per the
waves above). When done, prompt `verify` to audit the acceptance criteria — a clean
verify auto-retires `postgres-migration` and `forge-activity`.*
