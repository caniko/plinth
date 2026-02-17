+++
title = "Plinth"
sort_by = "weight"
+++

# Plinth Documentation

Plinth is a full-stack personal website platform built with [Leptos](https://leptos.dev) 0.8, featuring server-side rendering, WASM hydration, semantic search, and Typst blog support.

## Quick links

- [Installation](/docs/getting-started/installation/) — get Plinth running locally
- [Configuration](/docs/configuration/plinth-toml/) — customise your site via `plinth.toml`
- [Publishing](/docs/guides/publishing/) — write and publish blog posts
- [Deployment](/docs/deployment/nixos-module/) — deploy on NixOS
- [API Reference](/docs/api/) — REST endpoints and rustdoc

## Features

- **SSR + WASM hydration** — fast initial load with interactive client
- **SurrealDB** — schema-full graph database with RELATE-based tagging
- **Semantic search** — fastembed vector embeddings with cosine similarity
- **Typst support** — author blog posts in Typst with image management
- **Immich integration** — self-hosted image proxy with aggressive caching
- **NixOS module** — declarative deployment with systemd hardening
- **OTLP observability** — built-in OpenTelemetry tracing export
