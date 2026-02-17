+++
title = "Installation"
description = "Install Plinth from source"
weight = 10
+++

## Prerequisites

Plinth builds with Nix flakes. You need:

- **Nix** (2.18+) with `experimental-features = nix-command flakes`
- Or: Rust nightly, `wasm32-unknown-unknown` target, cargo-leptos, tailwindcss, wasm-bindgen-cli, binaryen

The recommended approach is Nix — it handles the entire toolchain.

## Clone and enter dev shell

```bash
git clone https://codeberg.org/caniko/plinth.git
cd plinth
nix develop
```

This drops you into a shell with Rust nightly, cargo-leptos, SurrealDB, Tailwind CSS, and all native dependencies (OpenSSL, ONNX Runtime, libclang).

## Build for production

```bash
nix build .#plinth
```

The output is in `result/`:
- `result/bin/plinth-server` — server binary (wrapper that sets `LEPTOS_SITE_ROOT`)
- `result/site/` — compiled WASM, JS, CSS, and static assets
- `result/share/plinth/plinth.toml` — example configuration

### Build variants

| Command | Profile | Use case |
|---------|---------|----------|
| `nix build .#plinth` | Release (opt-level 3) | Production deployment |
| `nix build .#plinth-dev` | Debug | Fast compile, debugging |
| `nix build .#plinth-minimal` | Release (opt-level z) | Minimal binary size |

## Run locally

```bash
# Start the development server with hot reload
cargo leptos watch
```

The server starts at `http://127.0.0.1:3000` by default. SurrealDB creates a `database.db` file in the working directory.
