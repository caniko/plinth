# Phase 06 — Integration tests + documentation finalisation

> **Recommended Codex model: GPT 5.5 medium**
>
> Verification phase. The work is concrete (update five existing integration tests, rewrite the configuration and deployment docs, update CLAUDE.md hints if present) but spans many files and requires running each test target end-to-end. Medium tier handles test-fixture work and doc updates fluently. Low tier risks missing a `#[sqlx::test]` setup or leaving a stale SurrealDB reference in docs; high is overkill — no design content.

## Working tree

`/data/nvme0/can/Projects/solo/plinth` — same repo. Depends on Phases 01–05 all landing. This is the final phase; the migration is not "done" until this is green.

## Goal

- All five integration test files (`blog_admin_integration.rs`, `db_integration.rs`, `migration_integration.rs`, `tag_integration.rs`, `todo_integration.rs`) pass against a freshly-migrated Postgres database, using `#[sqlx::test]` or an equivalent per-test isolated DB pattern.
- `cargo test --workspace --all-targets` exits 0 with no ignored tests that were previously running.
- `docs/book/src/` contains zero references to SurrealDB outside of this `planning/postgres-migration/` directory.
- The auto-memory note `feedback_surrealdb_none.md` is updated or removed (it no longer applies).
- The denormalised `tags` array column on `blog_posts` / `todos` is either justified (in a code comment) or removed in favour of JOIN-based reads — decision made and documented.

## Why this matters now

Phases 01–05 produced a working but unverified migration. This phase is where the migration is *proven* — without these tests passing, the migration is technically incomplete and regressions will leak in unnoticed.

The docs cleanup is bundled here because the previous phases deliberately deferred doc rewrites to avoid noisy diffs blocking review. Now is the time.

## Out of scope

- Adding *new* test coverage beyond what existed before the migration (those go in follow-up PRs).
- Performance benchmarking — separate concern.
- Removing the `planning/postgres-migration/` directory once done (keep as historical record; the user can decide later).

## Plan

1. **Test isolation strategy.** Adopt `#[sqlx::test]`. It spins up a per-test database, runs migrations, hands you a `PgPool`. Configure it in `crates/server/Cargo.toml`'s `[dev-dependencies]` and via `SQLX_TEST_DB_BASE_URL` in CI / dev-shell. Document this in `docs/book/src/development/testing.md`.
2. **Rewrite each integration test file** in turn:
   - `tests/migration_integration.rs` — assert all 7 tables + `vector` extension + HNSW index exist.
   - `tests/db_integration.rs` — CRUD on `site_content` and `tags`.
   - `tests/blog_admin_integration.rs` — full blog post lifecycle including tag association via the junction table, ordering, slug uniqueness.
   - `tests/tag_integration.rs` — tag creation, attaching tags to posts and todos, listing posts by tag.
   - `tests/todo_integration.rs` — todo CRUD, todo tagging, completion + ordering.
3. **Decide on the denormalised `tags` array column.** Run `cargo test` and benchmark a tag-listing query both ways (with the array vs with a JOIN). If JOIN is fast enough (it will be — the indexes are in place), drop the array column in a small follow-up migration `0006_drop_denorm_tags.sql` and rewrite the read paths to compute tags via `array_agg`. Document the decision either way in `crates/server/src/services/db.rs` near the post struct definition.
4. **Doc sweep**:
   - `docs/book/src/configuration/plinth-toml.md` — replace SurrealDB endpoint examples with `database_url`.
   - `docs/book/src/configuration/environment-vars.md` — `DATABASE_URL` documented.
   - `docs/book/src/deployment/nixos-module.md` — full rewrite reflecting Phase 05 module shape.
   - `docs/book/src/development/setup.md` — point at `scripts/dev-db.sh` and `nix develop`.
   - `docs/book/src/development/testing.md` — document `#[sqlx::test]` flow and how to run integration tests locally.
   - `docs/book/src/architecture/overview.md` — replace SurrealDB references; describe pgvector role.
   - `docs/book/src/api/search.md` — note that similarity is HNSW-approximate.
