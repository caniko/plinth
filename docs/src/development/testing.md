# Testing

Plinth uses unit tests for pure logic and Postgres-backed integration tests for database behaviour.

## Running tests

```bash
# All tests and integration tests
cargo test --workspace --all-targets

# Single test
cargo test --package plinth-server test_name

# With output
cargo test --workspace --all-targets -- --nocapture
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

Located in `crates/server/tests/`. Database-backed integration tests use `#[sqlx::test]`, which creates an isolated database for each test, runs `crates/server/migrations/`, and passes the test a `PgPool`.

- **`migration_integration.rs`** — verifies migrations create the expected tables, pgvector extension, vector column, and HNSW index.
- **`db_integration.rs`** — CRUD for core `site_content` and `tags`.
- **`blog_admin_integration.rs`** — blog post lifecycle, tag junction rows, denormalized tag cache, ordering, slug uniqueness, and delete cascades.
- **`tag_integration.rs`** — shared tag creation, post/todo tag attachment, counts, and tag-filtered list reads.
- **`todo_integration.rs`** — todo lifecycle, completion, ordering, tagging, slug uniqueness, and delete cascades.

## Local Postgres setup

Inside `nix develop`, start the local Postgres cluster:

```bash
./scripts/dev-db.sh start
```

The dev shell exports `DATABASE_URL=postgres://localhost/plinth?host=$PWD/.dev-pgsocket`. `#[sqlx::test]` uses that as the base URL for per-test databases. Outside the dev shell, set `DATABASE_URL` to a Postgres 16 instance where the test user can create and drop databases and where pgvector is installed.

```bash
DATABASE_URL='postgres://localhost/plinth?host=/path/to/socket' \
  cargo test -p plinth-server --test blog_admin_integration
```

## Constraints

- **No network in Nix sandbox**: tests cannot download fastembed models or make HTTP requests
- **Client crate**: browser-specific code targets `wasm32-unknown-unknown`; prefer workspace checks for full coverage
- **Postgres extensions**: pgvector must be installed in the base cluster before integration tests run
