+++
title = "Contributing"
description = "Guidelines for contributing to Plinth"
weight = 30
+++

## Code style

- Format with `cargo fmt --all`
- Lint with `cargo clippy --all-targets -- --deny warnings`
- Both are enforced in CI

## Before submitting

Run the full CI check locally:

```bash
nix flake check
```

This runs build, clippy, formatting, and all tests in one command. It matches exactly what CI runs.

If you've added new files, remember to `git add` them first — Nix only sees files in the git index.

## Architecture guidelines

- **Shared types go in `plinth-shared`** — anything used by both server and client
- **Feature-gate server deps** with `#[cfg(feature = "ssr")]` — the client compiles to WASM without tokio, axum, etc.
- **Actor messages** should be small and cloneable — actors handle concurrency
- **Use raw SQL for SurrealDB datetime fields** — `chrono::DateTime<Utc>` doesn't serialize correctly via `.content()`
- **Use `r##"..."##`** for raw strings containing `"#` (Markdown headings in SQL)

## CI

Woodpecker CI runs on Codeberg for push and PR events to `main`, `poc`, and `develop` branches:

1. `nix flake check` — build, clippy, fmt
2. `cargo test --workspace --all-features`
3. Release build (main branch only)
