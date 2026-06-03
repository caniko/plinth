# Summary

[Introduction](./introduction.md)

# Getting Started

- [Installation](./getting-started/installation.md)
- [Quick Start](./getting-started/quick-start.md)

# Architecture

- [Overview](./architecture/overview.md)
- [Actor System](./architecture/actor-system.md)

# Configuration

- [plinth.toml](./configuration/plinth-toml.md)
- [Environment Variables](./configuration/environment-vars.md)

# Deployment

- [NixOS Module](./deployment/nixos-module.md)
- [Reverse Proxy](./deployment/reverse-proxy.md)

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

# Plan: Postgres migration

- [01 — Deps and connection](./planning/postgres-migration/01-deps-and-connection.md)
- [02 — Schema and migrations](./planning/postgres-migration/02-schema-migrations.md)
- [03 — Query rewrite](./planning/postgres-migration/03-query-rewrite.md)
- [04 — Vector search via pgvector](./planning/postgres-migration/04-vector-search-pgvector.md)
- [05 — Nix and deploy](./planning/postgres-migration/05-nix-and-deploy.md)
- [06 — Tests and docs](./planning/postgres-migration/06-tests-and-docs.md)

# Plan: Forge activity

- [Overview](./planning/forge-activity/README.md)
- [01 — Shared types and migration](./planning/forge-activity/01-shared-types-and-migration.md)
- [02 — plinth-forge crate](./planning/forge-activity/02-forge-crate.md)
- [03 — Server brick core](./planning/forge-activity/03-server-brick-core.md)
- [04 — Lazy refresh actor](./planning/forge-activity/04-lazy-refresh-actor.md)
- [05 — CLI commands](./planning/forge-activity/05-cli-commands.md)
- [06 — Frontend surfaces](./planning/forge-activity/06-frontend-surfaces.md)
- [07 — Feed and search](./planning/forge-activity/07-feed-and-search.md)
- [08 — Nix, tests, docs](./planning/forge-activity/08-nix-tests-docs/README.md)
  - [Sub-01 — Nix packaging](./planning/forge-activity/08-nix-tests-docs/sub-01-nix-packaging.md)
  - [Sub-02 — End-to-end tests](./planning/forge-activity/08-nix-tests-docs/sub-02-e2e-tests.md)
  - [Sub-03 — Docs](./planning/forge-activity/08-nix-tests-docs/sub-03-docs.md)

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
