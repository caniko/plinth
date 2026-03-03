# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Development server (SSR + hot reload)
cargo leptos watch

# Build production release
nix build .#plinth

# Run all checks (build + clippy + fmt + tests) — same as CI
nix flake check

# Run tests locally (excludes client crate — it targets WASM)
cargo test --workspace --exclude plinth-client

# Run a single test
cargo test --package plinth-server test_name

# Format
cargo fmt --all

# Clippy
cargo clippy --all-targets -- --deny warnings
```

New files must be `git add`'ed before `nix flake check` can see them (Nix uses the git index).

## Key Technical Constraints

**SurrealDB SCHEMAFULL + Serde**: `db.create("table").content(rust_struct)` fails for `datetime` fields because `chrono::DateTime<Utc>` serializes as an ISO string. Use raw SQL with `time::now()` instead. Record IDs returned as `Thing` type need `deserialize_flexible_id`. `.bind()` requires `'static` — use `.bind(("key", value.to_string()))`.

**Nix sandbox**: No network access, no CA certificates. `reqwest::Client::new()` panics — use `Client::builder().build()` and handle errors. `fastembed::TextEmbedding::try_new()` downloads models at runtime and will fail. All tests must avoid it.

**Leptos features**: The `ssr` feature gates server-only deps (axum, tokio, surrealdb, actors). Client compiles to WASM without these. The workspace uses `default-features = false` for the client crate dependency.

**Raw string literals**: `r#"..."#` terminates at any `"#` inside — use `r##"..."##` when content contains `"#` (common with markdown headings).

## Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `SURREALDB_PATH` | `database.db` | DB file path |
| `PLINTH_API_KEY` | `dev_api_key_change_in_production` | Admin API auth (Bearer token) |
| `LEPTOS_SITE_ADDR` | `127.0.0.1:3000` | Server bind address |
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

## Logo & Favicons

Source logo lives in `logo/plinth-logo.svg`. Derived assets in `public/`:
- `public/plinth-logo.svg` — served at `/plinth-logo.svg`, used in site header and footer
- `public/favicon.svg` — square version (logo centered in 1478x1478 canvas), used as primary favicon
- `public/favicon-{16,32,48,180,192,512}x{size}.png` — rasterized from `favicon.svg`

Docs site copies in `docs/static/`: `favicon-16x16.png`, `favicon-32x32.png`, `apple-touch-icon.png`, `plinth-logo.svg`.

Regenerate all PNGs and sync to docs: `just favicons`

## CI

Woodpecker CI on Codeberg. Runs `nix flake check` (which includes build, clippy, fmt, and cargo test) on push/PR to main/poc/develop. Release builds only on main.
