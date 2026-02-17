+++
title = "Testing"
description = "Test organisation and how to run tests"
weight = 20
+++

Plinth has 68 tests across the workspace.

## Running tests

```bash
# All tests (excludes client crate — targets WASM)
cargo test --workspace --exclude plinth-client

# Single test
cargo test --package plinth-server test_name

# With output
cargo test --workspace --exclude plinth-client -- --nocapture
```

## Test organisation

### Unit tests

Unit tests live in `#[cfg(test)]` modules within source files. Examples:

- `crates/server/src/config.rs` — config parsing and defaults
- `crates/server/src/api/admin.rs` — request construction and error responses
- `crates/server/src/api/images.rs` — Immich URL building and query defaults
- `crates/server/src/services/markdown_processor.rs` — Markdown parsing, slug generation
- `crates/server/src/services/db.rs` — database operations
- `crates/shared/src/blog_post.rs` — reading time calculation

### Integration tests

Located in `crates/server/tests/`:

- **`db_integration.rs`** — SurrealDB operations using `surrealdb::engine::local::Mem` (in-memory, no disk). Tests CRUD operations, schema creation, search queries, and tag graph relations.

- **`content_cache_integration.rs`** — ContentCache actor with an in-memory SurrealDB backend. Tests cache population, invalidation, post retrieval, and tag filtering.

Both use in-memory SurrealDB to avoid filesystem dependencies.

## Constraints

- **No network in Nix sandbox**: tests cannot download fastembed models or make HTTP requests
- **Client crate excluded**: `plinth-client` targets `wasm32-unknown-unknown` and cannot run under `cargo test`
- **SurrealDB datetime**: `db.create("table").content(struct)` fails for `datetime` fields — tests use raw SQL with `time::now()`
