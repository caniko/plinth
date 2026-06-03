# Phase 02 — SSG: static-generate publish-only routes (`SsrMode::Static`)

> **Recommended Codex model: GPT 5.5 high**
>
> Complex design work in a near-foundational role. The mechanical part (adding
> `ssr=SsrMode::Static` attributes) is trivial, but the surrounding decisions are
> not: how parameterized routes (`/posts/:slug`, `/posts/tag/:tag`, `/series/:slug`,
> `/projects/:slug`) enumerate their static keys, when a statically-rendered page
> is regenerated/invalidated on publish, and how the static cache coexists with the
> `Cache-Control` middleware and the `file_and_error_handler` fallback. Getting the
> invalidation seam wrong ships stale content silently — a class of bug that
> doesn't surface in a quick smoke test. A medium model tends to add the attribute
> and declare victory without wiring regeneration. Not `max`: it's a bounded
> single-subsystem change with a clear reference (the activity SSR path) and no
> irreversible blast radius.

## Working tree

`/data/nvme0/can/Projects/solo/plinth` — depends on Phase 01 (all server functions
must return real data; a static route over a panicking loader pre-renders a panic).
**Shares `crates/client/src/app.rs` with Phases 03 and 04** — if Phase 03 has
already landed, `git pull`/rebase `app.rs` before editing the route table.

## Goal

Content that changes only on publish — `/posts`, `/posts/:slug`, `/posts/tag/:tag`,
`/series`, `/series/:slug`, `/projects`, `/projects/:slug`, `/about`, `/support` —
is rendered once and served as static HTML with no per-request database query, and
is regenerated (or invalidated) when the underlying content is published or edited
through the admin API. Dynamic routes are explicitly left on request-time SSR.

## Why this matters now

Phase 01 made every page render under request-time SSR, which means a hot blog post
re-runs its `SELECT` on every hit. Most of Plinth's surface is publish-cadence
content: a post changes when its author publishes, not when a reader requests it.
`leptos_routes_with_context` currently renders **every** route per request; there is
no `SsrMode` differentiation. Introducing `SsrMode::Static` for these routes is the
SSG mode the plan calls for and removes the redundant per-request DB work, while the
existing `s-maxage` CDN headers become a second, complementary cache layer rather
than the only one.

## Out of scope

- The home page (`/`) — that is streaming SSR, Phase 03. Do not mark `/` static; it
  embeds the always-changing activity strip.
- `/activity*` and `/todos*` — dynamic, stay on request-time SSR.
- Islands / partial hydration (Phase 04). Static routes will later *contain*
  islands, but this phase does not introduce the islands feature.
- Changing server-function bodies (Phase 01 owns the data layer).

## Plan

1. **Confirm the exact Leptos 0.8 static API.** Read the `leptos_router` version in
   `Cargo.lock` and its rustdoc for `SsrMode::Static`, `StaticRoute`, and the
   prerender/key API (`prerender_params` or equivalent). The surface changed across
   0.8 point releases — write the attributes against the pinned version, not memory.
2. **Define a single rendering-mode source of truth.** Add a small table (a doc
   comment or a `const`/match near `app_routes()` in `crates/client/src/app.rs`)
   listing each route → mode. This is the artifact the whole-set acceptance
   criterion checks against. Mode assignment for this phase:

   | Route | Mode |
   |---|---|
   | `/posts`, `/posts/tag/:tag`, `/series` | `SsrMode::Static` |
   | `/posts/:slug`, `/series/:slug` | `SsrMode::Static` (per-slug key) |
   | `/projects`, `/projects/:slug` | `SsrMode::Static` |
   | `/about`, `/support` | `SsrMode::Static` |
   | `/` | streaming (Phase 03) |
   | `/activity`, `/activity/:id`, `/todos*` | request-time SSR (unchanged) |

