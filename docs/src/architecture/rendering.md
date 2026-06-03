# Rendering

Plinth builds the same Leptos app for several rendering targets. The route-level
source of truth is `app_routes()` in `crates/client/src/app.rs`; the table below
documents the default all-bricks build.

## Route Modes

| Route | Mode | Why |
|-------|------|-----|
| `/` | Streaming SSR, `SsrMode::OutOfOrder` | The home page aggregates multiple bricks, so the shell can stream while slower sections resolve. |
| `/about` | SSG, `SsrMode::Static` | Site content changes only when an admin publishes the `about` key. |
| `/support` | SSG, `SsrMode::Static` | Site content changes only when an admin publishes the `support` key. |
| `/posts` | SSG, `SsrMode::Static` | Blog index content changes at publish cadence. |
| `/posts/:slug` | SSG, `SsrMode::Static` with prerendered slugs | Published posts are addressable static content. |
| `/posts/tag/:tag` | SSG, `SsrMode::Static` with prerendered tag names and slugs | Tag pages change when posts or tags are published. |
| `/series` | SSG, `SsrMode::Static` | Series membership changes at blog publish cadence. |
| `/series/:slug` | SSG, `SsrMode::Static` with prerendered series slugs | Series pages are derived from published posts. |
| `/projects` | SSG, `SsrMode::Static` | Portfolio index content changes at publish cadence. |
| `/projects/:slug` | SSG, `SsrMode::Static` with prerendered project slugs | Portfolio entries are stable published content. |
| `/activity` | Dynamic SSR, `SsrMode::OutOfOrder` | Activity is ranked and refreshed at request time. |
| `/activity/:id` | Dynamic SSR, `SsrMode::OutOfOrder` | Individual activity entries may refresh forge metadata. |
| `/todos` | Dynamic SSR, `SsrMode::OutOfOrder` | Todo ordering and completion state are ranked/user-curated dynamic data. |
| `/todos/tag/:tag` | Dynamic SSR, `SsrMode::OutOfOrder` | Tagged todo lists are request-time ranked views. |
| `/todos/:slug` | Dynamic SSR, `SsrMode::OutOfOrder` | Todo detail pages expose mutable state. |

Custom builds with one or more bricks disabled keep the static site-content
routes and omit the disabled brick routes at compile time.

## Decision Rule

Use static generation for publish-cadence content: site pages, blog posts,
series, and portfolio entries. Use streaming SSR for multi-source aggregate
pages where independent sections can resolve at different speeds. Use dynamic
SSR for user-curated, ranked, or externally refreshed data. Use islands for
interactive widgets that need client behavior without hydrating the whole app.
Use the CSR build for serverless or static-host deployments where a separate
Plinth API server supplies content.

## Islands Boundary

The SSR/hydrate build enables Leptos islands. The app shell and page bodies are
server-rendered, while the interactive header boundary hydrates on the client:
`Header` is an island in non-CSR islands builds, and `ThemeToggle` is always an
island. This keeps the mobile navigation toggle and theme persistence
interactive without hydrating read-only content pages.

## Static Regeneration

Static routes use Leptos `StaticRoute::regenerate` streams. Admin publish paths
send invalidation events after a successful write:

| Admin write | Invalidated static routes |
|-------------|---------------------------|
| Blog publish, update, delete, or tag change | `/posts`, matching `/posts/:slug`, matching `/posts/tag/:tag`, `/series`, and matching `/series/:slug` when a series is involved |
| Portfolio publish | `/projects` and matching `/projects/:slug` |
| Site content publish | Matching site-content route such as `/about` or `/support` |

The invalidation signal is narrow: request-time routes do not participate in
static regeneration because they are rendered dynamically.

## Build Targets

Build the default SSR/islands target with:

```bash
cargo leptos build
```

This produces the server binary and the browser WASM/CSS assets for the normal
deployment package. The same route table controls server rendering and the
hydration boundary.

Build the client-only CSR target with:

```bash
nix build .#plinth-csr
```

The CSR package emits static files only. It renders routes in the browser and
uses public `GET /api/*` endpoints instead of Leptos server functions. Use it
for static previews or static hosting paired with a separate Plinth API server;
prefer the default SSR package when the deployment should serve rendered HTML,
feeds, admin APIs, and proxied images from one process.

## WASM Safety

Server-only dependencies stay behind the `ssr` feature. The client crate is
included with `default-features = false`, and browser builds must not pull in
Axum, Tokio server actors, SQLx, forge refresh code, or other server-only
runtime dependencies. Data that the browser needs comes through shared types,
hydrated resources, server functions in SSR builds, or public REST calls in the
CSR build.
