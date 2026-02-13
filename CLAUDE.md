# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Development server (SSR + hot reload)
cargo leptos watch

# Build production release
nix build .#personal-website

# Run all checks (build + clippy + fmt + tests) — same as CI
nix flake check

# Run tests locally (excludes client crate — it targets WASM)
cargo test --workspace --exclude client

# Run a single test
cargo test --package server test_name

# Format
cargo fmt --all

# Clippy
cargo clippy --all-targets -- --deny warnings
```

New files must be `git add`'ed before `nix flake check` can see them (Nix uses the git index).

## Architecture

Four-crate Rust workspace: a Leptos 0.8 full-stack app with SSR + WASM hydration.

- **`crates/shared`** — Domain types (`BlogPost`, `PortfolioItem`, `PublishArticleRequest`) shared between all crates. Contains `serde_helpers::deserialize_flexible_id` for SurrealDB `Thing` → `Option<String>` conversion.
- **`crates/client`** — Leptos frontend compiled to WASM. Pages in `pages/`, components in `components/`. Features: `csr` (default), `hydrate` (SSR mode).
- **`crates/server`** — Axum HTTP server with Leptos SSR. Has both `lib.rs` (for integration test imports) and `main.rs`. `AppState` holds `LeptosOptions`, actor refs, and DB handle.
  - `actors/` — Kameo actors: `ContentCache` (in-memory blog/portfolio cache), `VectorSearch` (fastembed semantic search)
  - `api/` — REST endpoints: `admin.rs` (auth-protected article publishing), `search.rs` (semantic search)
  - `services/` — `db.rs` (SurrealDB init/schema/seed), `markdown_processor.rs` (frontmatter + HTML)
  - `server_fns/` — Leptos server functions for SSR data loading
  - `observability.rs` — Tracing + optional OTLP export
- **`crates/cli`** — `blog-cli` binary for publishing markdown articles with embeddings via the admin API.

## Key Technical Constraints

**SurrealDB SCHEMAFULL + Serde**: `db.create("table").content(rust_struct)` fails for `datetime` fields because `chrono::DateTime<Utc>` serializes as an ISO string. Use raw SQL with `time::now()` instead. Record IDs returned as `Thing` type need `deserialize_flexible_id`. `.bind()` requires `'static` — use `.bind(("key", value.to_string()))`.

**Nix sandbox**: No network access, no CA certificates. `reqwest::Client::new()` panics — use `Client::builder().build()` and handle errors. `fastembed::TextEmbedding::try_new()` downloads models at runtime and will fail. All tests must avoid it.

**Leptos features**: The `ssr` feature gates server-only deps (axum, tokio, surrealdb, actors). Client compiles to WASM without these. The workspace uses `default-features = false` for the client crate dependency.

**Raw string literals**: `r#"..."#` terminates at any `"#` inside — use `r##"..."##` when content contains `"#` (common with markdown headings).

## Test Organization

68 tests total. Unit tests live in `#[cfg(test)]` modules within source files. Integration tests in `crates/server/tests/`:
- `db_integration.rs` — SurrealDB operations with `surrealdb::engine::local::Mem` (in-memory, no disk)
- `content_cache_integration.rs` — ContentCache actor with in-memory DB

The `client` crate is excluded from test runs (`--exclude client`) because it targets `wasm32-unknown-unknown`.

## Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `SURREALDB_PATH` | `database.db` | DB file path |
| `BLOG_API_KEY` | `dev_api_key_change_in_production` | Admin API auth (Bearer token) |
| `LEPTOS_SITE_ADDR` | `127.0.0.1:3000` | Server bind address |
| `RUST_LOG` | `info` | Log level |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | _(none)_ | Enables OTLP telemetry export |
| `BLOG_API_URL` | `http://localhost:3000` | CLI target server |

## CI

Woodpecker CI on Codeberg. Runs `nix flake check` (which includes build, clippy, fmt, and cargo test) on push/PR to main/poc/develop. Release builds only on main.
