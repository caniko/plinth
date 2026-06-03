# Summary

[Introduction](./introduction.md)

# Getting Started

- [Installation](./getting-started/installation.md)
- [Quick Start](./getting-started/quick-start.md)

# Architecture

- [Overview](./architecture/overview.md)
- [Actor System](./architecture/actor-system.md)
- [Rendering](./architecture/rendering.md)

# Configuration

- [plinth.toml](./configuration/plinth-toml.md)
- [Environment Variables](./configuration/environment-vars.md)

# Deployment

- [NixOS Module](./deployment/nixos-module.md)
- [Reverse Proxy](./deployment/reverse-proxy.md)
- [CSR Static Build](./deployment/csr.md)

# Guides

- [Publishing Blog Posts](./guides/publishing.md)
- [Image Handling](./guides/image-handling.md)
- [Curating External Activity](./guides/activity.md)

# API Reference

- [Admin API](./api/admin.md)
- [Search API](./api/search.md)
- [Image Proxy](./api/images.md)
- [Activity API](./api/activity.md)

# Development

- [Dev Environment](./development/setup.md)
- [Testing](./development/testing.md)
- [Contributing](./development/contributing.md)

# Plan: Rendering modes

- [Overview](./planning/rendering-modes/README.md)
- [01 — SSR data path](./planning/rendering-modes/01-ssr-data-path.md)
- [02 — SSG static routes](./planning/rendering-modes/02-ssg-static-routes.md)
- [03 — Streaming home](./planning/rendering-modes/03-streaming-home.md)
- [04 — Islands / partial hydration](./planning/rendering-modes/04-islands.md)
- [05 — CSR build profile](./planning/rendering-modes/05-csr-profile.md)
- [06 — Nix, tests, docs + retire](./planning/rendering-modes/06-nix-tests-docs/README.md)
  - [Sub-01 — Nix build matrix](./planning/rendering-modes/06-nix-tests-docs/sub-01-nix.md)
  - [Sub-02 — Per-mode e2e tests](./planning/rendering-modes/06-nix-tests-docs/sub-02-tests.md)
  - [Sub-03 — Docs + retire](./planning/rendering-modes/06-nix-tests-docs/sub-03-docs-retire.md)
