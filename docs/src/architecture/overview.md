# Architecture Overview

Plinth is a four-crate Rust workspace built on Leptos 0.8 with SSR and WASM hydration.

## Workspace layout

```
crates/
  shared/    plinth-shared   Domain types shared by all crates
  client/    plinth-client   Leptos frontend (compiles to WASM)
  server/    plinth-server   Axum server with Leptos SSR
  cli/       plinth-cli      CLI for publishing and management
```

## Crate responsibilities

### plinth-shared

Domain types used across the stack: `BlogPost`, `BlogListItem`, `PortfolioItem`, `SiteConfig`, `Tag`, `SiteContent`, `ContentFormat`, `PublishArticleRequest`. Also contains `serde_helpers` for flexible database ID deserialization.

Compiled to both native (server/CLI) and `wasm32-unknown-unknown` (client).

### plinth-client

Leptos frontend compiled to WASM. Contains:

- **Pages**: `HomePage`, `BlogListPage`, `BlogPostPage`, `BlogTagPage`, `PortfolioPage`, `PortfolioDetailPage`, `AboutPage`, `NotFound`
- **Components**: `Header`, `Footer`, `ThemeToggle`
- **API module**: server function calls for data loading

Feature-gated: `csr` for client-side rendering, `hydrate` for SSR hydration mode.

### plinth-server

Axum HTTP server that renders Leptos components server-side and serves the hydrated WASM client. Contains:

- **Actors** (`actors/`): Kameo actors for in-memory caching and vector search
- **API** (`api/`): REST endpoints for admin, search, and image proxy
- **Services** (`services/`): Postgres access, migrations, row decoding, markdown processing
- **Server functions** (`server_fns/`): Leptos server functions for SSR data loading
- **Config** (`config.rs`): `PlinthConfig` loaded from `plinth.toml` with env var overrides

`AppState` holds `LeptosOptions`, actor refs (`CoreCache`, brick-specific caches, `VectorSearch`), the Postgres pool, HTTP client, and config.

### plinth-cli

CLI binary for content management:

- `publish` — publish Markdown or Typst articles with embedding generation
- `tag` — manage tags (list, add, remove)
- `content` — update site content blocks

Typst support includes local image scanning, Immich upload, and `typst-as-lib` compilation to HTML.

## SSR + WASM hydration flow

1. Browser requests a page
2. Axum receives the request and invokes Leptos SSR
3. Leptos server functions fetch data from the cache actors, which read from Postgres on cache misses
4. Server renders full HTML and streams it to the browser
5. Browser loads the WASM bundle and hydrates the page for interactivity
6. Subsequent navigation happens client-side via Leptos router

## Data flow: publishing an article

1. Author writes a `.md` or `.typ` file
2. CLI parses frontmatter, processes content (Markdown to HTML, or Typst compilation)
3. CLI generates a 384-dimensional fastembed vector embedding
4. CLI sends `POST /api/admin/articles` with content, metadata, and embedding
5. Server stores the article in Postgres, creates tag junction rows, syncs the read-side tag array, and invalidates caches
6. pgvector stores the embedding and the HNSW index supports approximate similarity search
