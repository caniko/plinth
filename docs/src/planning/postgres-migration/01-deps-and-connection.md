# Phase 01 — Swap SurrealDB dependency for sqlx + pgvector

> **Recommended Codex model: GPT 5.5 medium**
>
> Mechanical dependency swap and connection-layer rewrite. The scope is small (one Cargo.toml, one connection module, config plumbing) but it's the foundation every later phase builds on, so the model needs enough judgement to pick sane defaults for pool size, TLS, and migration runner wiring. Low tier would underweight the design choices around `sqlx::PgPool` lifetime in the actor system; max tier is overkill for what is ultimately a wiring task.

## Working tree

`/data/nvme0/can/Projects/solo/plinth` — same repo as parent project. This phase must land before any other phase in the set.

## Goal

`cargo check -p plinth-server` compiles against `sqlx` + `pgvector` with the SurrealDB crate fully removed from `crates/server/Cargo.toml`. A new `PgPool`-based connection module replaces `crates/server/src/services/db.rs::connect()`, reads its DSN from `plinth.toml` / env, and is wired into the actor system in `lib.rs` in place of the `Surreal<Any>` handle. Existing query callsites are left as `todo!()` stubs to be rewritten in Phase 03.

## Why this matters now

The audit (chat transcript, 2026-05-19) established that Plinth uses ~5% of SurrealDB's distinctive surface and that the embedded RocksDB mode is the only thing being given up. Every subsequent phase — schema, query rewrite, vector search, Nix, tests — needs a working `PgPool` to be wired through. Doing this first means the compiler enforces phase boundaries: phases 02–06 can land incrementally without the codebase ever fully de-stubbing until phase 06 verification.

## Out of scope

- Rewriting any SurrealQL queries to SQL (Phase 03).
- Translating schema definitions (Phase 02).
- Touching `vector_search.rs` beyond stubbing it (Phase 04).
- Updating the NixOS module or flake (Phase 05).
- Making integration tests pass (Phase 06).
- Removing the `surrealdb` references from docs/book — defer until phase 06.

## Plan

1. **Cargo.toml** — in `crates/server/Cargo.toml`: remove `surrealdb`, add:
   - `sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "macros", "chrono", "uuid", "migrate"] }`
   - `pgvector = { version = "0.4", features = ["sqlx"] }`
2. **Config schema** — in `crates/shared/src/toml_config.rs`, replace the SurrealDB endpoint/namespace/database fields (currently defaulting to `rocksdb://database.db`, see line 195) with a single `database_url: String` defaulting to `postgres://plinth:plinth@localhost:5432/plinth`. Keep the field name backwards-incompatible — don't add migration shims.
3. **Connection module** — rewrite `crates/server/src/services/db.rs::connect()` to build a `PgPool` via `PgPoolOptions`. Set `max_connections = 16`, `acquire_timeout = 10s`. Register pgvector with `sqlx::postgres::PgPoolOptions::after_connect` calling `SET search_path` and a no-op `SELECT 1` to verify the extension loads. Export `pub type Db = PgPool`.
4. **Actor wiring** — in `crates/server/src/lib.rs` and every actor (`actors/core_cache.rs`, `actors/vector_search.rs`, `bricks/*/cache.rs`), change the held handle type from `Surreal<Any>` to `PgPool`. Leave query bodies as `todo!("phase 03")` — the goal is type-level correctness only.
5. **CLI check-config** — `crates/cli/src/commands/check_config.rs` likely prints the DB endpoint; update the format string to the new field.
6. **Compile** — `cargo check --workspace` must pass. Tests will not pass; that's expected (Phase 06).

## Acceptance criteria

- [ ] `rg -i surrealdb crates/` returns zero hits outside of `// removed:` comments (and there should be no such comments — fully delete).
- [ ] `cargo check --workspace` exits 0 with no warnings about unused `sqlx` or `pgvector` imports in `db.rs`.
- [ ] `cargo build -p plinth-server` produces a binary (it will panic on `todo!()` at runtime — that's fine).
- [ ] `crates/server/src/services/db.rs` exports `pub type Db = sqlx::PgPool`.
- [ ] `plinth.toml` example in `docs/book/src/configuration/plinth-toml.md` shows the new `database_url` field (one-line change; doc rewrite is Phase 06).
- [ ] Workspace `Cargo.lock` contains `sqlx`, `pgvector`, no `surrealdb`.

## Files likely touched

- `crates/server/Cargo.toml`
- `crates/server/src/services/db.rs`
- `crates/server/src/lib.rs`
- `crates/server/src/actors/core_cache.rs`
- `crates/server/src/actors/vector_search.rs`
- `crates/server/src/bricks/blog/cache.rs`
- `crates/server/src/bricks/portfolio/cache.rs`
- `crates/server/src/bricks/todo/cache.rs`
- `crates/server/src/bricks/mod.rs`
- `crates/server/src/services/declarative_content.rs`
- `crates/server/src/services/migrations.rs` (stub only — full rewrite in Phase 02)
- `crates/shared/src/toml_config.rs`
- `crates/cli/src/commands/check_config.rs`
- `crates/client/src/api.rs` (only if it references DB types directly)

## Pitfalls

- **`sqlx` macro vs runtime queries.** Do not use the `query!` macro yet — it requires a live DB at compile time. Use `sqlx::query` (runtime-checked) for all callsites. This is also Phase 03's concern but it affects what you stub.
- **`PgPool` is `Clone` and cheap to clone** — don't wrap it in `Arc<Mutex<…>>` out of habit. The actor system can hold a plain `PgPool` field.
- **pgvector feature flag.** The `pgvector` crate requires the `sqlx` feature; without it, `Vector` won't implement `Encode`/`Decode`.
- **`async_trait` + `Surreal<Any>` generic bounds.** SurrealDB's `Any` engine had quirky trait bounds; `PgPool` is concrete, which may let you remove some generic params. Resist the urge to refactor — leave generics as-is, just retype the concrete handle. Cleanup is Phase 06.

## Reference

- Audit transcript: chat session 2026-05-19, "scan and report".
- SurrealDB connection setup being replaced: `crates/server/src/services/db.rs:8-24`.
- Existing config defaults: `crates/shared/src/toml_config.rs:195`.
- Next phase: [02-schema-migrations.md](./02-schema-migrations.md).
