# Phase 03 — Rewrite SurrealQL queries as SQL

> **Recommended Codex model: GPT 5.5 high**
>
> This phase has the largest surface area (multiple brick caches, `db.rs`, `declarative_content.rs`) and the highest semantic risk: getting tag-set semantics, ordering, and `Option<T>` handling subtly wrong is easy and won't show up until integration tests. The graph-traversal-to-JOIN translation requires understanding what each SurrealQL query is *trying* to return, not just transliteration. Medium tier handles simple CRUD fine but tends to fumble nullable-column edge cases and the denormalised `tags array` writes; high tier is the right floor. Max is overkill — there are no architectural decisions, just careful translation.

## Working tree

`/data/nvme0/can/Projects/solo/plinth` — same repo. Depends on Phase 01 (PgPool wiring) and Phase 02 (schema). Phase 04 can run in parallel with this one; it touches `actors/vector_search.rs` and the embedding column, which this phase only reads/writes opaquely.

## Goal

Every `todo!("phase 03")` stub from Phase 01 is replaced with a `sqlx::query`-based implementation. All previously-passing query semantics are preserved: same return shapes, same ordering, same filter behaviours. The codebase compiles and `cargo clippy --workspace -- -D warnings` is clean. Integration tests are still expected to need work — that's Phase 06.

## Why this matters now

This is the meat of the migration. Phases 01–02 set the stage; this is where SurrealQL — including the `->tagged->tags` graph traversal — becomes ordinary `JOIN`s. The denormalised `tags` array column on `blog_posts` and `todos` is a write-side concern: every tag mutation must update both the junction table and the array, or be replaced by a JOIN-based read. Per the audit (chat 2026-05-19), Plinth uses `SELECT VALUE name FROM $post->tagged->tags` to populate the array — that becomes a `SELECT array_agg(t.name) FROM blog_post_tags bpt JOIN tags t ON t.id = bpt.tag_id WHERE bpt.post_id = $1`.

## Out of scope

- Vector similarity queries (Phase 04 — leave the embedding column read/write but no `<->` operator yet).
- Changing the public API of the cache actors or the `Db` service trait.
- Performance tuning beyond using the indexes Phase 02 added.
- Updating integration tests (Phase 06).
- Removing the denormalised `tags` array column from `blog_posts` / `todos` — keep it for now, populated via trigger or app-level write. Decision deferred to Phase 06 if it proves redundant.

## Plan

1. **Inventory every SurrealQL string.** `rg -n 'query\(|sql!|SELECT |CREATE |UPDATE |DELETE |RELATE ' crates/server/src/` and produce a list. Expected hot spots:
   - `crates/server/src/services/db.rs` (blog post + tag CRUD + graph traversal)
   - `crates/server/src/services/declarative_content.rs` (site_content upserts)
   - `crates/server/src/bricks/blog/cache.rs`
   - `crates/server/src/bricks/portfolio/cache.rs`
   - `crates/server/src/bricks/todo/cache.rs`
   - `crates/server/src/actors/core_cache.rs`
2. **Translate by category:**
   - **Plain SELECT/INSERT/UPDATE** → direct `sqlx::query_as!` (if you opt into compile-time checking) or `sqlx::query_as` runtime form. Stick with runtime form for now — compile-time checking requires `SQLX_OFFLINE=true` + `cargo sqlx prepare`, which is a Phase 05/06 ergonomic concern.
   - **`RELATE $post->tagged->$tag`** → `INSERT INTO blog_post_tags (post_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING`.
   - **`SELECT VALUE name FROM $post->tagged->tags`** → `SELECT t.name FROM blog_post_tags bpt JOIN tags t ON t.id = bpt.tag_id WHERE bpt.post_id = $1 ORDER BY t.name`.
   - **`UPDATE … SET tags = [...]`** (denormalised) → either (a) keep the column and update it in the same transaction as the junction-table writes, or (b) drop the column and synthesise via JOIN at read time. Pick (a) for this phase to minimise blast radius; revisit in Phase 06.
