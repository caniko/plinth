+++
title = "Dev Environment"
description = "Set up the development environment"
weight = 10
+++

## Using Nix (recommended)

```bash
git clone https://codeberg.org/caniko/plinth.git
cd plinth
nix develop
```

The dev shell provides:
- Rust nightly with `wasm32-unknown-unknown` target
- cargo-leptos, wasm-bindgen-cli, binaryen
- Tailwind CSS standalone binary
- SurrealDB
- OpenSSL, ONNX Runtime, libclang
- Mold linker (Linux)
- Zola (for documentation development)

## Development server

```bash
cargo leptos watch
```

This starts the Axum server with Leptos hot reload at `http://127.0.0.1:3000`. Changes to Rust source files trigger recompilation of both the server and the WASM client.

## Running checks

```bash
# Full CI check (build + clippy + fmt + tests)
nix flake check

# Individual commands
cargo fmt --all
cargo clippy --all-targets -- --deny warnings
cargo test --workspace --exclude plinth-client
```

The client crate is excluded from `cargo test` because it targets `wasm32-unknown-unknown`.

## Building documentation locally

```bash
cd docs
zola serve
```

This starts a local preview server with live reload. The AdiDoks theme is automatically linked in the dev shell.

## Important build notes

- **New files must be `git add`-ed** before `nix flake check` or `nix build` can see them (Nix uses the git index)
- **`reqwest::Client::new()` panics in Nix sandbox** — use `Client::builder().build()` and handle errors
- **`fastembed::TextEmbedding::try_new()`** downloads models at runtime and fails in the Nix sandbox — all tests must avoid it
- **Raw string literals**: use `r##"..."##` when content contains `"#` (common with Markdown headings in SurrealQL)
