# Architecture Overview

Plinth is a Rust workspace built on Dioxus 0.7.9 with fullstack SSR and WASM hydration.

## Workspace layout

```
crates/
  shared/    plinth-shared   Domain types shared by all crates
  dioxus-ui/ plinth-web      Dioxus frontend and fullstack server entrypoint
  client/    plinth-client   Legacy Leptos frontend retained during rollback window
  server/    plinth-server   Framework-neutral Axum API, actors, and bootstrap
  cli/       plinth-cli      CLI for publishing and management
```

## Crate responsibilities

### plinth-shared

Domain types used across the stack: `BlogPost`, `BlogListItem`, `PortfolioItem`, `SiteConfig`, `Tag`, `SiteContent`, `ContentFormat`, `PublishArticleRequest`. Also contains `serde_helpers` for flexible database ID deserialization.

Compiled to both native (server/CLI) and `wasm32-unknown-unknown` (client).

### plinth-web

Dioxus frontend compiled to WASM and native fullstack server. Contains:

- **Typed routes**: the complete public URL contract in `crates/dioxus-ui/src/lib.rs`
- **Loaders**: Dioxus fullstack server functions backed by the existing read
  model (cache actors for publish-cadence bricks; direct PostgreSQL plus a
  non-blocking refresh poke for activity)
- **Cache policy**: explicit cached/fresh/streaming route policy and invalidation keys
- **Server entrypoint**: API composition, static assets, security headers, and SSR

Feature-gated: `web` for browser hydration, `server` for native SSR, and one
feature per content brick.

### plinth-server

Axum backend library consumed by the Dioxus entrypoint. Contains:

- **Actors** (`actors/`): Kameo actors for in-memory caching and vector search
- **API** (`api/`): REST endpoints for admin, search, and image proxy
- **Services** (`services/`): Postgres access, migrations, row decoding, markdown processing
- **Bootstrap/router**: framework-neutral initialization and stable `/api/*` routes
- **Config** (`config.rs`): `PlinthConfig` loaded from `plinth.toml` with env var overrides

`AppState` holds actor refs (`CoreCache`, brick-specific caches, `VectorSearch`),
the Postgres pool, HTTP client, and config. The legacy Leptos options field is
compiled only behind the rollback feature and is not present in the Dioxus
production feature graph.

### plinth-cli

CLI binary for content management:

- `publish` — publish Markdown or Typst articles with embedding generation
- `tag` — manage tags (list, add, remove)
- `content` — update site content blocks

Typst support includes local image scanning, Immich upload, and `typst-as-lib` compilation to HTML.

## SSR + WASM hydration flow

1. Browser requests a page
2. Axum receives the request and invokes Dioxus SSR
3. Dioxus fullstack loaders fetch data from the cache actors, which read from Postgres on cache misses
4. Server renders full HTML (with out-of-order streaming reserved for the home aggregate)
5. Browser loads the WASM bundle and hydrates the page for interactivity
6. Subsequent navigation happens client-side via Dioxus router and generated server-function endpoints

## Data flow: publishing an article

1. Author writes a `.md` or `.typ` file
2. CLI parses frontmatter, processes content (Markdown to HTML, or Typst compilation)
3. CLI generates a 384-dimensional fastembed vector embedding
4. CLI sends `POST /api/admin/articles` with content, metadata, and embedding
5. Server stores the article in Postgres, creates tag junction rows, syncs the read-side tag array, and invalidates caches
6. pgvector stores the embedding and the HNSW index supports approximate similarity search
