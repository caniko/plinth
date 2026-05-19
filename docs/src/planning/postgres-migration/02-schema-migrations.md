# Phase 02 — Translate schema to Postgres with junction tables

> **Recommended Codex model: GPT 5.5 medium**
>
> Schema translation with one non-trivial design call: converting two SurrealDB `RELATE` edges (`tagged`, `todo_tagged`) into proper junction tables with composite primary keys and `ON DELETE CASCADE`. Mostly mechanical, but the FK direction and indexing choices have downstream query-performance consequences. Low tier risks producing a schema that "works" but lacks the indexes needed for tag-listing queries; max tier is unnecessary for a ~7-table schema with no complex constraints.

## Working tree

`/data/nvme0/can/Projects/solo/plinth` — same repo. Depends on Phase 01 landing (needs `sqlx::migrate!` wiring and `PgPool`).

## Goal

`sqlx migrate run` (or the per-brick migration runner) applied against an empty Postgres 16 database produces a schema equivalent to the SurrealDB one, with:

- All 7 content tables (`site_content`, `tags`, `blog_posts`, `portfolio_items`, `todos`, plus the two junction tables `blog_post_tags`, `todo_tags`).
- The `pgvector` extension created and `blog_posts.embedding` typed as `vector(384)`.
- `schema_migrations` tracking table preserved (brick, version, name, applied_at).
- All `created_at` / `updated_at` columns defaulting to `now()`.

## Why this matters now

Phase 01 left every query as `todo!()`. Phase 03 will rewrite them, but it needs a target schema to query against. Landing the schema as its own commit makes it independently reviewable and gives Phase 03 a known-good DB to develop against (`sqlx migrate run && cargo test -- --ignored db_smoke`).

The audit identified the exact tables and columns; this phase is the translation:

| SurrealDB | Postgres |
|---|---|
| `DEFINE TABLE … SCHEMAFULL` | `CREATE TABLE …` |
| `tagged` edge (FROM blog_posts TO tags) | `blog_post_tags (post_id, tag_id, created_at) PK (post_id, tag_id)` |
| `todo_tagged` edge | `todo_tags (todo_id, tag_id, created_at)` |
| `array<float>` for embedding | `vector(384)` |
| `record<table>` links | `BIGINT REFERENCES table(id) ON DELETE CASCADE` (or `uuid` if you prefer) |

## Out of scope

- Rewriting any query in `db.rs` or `bricks/*/cache.rs` (Phase 03).
- Backfilling data from an existing SurrealDB deployment — this is a greenfield migration; data migration is a separate concern the user has not asked for.
- Adding new columns or indexes beyond what the SurrealDB schema had, except for the FK indexes called out below.
- Setting up Postgres locally or in Nix (Phase 05).

## Plan

1. **Pick an ID strategy.** SurrealDB used opaque record IDs; pick `BIGSERIAL` for all tables for simplicity. If you prefer `uuid v7`, add the `uuid-ossp` extension; document the choice in the migration header comment.
2. **Migration layout.** Use `sqlx::migrate!` pointing at `crates/server/migrations/`. One file per logical step:
   - `0001_init.sql` — extensions (`CREATE EXTENSION IF NOT EXISTS vector;`), `schema_migrations` table.
   - `0002_core.sql` — `site_content`, `tags`.
   - `0003_blog.sql` — `blog_posts` + `blog_post_tags` junction.
   - `0004_portfolio.sql` — `portfolio_items`.
   - `0005_todo.sql` — `todos` + `todo_tags` junction.
3. **Junction tables.** For each:
   ```sql
   CREATE TABLE blog_post_tags (
     post_id BIGINT NOT NULL REFERENCES blog_posts(id) ON DELETE CASCADE,
     tag_id  BIGINT NOT NULL REFERENCES tags(id)       ON DELETE CASCADE,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     PRIMARY KEY (post_id, tag_id)
   );
   CREATE INDEX blog_post_tags_tag_id_idx ON blog_post_tags (tag_id);
   ```
   The reverse-direction index is what makes "list posts with tag X" queries fast — don't skip it.
