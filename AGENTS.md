# AGENTS.md

This file provides guidance to coding agents when working with code in this repository.

## Build & Development Commands

```bash
# Development server (Dioxus fullstack SSR + hot reload)
dx serve --web --fullstack

# Build production release
nix build .#plinth

# Run all checks (build + clippy + fmt + tests) — same as CI
nix flake check

# Run tests locally (the legacy Leptos client targets WASM)
cargo test --workspace --exclude plinth-client

# Run a single test
cargo test --package plinth-server test_name

# Format
cargo fmt --all

# Clippy
cargo clippy --all-targets -- --deny warnings
```

New files must be `git add`'ed before `nix flake check` can see them (Nix uses the git index).

## Bricks (Modular Features)

Content types are optional **bricks** gated by Cargo feature flags. Default builds include all bricks.

| Brick | Feature Flag | Content |
|-------|-------------|---------|
| Blog | `brick-blog` | Blog posts, series, search, RSS feeds, CLI publish/tag |
| Portfolio | `brick-portfolio` | Project showcase items, RSS feed |
| Todo | `brick-todo` | Bucket list / TODO items, CLI todo commands |
| Activity | `brick-activity` | Forge activity list/detail, refresh, RSS feed |

**Core (always present):** DB, config, tags, site content, health check, image proxy, auth, middleware.

```bash
# Build with all bricks (default)
dx serve --web --fullstack

# Build with specific bricks only
cargo check -p plinth-web --no-default-features --features "server,web,brick-blog,brick-portfolio"

# Check compilation without a brick
cargo check -p plinth-web --no-default-features --features "server,brick-blog,brick-todo"
```

**Brick code lives in:**
- Server: `crates/server/src/bricks/{blog,portfolio,todo,activity}/` (cache actors, admin handlers, migrations)
- Web: pages and fullstack loaders in `crates/dioxus-ui/src/` are `#[cfg]`-gated
- Shared: type modules in `crates/shared/src/` are `#[cfg]`-gated
- CLI: commands in `crates/cli/src/main.rs` are `#[cfg]`-gated

**Adding a new brick:** Implement the `Brick` trait in `crates/server/src/bricks/mod.rs`, add a feature flag to all 4 `Cargo.toml` files, and register in `enabled_bricks()`.

## Key Technical Constraints

**Postgres + sqlx**: Server persistence uses PostgreSQL with migrations in `crates/server/migrations/`. Integration tests should use `#[sqlx::test]` or the shared isolated-DB helper so migrations run against a fresh database. `DATABASE_URL` is the runtime application database; `SQLX_TEST_DB_BASE_URL` can override the base URL used by sqlx test databases.

**Nix sandbox**: No network access, no CA certificates. `reqwest::Client::new()` panics — use `Client::builder().build()` and handle errors. `fastembed::TextEmbedding::try_new()` downloads models at runtime and will fail. All tests must avoid it.

**Dioxus features**: The `server` feature gates server-only deps (Axum, Tokio,
SQLx, actors). The `web` feature compiles the browser/WASM client without
those dependencies. The workspace uses `default-features = false` for the
web crate's target-specific dependency graph. The legacy Leptos crate remains
only as a rollback/test seam until the final migration cleanup.

**Raw string literals**: `r#"..."#` terminates at any `"#` inside — use `r##"..."##` when content contains `"#` (common with markdown headings).

## Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `DATABASE_URL` | `postgres://plinth:plinth@localhost:5432/plinth` | PostgreSQL connection string |
| `SQLX_TEST_DB_BASE_URL` | _(falls back to `DATABASE_URL`)_ | Base PostgreSQL URL for isolated sqlx test databases |
| `PLINTH_API_KEY` | `dev_api_key_change_in_production` | Admin API auth (Bearer token) |
| `PLINTH_SITE_ADDR` | `127.0.0.1:3000` | Dioxus server bind address |
| `DIOXUS_PUBLIC_PATH` | package `site/` | Packaged Dioxus public assets |
| `PLINTH_RENDER_CACHE_DIR` | state-directory render-cache | Durable rendered-page cache |
| `RUST_LOG` | `info` | Log level |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | _(none)_ | Enables OTLP telemetry export |
| `PLINTH_API_URL` | `http://localhost:3000` | CLI target server |
| `IMMICH_API_URL` | _(none)_ | Immich server URL (enables image proxy on server, image upload on CLI) |
| `IMMICH_API_KEY` | _(none)_ | Immich API key for image proxy/upload |
| `PLAUSIBLE_DOMAIN` | _(none)_ | Site domain for Plausible analytics |
| `PLAUSIBLE_SCRIPT_URL` | _(none)_ | URL to self-hosted Plausible script |