5. **CLAUDE.md / repo conventions** — if `CLAUDE.md` exists at the repo root, update DB section. (Git status shows `D CLAUDE.md` — check whether it's intentionally deleted or just staged for replacement.)
6. **Auto-memory cleanup.** Edit `~/.claude/projects/-data-nvme0-can-Projects-solo-plinth/memory/feedback_surrealdb_none.md` to mark it historical (or remove via the auto-memory system if Claude is doing the work). Remove its line from `MEMORY.md`. Add a new memory if any Postgres-specific gotcha emerged during phases 01–05.
7. **Final sweep**:
   ```bash
   rg -i 'surrealdb|surrealql|surreal::' crates/ docs/book/src/ --glob '!planning/postgres-migration/**'
   ```
   Expected: zero hits.

## Acceptance criteria

- [ ] `cargo test --workspace --all-targets` exits 0.
- [ ] `cargo test -p plinth-server --test blog_admin_integration` exits 0.
- [ ] `cargo test -p plinth-server --test db_integration` exits 0.
- [ ] `cargo test -p plinth-server --test migration_integration` exits 0.
- [ ] `cargo test -p plinth-server --test tag_integration` exits 0.
- [ ] `cargo test -p plinth-server --test todo_integration` exits 0.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` exits 0.
- [ ] `rg -i 'surrealdb|surrealql' crates/ docs/book/src/ --glob '!planning/postgres-migration/**'` returns zero hits.
- [ ] `mdbook build docs/book` exits 0 with no broken links.
- [ ] `feedback_surrealdb_none.md` is removed or marked historical; `MEMORY.md` index updated.

## Files likely touched

- `crates/server/tests/blog_admin_integration.rs`
- `crates/server/tests/db_integration.rs`
- `crates/server/tests/migration_integration.rs`
- `crates/server/tests/tag_integration.rs`
- `crates/server/tests/todo_integration.rs`
- `crates/server/Cargo.toml` (dev-deps for `sqlx::test`)
- `docs/book/src/configuration/plinth-toml.md`
- `docs/book/src/configuration/environment-vars.md`
- `docs/book/src/deployment/nixos-module.md`
- `docs/book/src/development/setup.md`
- `docs/book/src/development/testing.md`
- `docs/book/src/architecture/overview.md`
- `docs/book/src/api/search.md`
- Possibly `CLAUDE.md` if reintroduced
- Auto-memory files in `~/.claude/projects/.../memory/`
- Possibly `crates/server/migrations/0006_drop_denorm_tags.sql` (decision-dependent)

## Pitfalls

- **`#[sqlx::test]` requires a base DB URL** at runtime; without it tests fail to set up. Export `DATABASE_URL` in the dev shell shellHook (already done in Phase 05) but also document the override pattern for CI.
- **Test concurrency.** `#[sqlx::test]` creates a new DB per test in parallel. Postgres needs enough connection slots — bump `max_connections` if you hit "too many clients already". For ~50 tests, default 100 is fine.
- **Migration drift.** If Phase 02 left any IF-NOT-EXISTS guards, the test-DB migration runs may silently skip a real change. `#[sqlx::test]` against fresh DBs avoids this in tests but production-side drift is still a risk — flag in deployment doc.
- **Doc-link rot.** `mdbook build` is strict about broken links; a stale link to `docs/content/...` (the old Hugo path being deleted per git status) will fail the build. Sweep for `docs/content/` references in `.md` files.
- **`MEMORY.md` truncation.** Auto-memory index is loaded into every session and truncates after line 200. Don't bloat the index; consolidate where possible.

## Reference

- Audit transcript: chat session 2026-05-19.
- Auto-memory file to retire: `~/.claude/projects/-data-nvme0-can-Projects-solo-plinth/memory/feedback_surrealdb_none.md`.
- Prev: [05-nix-and-deploy.md](./05-nix-and-deploy.md). This is the final phase.