4. **Vector column.** `embedding vector(384)` on `blog_posts`. Add an HNSW index *after* the table is populated in production, but include the migration now:
   ```sql
   CREATE INDEX blog_posts_embedding_hnsw_idx
     ON blog_posts USING hnsw (embedding vector_cosine_ops);
   ```
5. **Slug uniqueness.** SurrealDB had `slug` as part of the record ID; Postgres needs `UNIQUE` constraints on every `slug` column.
6. **Replace `services/migrations.rs`.** Keep the per-brick versioning API but back it with `sqlx::migrate!` per migration dir. The user-facing API (`run_migrations(&pool, brick)`) stays; the implementation changes.
7. **Smoke test.** Add `tests/migration_integration.rs` (already exists per git status) update to: spin up a fresh Postgres via `sqlx::PgPool`, run migrations, assert all 7 tables exist via `information_schema.tables`.

## Acceptance criteria

- [ ] `sqlx migrate run --database-url $TEST_DB_URL` against an empty DB exits 0.
- [ ] `psql $TEST_DB_URL -c "\dt"` lists exactly: `site_content`, `tags`, `blog_posts`, `blog_post_tags`, `portfolio_items`, `todos`, `todo_tags`, `_sqlx_migrations` (or your `schema_migrations` if you keep the brick-aware one).
- [ ] `psql -c "\d blog_posts"` shows `embedding | vector(384)`.
- [ ] `psql -c "\di"` shows `blog_post_tags_tag_id_idx`, `todo_tags_tag_id_idx`, and the HNSW index on `blog_posts.embedding`.
- [ ] `cargo test -p plinth-server --test migration_integration` passes.
- [ ] Running migrations twice is a no-op (idempotency).

## Files likely touched

- `crates/server/migrations/0001_init.sql` (new)
- `crates/server/migrations/0002_core.sql` (new)
- `crates/server/migrations/0003_blog.sql` (new)
- `crates/server/migrations/0004_portfolio.sql` (new)
- `crates/server/migrations/0005_todo.sql` (new)
- `crates/server/src/services/migrations.rs` (rewrite)
- `crates/server/src/services/db.rs` (call `sqlx::migrate!` after pool init)
- `crates/server/tests/migration_integration.rs`
- Delete: SurrealQL `migrations.rs` content in each brick's `migrations.rs` (`bricks/blog/migrations.rs`, etc.) — replace with re-exports of the new `.sql` directories or fold into the central runner.

## Pitfalls

- **pgvector dimension mismatch panics at insert time, not migration time.** If you change the embedding model later (e.g. 384 → 768), `vector(384)` will silently accept the old code path and fail loudly only when an insert happens. Hardcode the dim from the `fastembed` config to catch drift at compile time — wire this in Phase 04.
- **`ON DELETE CASCADE` direction.** Cascading from `blog_posts → blog_post_tags` is what you want (delete a post, drop its tag links). Cascading from `tags → blog_post_tags` is *also* what you want (delete a tag, drop links from posts). Cascading from `blog_post_tags` outward is *not* what you want. Easy to get backwards.
- **`sqlx::migrate!` is compile-time-embedded.** Migrations baked into the binary; you cannot edit them in production. That's the intended behaviour but mention it in the deployment doc (Phase 06).
- **HNSW index on an empty table is fine** but pgvector recommends building it after bulk-loading. For greenfield this doesn't matter; flag in deployment docs.

## Reference

- Audit transcript: chat session 2026-05-19.
- Existing SurrealDB schemas:
  - `crates/server/src/services/migrations.rs:17,27`
  - `crates/server/src/bricks/blog/migrations.rs:10,33`
  - `crates/server/src/bricks/portfolio/migrations.rs:9`
  - `crates/server/src/bricks/todo/migrations.rs:9,28`
- Prev: [01-deps-and-connection.md](./01-deps-and-connection.md). Next: [03-query-rewrite.md](./03-query-rewrite.md).
