# Phase 05 — A real CSR build profile (client-only, REST-backed)

> **Recommended Codex model: GPT 5.5 medium**
>
> Moderate complexity, sub-agent role. The deliverable is bounded — make the
> already-declared-but-inert `csr` feature into a buildable, runnable client-only
> target — but it has one genuine design decision: in a pure-CSR deployment there is
> no Leptos server, so the data layer cannot be Leptos server functions; it must hit
> the existing REST JSON API. That requires a clean compile-time split of the data
> source (`server-fn` under `ssr`/`hydrate`, `fetch`-to-`/api/*` under `csr`) without
> duplicating the page components. A medium model handles the split and the build
> wiring; the risk is low and reversible (a new, additive build target). Not high:
> no cross-system invariant, and the REST endpoints already exist.

## Working tree

`/data/nvme0/can/Projects/solo/plinth` — depends on Phase 01 (the data-fetching API
surface must be complete before it can be abstracted). Touches
`crates/client/src/api.rs` (re-shaped by Phase 01 first — land 01 before this),
`Cargo.toml`, and `flake.nix` (shared with Phase 06/sub-01 — coordinate on the
package set).

## Goal

`csr` becomes a real, documented build target: a client-only WASM bundle that boots
without the Rust SSR server and fetches all content from the existing `/api/*` REST
endpoints, suitable for static-host / preview / offline deployment. A `plinth-csr`
artifact builds via nix and renders every page client-side.

## Why this matters now

The client crate declares `default = ["csr", …]` and a `csr = ["leptos/csr"]`
feature, but it is **inert**: `cargo-leptos` overrides it with `lib-features =
["hydrate", …]`, so no CSR bundle is ever produced and the feature is dead weight.
The plan calls for CSR as a first-class mode. The blocker is the data layer: every
page fetches via Leptos server functions, which in CSR compile to HTTP POSTs to
`/api/<FnName>` served by the Leptos server — but a pure-CSR deployment has no such
server. The server already exposes a conventional REST API (`/api/portfolio`,
`/api/activity`, blog/search endpoints) that a CSR build can target instead. This
phase makes the `csr` feature mean something and gives Plinth a serverless
deployment story.

## Out of scope

- Removing SSR/hydrate (the default) — CSR is an *additional* target, not a
  replacement.
- Islands (Phase 04) — orthogonal; CSR ships a full client bundle by definition.
- Adding new REST endpoints unless an existing server-fn has no REST equivalent —
  prefer reusing the REST surface the server already serves.
- Auth/admin in CSR — the CSR target is the public read-only site.

## Plan

1. **Inventory the REST API parity.** List the server-fn data needs from
   `crates/client/src/api.rs` (post Phase 01) against the REST endpoints the server
   already exposes (`crates/server/src/main.rs` public API router: `/api/portfolio`,
   `/api/activity`, search, feeds…). Note any data a page needs that has a server-fn
   but no REST endpoint — those are the only places a small REST handler must be
   added.
2. **Introduce a compile-time data-source split** in `api.rs`. Keep each public
   function signature stable (pages call `api::get_blog_posts()` regardless of
   build). Behind it:
   - under `feature = "ssr"` (and the hydrate path): the existing `#[server]` impl.
   - under `feature = "csr"`: a body that `fetch`es `GET /api/<resource>` and
     deserializes the same `plinth_shared` types.
   Use `#[cfg(feature = "csr")]` / `#[cfg(not(feature = "csr"))]` arms or a thin
   `data_source` module with two impls. Do **not** fork the page components.
3. **Pick the CSR build tool.** `cargo-leptos` is SSR-oriented; a pure-CSR bundle is
   typically built with Trunk or `wasm-bindgen` + a static `index.html`. Decide:
   either add a `Trunk.toml` + CSR `index.html` that loads the `csr`-featured WASM,
   or a `cargo-leptos` CSR profile if the pinned version supports a client-only
   output. Document the choice; keep the SSR build path untouched.
