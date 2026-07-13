# Rendering

Plinth builds the same Dioxus app for browser and native targets. The route
source of truth is `Route` in `crates/dioxus-ui/src/lib.rs`; the table below
documents the default all-bricks build.

## Route Modes

| Route | Mode | Why |
|-------|------|-----|
| `/` | Streaming SSR (`StreamingMode::OutOfOrder`) | The home page is the only route allowed to introduce server futures; the shell can stream while its content resolves. |
| `/about` | Cached SSR | Site content changes only when an admin publishes the `about` key; external page-cache invalidation is explicit. |
| `/support`, `/posts`, `/posts/:slug`, `/posts/tag/:tag`, `/series`, `/series/:slug`, `/projects`, `/projects/:slug` | Cached SSR | Publish-cadence content is served through `PageCache`; writes invalidate affected keys/tags. |
| `/activity`, `/activity/:id` | Fresh SSR | Activity is ranked and refreshed at request time. |
| `/todos`, `/todos/tag/:tag`, `/todos/:slug` | Fresh SSR | Todo ordering and completion state are mutable. |

Custom builds with one or more bricks disabled keep the static site-content
routes and omit the disabled brick routes at compile time.

## Decision Rule

Use external page caching for publish-cadence content: site pages, blog posts,
series, and portfolio entries. Use streaming SSR only for the home aggregate.
Use fresh SSR for user-curated, ranked, or externally refreshed data. Dioxus
hydration is kept at the app boundary; interactive widgets use ordinary Dioxus
signals and event handlers.

## Islands Boundary

The SSR/hydrate build renders the same Dioxus route tree on both targets. The
mobile menu is a local signal boundary, so read-only content does not need a
separate framework island.

## Static Regeneration

Admin publish paths send page-cache invalidation events after a successful write:

| Admin write | Invalidated static routes |
|-------------|---------------------------|
| Blog publish, update, delete, or tag change | `/posts`, matching `/posts/:slug`, matching `/posts/tag/:tag`, `/series`, and matching `/series/:slug` when a series is involved |
| Portfolio publish | `/projects` and matching `/projects/:slug` |
| Site content publish | Matching site-content route such as `/about` or `/support` |

The invalidation signal is narrow: fresh activity/todo routes are never cached.

## Build Targets

Build the default SSR/islands target with:

```bash
cargo build --package plinth-web --bin plinth-web --features server,brick-blog,brick-portfolio,brick-todo,brick-activity
```

The Nix package additionally builds the browser target, runs `wasm-bindgen`,
and emits `target/site/pkg` plus Tailwind CSS. The same route table controls
server rendering and the hydration boundary.

Build the client-only CSR target with:

```bash
nix build .#plinth-csr
```

The CSR package emits static files only. It renders routes in the browser and
uses public `GET /api/*` endpoints instead of server-only Dioxus functions. Use it
for static previews or static hosting paired with a separate Plinth API server;
prefer the default SSR package when the deployment should serve rendered HTML,
feeds, admin APIs, and proxied images from one process.

## WASM Safety

Server-only dependencies stay behind the Dioxus `server` feature. The web crate
is built with `default-features = false`, and browser builds must not pull in
Axum, Tokio server actors, SQLx, forge refresh code, or other server-only
runtime dependencies. Data that the browser needs comes through shared types,
hydrated resources, generated fullstack endpoints, or public REST calls in the
CSR build.
