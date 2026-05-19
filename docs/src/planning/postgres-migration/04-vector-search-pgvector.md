# Phase 04 — Move vector similarity to pgvector

> **Recommended Codex model: GPT 5.5 medium**
>
> Small scope (one actor file plus its callers) but the semantic change is meaningful: cosine similarity moves from in-Rust iteration over `Vec<f32>` to a `vector_cosine_ops` index lookup. The change is mostly subtractive (delete the Rust loop, replace with a SQL query) but the agent must verify dimension consistency and HNSW query tuning. Low tier risks shipping a `<->` query without `LIMIT` or without the index hint; medium tier handles this cleanly. Not a candidate for high — there's no architectural choice once the pgvector decision is made.

## Working tree

`/data/nvme0/can/Projects/solo/plinth` — same repo. Depends on Phase 01 (PgPool) and Phase 02 (vector column + HNSW index). Can run in parallel with Phase 03 — disjoint file set.

## Goal

`crates/server/src/actors/vector_search.rs` no longer computes cosine similarity in Rust. Instead, similarity queries are issued as SQL using pgvector's `<=>` operator (cosine distance) against the HNSW index created in Phase 02. The `fastembed` embedding generation path is unchanged; only the storage and retrieval of the resulting vector changes. The actor's public API (search-by-query-string returning ranked posts) is preserved bit-for-bit.

## Why this matters now

Per the audit (chat 2026-05-19), Plinth currently stores embeddings as `array<float>` in SurrealDB, loads all of them into the actor's memory, and computes cosine similarity in Rust. That's O(n) per query and O(n) memory in the actor. pgvector's HNSW index gives O(log n) lookup and keeps the vectors in Postgres where the rest of the data lives — a strict simplification.

This is also the phase where the embedding-dimension constant becomes load-bearing: Phase 02 hardcoded `vector(384)` to match `AllMiniLML6V2`. If the model changes later, this is the single place to update.

## Out of scope

- Changing the embedding model (still `fastembed::AllMiniLML6V2`).
- Changing what triggers a re-embedding (still blog-post create/update).
- Tuning HNSW parameters beyond defaults (`m=16, ef_construction=64`). Default is fine for Plinth's scale (low thousands of posts).
- Hybrid keyword+vector search — out of scope unless the existing code already does it, in which case preserve verbatim.

## Plan

1. **Embedding storage write path.** Wherever Phase 03 left a `todo!()` or stub for "store embedding", replace with:
   ```rust
   use pgvector::Vector;
   let embedding = Vector::from(vec_f32);
   sqlx::query("UPDATE blog_posts SET embedding = $1 WHERE id = $2")
       .bind(embedding)
       .bind(post_id)
       .execute(&pool).await?;
   ```
2. **Similarity query.** Replace the in-Rust cosine loop with:
   ```sql
   SELECT id, slug, title, 1 - (embedding <=> $1) AS similarity
   FROM blog_posts
   WHERE embedding IS NOT NULL
   ORDER BY embedding <=> $1
   LIMIT $2;
   ```
   `<=>` is cosine distance; `1 - distance` gives the similarity score the existing API returns.
3. **Drop the in-memory cache.** The vector_search actor likely keeps a `Vec<(PostId, Vec<f32>)>` for fast iteration. Delete it. The HNSW index in Postgres replaces it. If the actor still needs to hold a `PgPool`, fine; if it now has no state, consider making it a free function — but only if that doesn't ripple into Phase 03's actor wiring. Default: keep the actor struct, gut its body.
4. **Embedding-dimension constant.** Add `pub const EMBEDDING_DIM: usize = 384;` near the `fastembed` setup and reference it in `assert!(vec.len() == EMBEDDING_DIM)` before insertion. Failure here means model and schema have drifted.
5. **Verify HNSW is used.** `EXPLAIN ANALYZE` the similarity query against a populated test DB; confirm `Index Scan using blog_posts_embedding_hnsw_idx`. Note the result in the PR description; this is a one-time check, not a regression test.

## Acceptance criteria

- [ ] `rg -n 'cosine|dot_product|f32.*sum' crates/server/src/actors/vector_search.rs` returns zero hits (no in-Rust math).
- [ ] `cargo build -p plinth-server` succeeds.
- [ ] Manual smoke: insert ≥3 blog posts via the admin API, issue a search query, confirm results are ranked by semantic similarity (top hit is the most relevant post by eyeball).
- [ ] `EXPLAIN ANALYZE` of the similarity query (pasted into PR description) shows index usage, not Seq Scan.
- [ ] `EMBEDDING_DIM` constant is referenced in both the embedding-generation site and the schema documentation comment in `0003_blog.sql`.

## Files likely touched

- `crates/server/src/actors/vector_search.rs` (gut and rewrite)
- Callers of `vector_search`: probably `crates/server/src/lib.rs` and one or two HTTP handlers — only if their type signatures shift.
- `crates/server/migrations/0003_blog.sql` — add a `-- EMBEDDING_DIM = 384` comment for traceability.

## Pitfalls

- **`<->` vs `<=>` vs `<#>`.** Three operators: L2 distance (`<->`), cosine distance (`<=>`), inner product (`<#>`). The existing code uses cosine similarity, so `<=>` is the right one. Easy to grab the wrong one and have results that look "kinda right but not great".
- **HNSW index is approximate.** It will not always return the exact top-k. For Plinth's use case this is fine; mention it in the API docs (Phase 06).
- **`ORDER BY embedding <=> $1` uses the index** only if there's a `LIMIT`. Without `LIMIT`, Postgres falls back to a sequential scan. Always include `LIMIT`.
- **NULL embeddings.** New posts without embeddings yet must be filtered with `WHERE embedding IS NOT NULL` — otherwise pgvector errors on the operator.
- **`pgvector::Vector` ownership.** Construct from `Vec<f32>` with `Vector::from`; don't try to borrow a slice across an await point.

## Reference

- Audit transcript: chat session 2026-05-19, vector search section.
- Existing implementation being replaced: `crates/server/src/actors/vector_search.rs:56-70` (in-Rust cosine).
- pgvector operator reference: <https://github.com/pgvector/pgvector#querying>.
- Prev: [03-query-rewrite.md](./03-query-rewrite.md). Next: [05-nix-and-deploy.md](./05-nix-and-deploy.md). Parallel with Phase 03.