3. **Transactions.** Any operation that touches both a content table and its junction table (e.g. updating a blog post's tag set) must run inside `pool.begin()` / `tx.commit()`. Specifically:
   - Update post, delete from junction, insert new junction rows — single tx.
   - Delete post — single tx (cascades handle junctions, but read-modify-write of dependent data needs the tx).
4. **Nullable handling.** SurrealDB's NONE-vs-null trap (see auto-memory `feedback_surrealdb_none.md`) goes away, but watch for the inverse: SurrealDB tolerated implicit type coercions Postgres won't. `Option<String>` maps to `TEXT NULL` and queries must use `IS NULL` not `= NULL`.
5. **Ordering stability.** Postgres won't guarantee any ordering without `ORDER BY`. Audit every list-returning query and add explicit `ORDER BY created_at DESC, id DESC` (or whatever the SurrealDB query relied on by accident).
6. **Compile + clippy.**

## Acceptance criteria

- [ ] `rg 'todo!\("phase 03"\)' crates/server/src/` returns zero hits.
- [ ] `cargo build -p plinth-server` produces a binary.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` exits 0.
- [ ] `rg -i 'surrealql|->tagged->|->todo_tagged->|RELATE ' crates/server/src/` returns zero hits.
- [ ] At least one end-to-end smoke run: start the server against a migrated test DB, POST a blog post with two tags via the admin API, GET it back, assert the tag array round-trips. (Document the curl commands in the PR description; full test coverage is Phase 06.)
- [ ] All transactions involving multiple tables use `pool.begin()`; no raw multi-statement strings.

## Files likely touched

- `crates/server/src/services/db.rs` (major rewrite)
- `crates/server/src/services/declarative_content.rs`
- `crates/server/src/actors/core_cache.rs`
- `crates/server/src/bricks/blog/cache.rs`
- `crates/server/src/bricks/portfolio/cache.rs`
- `crates/server/src/bricks/todo/cache.rs`
- `crates/server/src/bricks/mod.rs` (only if it has shared query helpers)

## Pitfalls

- **`sqlx::query_as` requires `FromRow`.** Derive it on every content struct, or use `#[derive(sqlx::FromRow)]`. `serde::Deserialize` is not enough.
- **`Vec<String>` columns** (the denormalised `tags` array on blog_posts/todos) map to `text[]` in Postgres, not JSON. Use `sqlx::types::Json` only if you actually want JSON. For the tag array, prefer `TEXT[]` with `array_agg`.
- **Slug-based "primary keys".** SurrealDB allowed `blog_posts:my-post-slug` as a record ID; Phase 02 separated `id BIGSERIAL` from `slug TEXT UNIQUE`. Any query that previously looked up by ID-as-slug must now lookup by slug. Easy to miss — grep for `:` inside query strings as a sanity check.
- **Booleans.** SurrealDB returned `true`/`false`/`NONE`; Postgres returns `t`/`f`/NULL. `Option<bool>` semantics survive, but explicit cast to `BOOLEAN` may be needed in `WHERE` clauses if the input is text.
- **The denormalised tags column** is now a foot-gun: writes must update two places. If you forget, reads will return stale data. Consider a `BEFORE INSERT/UPDATE` trigger that rebuilds the array from the junction table — but only if you're confident; otherwise just do it in app-side transactions and trust the test suite.

## Reference

- Audit transcript: chat session 2026-05-19.
- Graph traversal sites being rewritten: `crates/server/src/services/db.rs:97-98, 163, 188, 206`.
- Auto-memory note on SurrealDB NONE/null: `~/.claude/projects/-data-nvme0-can-Projects-solo-plinth/memory/feedback_surrealdb_none.md`.
- Prev: [02-schema-migrations.md](./02-schema-migrations.md). Next: [04-vector-search-pgvector.md](./04-vector-search-pgvector.md). Can run in parallel with Phase 04.