## Typst Blog Posts

Blog posts can be authored in Typst (`.typ`) as well as Markdown. The CLI detects format by extension.

**Typst frontmatter** uses comment-based YAML (mirrors the markdown experience):
```typst
// ---
// title: My Post
// tags: ["rust", "typst"]
// description: A post about something
// ---
```

**Image placement** uses custom functions defined in `templates/blog.typ`:
- `#blog-image("photo.jpg", placement: "inline", caption: "...", alt: "...")` — placements: `inline`, `hero`, `float-left`, `float-right`, `full-width`
- `#hero-image("photo.jpg", alt: "...")` — convenience for `placement: "hero"`
- `#gallery((src: "a.jpg"), (src: "b.jpg"))` — grid layout

**Publishing flow** for `.typ` files:
1. CLI extracts comment-based YAML frontmatter
2. Scans for local image references (`#blog-image("local.jpg", ...)`)
3. Uploads local images to Immich, gets asset IDs
4. Replaces local paths with `/api/images/{asset_id}` proxy URLs
5. Compiles Typst to HTML via `typst-as-lib` + `typst-html`
6. Generates fastembed embedding from text content
7. Sends pre-rendered HTML + metadata to server API

**Image proxy**: `GET /api/images/{asset_id}?size=original|preview|thumbnail` — server fetches from Immich and streams to readers with 1-year cache headers.

## Portfolio Publishing

This is how you add a new tool to the portfolio: ship a small `portfolio.toml`
with the tool repository, then publish it to a deployed Plinth instance with the
admin CLI.

```bash
PLINTH_API_URL=https://example.com \
PLINTH_API_KEY=... \
plinth-cli portfolio publish path/to/portfolio.toml
```

The CLI posts to `POST /api/admin/portfolio` with Bearer auth and the server
upserts `portfolio_items` by `slug`, renders markdown `content` to
`html_content`, and invalidates the portfolio cache. Re-publishing the same
manifest updates the existing row instead of creating a duplicate.

`portfolio.toml` schema:

```toml
# Optional. If omitted, generated from title.
slug = "my-tool"

title = "My Tool"
description = "Short portfolio-card summary."
tech_stack = ["Rust", "Dioxus", "Postgres"]
date = "2026-05-28T00:00:00Z"

# Optional fields.
content = """
# My Tool

Long-form markdown description.
"""
link = "https://codeberg.org/example/my-tool"
demo = "https://example.com/my-tool"
image_url = "https://example.com/assets/my-tool.png"
featured = false
order = 0
content_format = "markdown"
```

Required fields: `title`, `description`, `tech_stack`, and `date`. `slug` is
optional but must be present or derivable from `title`. `content_format`
defaults to `markdown`; no other portfolio content format is currently
accepted. `image_url` must already point at hosted media; the portfolio command
does not upload images.

## Logo & Favicons

Source logo lives in `logo/plinth-logo.svg`. Derived assets in `public/`:
- `public/plinth-logo.svg` — served at `/plinth-logo.svg`, used in site header and footer
- `public/favicon.svg` — square version (logo centered in 1478x1478 canvas), used as primary favicon
- `public/favicon-{16,32,48,180,192,512}x{size}.png` — rasterized from `favicon.svg`

Docs site copies in `docs/static/`: `favicon-16x16.png`, `favicon-32x32.png`, `apple-touch-icon.png`, `plinth-logo.svg`.

Regenerate all PNGs and sync to docs: `just favicons`

## CI

Forgejo Actions on Codeberg run on the self-hosted `atlas` runner. The CI workflow runs `nix flake check`, builds release packages, and pushes release builds to Attic on main/tags when the runner has an onboarded token.