3. **Annotate the static routes** in `app_routes()`:
   `<Route path=path!("/posts/:slug") view=BlogPostPage ssr=SsrMode::Static(/* key */)/>`.
   For parameterized routes, supply the key/prerender-params function that enumerates
   slugs/tags from the DB at generation time (a server-side listing reused from Phase
   01's `get_all_series` / `get_blog_posts` / tag list).
4. **Wire regeneration/invalidation on publish.** The admin handlers
   (`bricks/blog/admin.rs`, `bricks/portfolio/admin.rs`) already invalidate their
   Kameo caches. Extend that path to also invalidate the static render for the
   affected route(s) — clear the cached static response so the next request
   re-generates it (consult the Leptos 0.8 static-route invalidation API; if it
   exposes no programmatic purge, fall back to keying static responses behind the
   existing `Cache-Control`/CDN purge and document that choice explicitly). The
   admin publish path is the single trigger; do not invalidate on read.
5. **Reconcile with the cache-control middleware.** Static HTML routes should keep a
   sane `Cache-Control` (e.g. `public, s-maxage=...`) so the CDN layer and the
   in-process static cache agree. Audit `cache_control_middleware`
   (`crates/server/src/main.rs`) so static routes are not accidentally sent
   `max-age=0` in a way that defeats SSG.
6. **Verify `generate_route_list(App)` carries the `SsrMode`.** In Leptos, the
   route list feeds `leptos_routes_with_context`; confirm static routes are rendered
   through the static path and not the per-request path. Note the existing caveat in
   `main.rs`: routes behind `<Suspense>` may not be discovered — static routes that
   wrap their body in `<Suspense>` need checking here.

## Acceptance criteria

- [ ] Every route in `app_routes()` has an explicit `ssr=` mode OR is documented in
      the mode table as intentionally request-time; the table and the attributes agree.
- [ ] A second request to `/about` and to a published `/posts/<slug>` executes **no
      SQL** (verify via DB query log / a counter assertion in a test, or by asserting
      identical byte-for-byte HTML served from cache without a DB round-trip).
- [ ] Publishing/editing an article via the admin API causes the next request to
      `/posts/<slug>` and `/posts` to reflect the change (regeneration/invalidation
      works); an unrelated post's static page is untouched.
- [ ] `cargo leptos build` succeeds; the static routes appear in the generated route
      list; `cargo clippy --workspace -- -D warnings` is clean.
- [ ] `/` still streams (not static), `/activity*` and `/todos*` still render per
      request — confirmed by the mode table and a smoke check.

## Files likely touched

- `crates/client/src/app.rs` (`app_routes()` route attributes + mode table).
- `crates/server/src/main.rs` (`generate_route_list` handling, static-route wiring
  in `leptos_routes_with_context`, `cache_control_middleware` reconciliation).
- `crates/server/src/bricks/blog/admin.rs`, `crates/server/src/bricks/portfolio/admin.rs`
  (publish-time static invalidation hook).

## Pitfalls

- **Marking a route static whose data is actually dynamic.** `/` and `/activity*`
  embed live data; a static render freezes it. Only publish-cadence content is safe.
- **Forgetting parameterized-route key generation.** `SsrMode::Static` on
  `/posts/:slug` without a key/prerender function either fails to build or generates
  nothing — the route silently 404s or falls through to the fallback. Enumerate keys
  from the DB.
- **Stale-after-publish.** If invalidation isn't wired to the admin publish path,
  editing a post shows the old version until process restart. This is the headline
  failure mode — test it explicitly (criterion 3).
- **`SsrMode::Static` API drift.** Do not copy a 0.7-era `StaticData`/`StaticRoute`
  signature; verify against the pinned 0.8 (Pitfall-driven by the global constraint).
- **Double-cache disagreement.** The CDN `s-maxage` and the in-process static cache
  can disagree on TTL; make the in-process static render authoritative and let CDN
  TTL be ≤ it, or document the chosen relationship.

## Reference

- Global constraint on verifying the Leptos 0.8 static API: plan
  [README](./README.md) "Global constraints".
- SSR wiring being differentiated: `crates/server/src/main.rs`
  (`generate_route_list` / `leptos_routes_with_context` / `cache_control_middleware`).
- Prereq data layer: [01-ssr-data-path.md](./01-ssr-data-path.md). Serializes with
  [03-streaming-home.md](./03-streaming-home.md) and [04-islands.md](./04-islands.md)
  on `crates/client/src/app.rs`.
