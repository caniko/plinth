+++
title = "API Reference"
description = "REST API and Rust API documentation"
weight = 60
sort_by = "weight"
+++

Plinth exposes a REST API for article management, search, and image proxying. All admin endpoints require `Authorization: Bearer <PLINTH_API_KEY>`.

## REST Endpoints

- [Admin API](/docs/api/admin/) — publish articles, manage tags, update site content
- [Search API](/docs/api/search/) — semantic search, related articles, opinion tracking
- [Image Proxy](/docs/api/images/) — proxy images from Immich with caching

## Rust API (rustdoc)

Generated API documentation for each crate:

- [plinth-shared](/api/rustdoc/plinth_shared/) — domain types and shared models
- [plinth-server](/api/rustdoc/plinth_server/) — server library (actors, services, API handlers)
- [plinth-cli](/api/rustdoc/plinth_cli/) — CLI binary documentation