4. **Provide the HTML shell for CSR.** CSR needs a static `index.html` that mounts
   the app (`leptos::mount::mount_to_body(App)` under `csr`) — distinct from the
   server `shell()`. Add a `csr`-gated mount entry in `lib.rs` (sibling to
   `hydrate()`), and the static shell that references the built WASM/JS + CSS.
5. **Point CSR at an API base URL.** The CSR bundle needs to know where `/api/*`
   lives (same origin for static-host-behind-proxy, or a configured base). Read it
   from a build-time env or a small runtime config; default to same-origin.
6. **Add a nix package output** `plinth-csr` (coordinate with Phase 06/sub-01):
   builds the `csr` WASM bundle + static shell + CSS into a servable static dir.

## Acceptance criteria

- [ ] `crates/client` builds with `--no-default-features --features csr` (plus brick
      features) for `wasm32-unknown-unknown`; `cargo clippy` clean for that feature set.
- [ ] The CSR bundle, served as static files (no `plinth-server` running), loads and
      renders `/`, `/posts`, `/projects`, `/activity` with data fetched from `/api/*`
      REST endpoints (verified by pointing it at a running API and loading in a browser
      or headless check).
- [ ] The default SSR+hydrate (and islands) build is unchanged — same server-fn data
      path, no regression (`cargo leptos build` still green).
- [ ] `cargo tree -p plinth-client --target wasm32-unknown-unknown --no-default-features
      --features csr,...` excludes `sqlx`/`pgvector`/`fastembed`/`plinth-forge`.
- [ ] `nix build .#plinth-csr` produces a static site directory.
- [ ] A doc note states when to use CSR (serverless/static-host/preview) vs the
      default SSR build (Phase 06/sub-03 folds this into the rendering doc).

## Files likely touched

- `crates/client/src/api.rs` (the server-fn-vs-REST data-source split).
- `crates/client/src/lib.rs` (a `csr`-gated `mount_to_body` entry alongside `hydrate()`).
- `Cargo.toml` (CSR profile / feature reconciliation; shared with Phases 04 & 06).
- `flake.nix` (the `plinth-csr` package output; shared with Phase 06/sub-01).
- A `Trunk.toml` + CSR `index.html` (or a cargo-leptos CSR profile), new.
- Possibly `crates/server/src/main.rs` / a brick api module — only to add a REST
  endpoint for any datum that has a server-fn but no REST equivalent.

## Pitfalls

- **CSR calling server-fn endpoints.** If the `csr` build keeps the `#[server]` data
  path, it POSTs to `/api/<FnName>` which only exists when the Leptos server runs —
  the whole point of CSR is to not need that server. Route CSR through REST.
- **Forking the page components.** The split belongs in the data layer, not the
  views. If you find yourself duplicating `BlogListPage`, stop — push the `#[cfg]`
  down into `api.rs`.
- **WASM bloat / leak.** A careless `csr` feature can pull a server-only dep into the
  WASM tree. Re-run the `cargo tree` check for the `csr` feature set specifically,
  not just the hydrate set.
- **Two mount entrypoints clashing.** `hydrate()` (`hydrate_body`) and the CSR
  `mount_to_body` must be mutually `#[cfg]`-exclusive, or the wrong one runs.
- **API base URL in static hosting.** Same-origin works behind a reverse proxy; a
  truly static host on a different origin needs CORS on the API and a configured base
  URL. Document the deployment assumption.

## Reference

- Existing REST API surface to target: `crates/server/src/main.rs` public API router
  (`/api/portfolio`, `/api/activity`, search, feeds).
- Server-fn data path being split: `crates/client/src/api.rs` (post Phase 01).
- Hydrate entry to mirror for the CSR mount: `crates/client/src/lib.rs:10-18`.
- Prereq: [01-ssr-data-path.md](./01-ssr-data-path.md). Nix packaging coordination:
  [06-nix-tests-docs/sub-01-nix.md](./06-nix-tests-docs/sub-01-nix.md).
