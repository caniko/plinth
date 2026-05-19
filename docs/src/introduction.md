# Plinth

A self-hosted personal website platform built with [Leptos](https://leptos.dev) 0.8 — server-side rendering, WASM hydration, semantic search, and Typst blog support.

## Features

- **SSR + WASM hydration** — Fast initial page loads with server-side rendering, then seamless interactivity via WASM hydration powered by Leptos 0.8.
- **Postgres + pgvector** — Schema-backed relational storage with junction-table tagging and approximate vector search.
- **Semantic search** — Built-in vector embeddings via fastembed with cosine similarity search — find content by meaning, not just keywords.
- **Typst & Markdown** — Author blog posts in [Typst](https://typst.app) or Markdown. The CLI handles frontmatter, image uploads, embeddings, and publishing.
- **Immich integration** — Self-hosted image proxy backed by Immich. Upload images from the CLI, serve them through a caching proxy with aggressive cache headers.
- **Plausible analytics** — Optional self-hosted Plausible integration — privacy-friendly analytics with a single config toggle. No tracking when unconfigured.
- **NixOS deployment** — Declarative NixOS module with systemd hardening, reverse proxy configuration, and reproducible builds via Nix flakes.

## Source code

[codeberg.org/caniko/plinth](https://codeberg.org/caniko/plinth)
