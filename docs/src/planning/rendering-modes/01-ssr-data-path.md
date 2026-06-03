# Phase 01 — Fill the SSR data path (replace every `todo!("phase 03")` server fn)

> **Recommended Codex model: GPT 5.5 high**
>
> Moderate-to-complex work in a foundational, blocking role: 12 server functions
> must be wired from the WASM-safe client crate to the already-migrated Postgres
> backend without pulling `sqlx` into the WASM build. The risk is semantic, not
> volume: nullable columns, `TEXT[]` tag arrays, deterministic ordering, and the
> `#[cfg(feature = "ssr")]` body/`unreachable!()` split are each easy to get
> subtly wrong in ways that compile cleanly and only panic or mis-render at
> runtime. A medium model tends to fumble the nullable/tag-array edge cases and
> the feature-gating. This phase blocks every other phase, so a quiet defect here
> propagates. Not `max`: there are no architectural decisions, just careful,
> pattern-following translation against an existing reference impl.

## Working tree

`/data/nvme0/can/Projects/solo/plinth` — same repo, no prerequisite phase. This is
Wave 0; everything else depends on it.

## Goal

Every `todo!("phase 03")` stub in `crates/client/src/api.rs` is replaced by a real,
`ssr`-gated implementation that reads from the existing Postgres backend, so that
`/posts`, `/posts/:slug`, `/posts/tag/:tag`, `/series`, `/series/:slug`,
`/projects`, `/projects/:slug`, `/todos`, `/todos/:slug`, `/todos/tag/:tag`, and
the home `intro` render real data under the current SSR + hydrate build — with **no
runtime panic** and **no `sqlx` in the WASM dependency tree**.

## Why this matters now

The blog/portfolio/todo/series/site-content server functions are stubs:

```
crates/client/src/api.rs:14   get_site_content      todo!("phase 03")
crates/client/src/api.rs:22   get_blog_posts        todo!("phase 03")
crates/client/src/api.rs:31   get_blog_post_by_slug todo!("phase 03")
crates/client/src/api.rs:40   get_blog_posts_by_tag todo!("phase 03")
crates/client/src/api.rs:49   get_series_nav        todo!("phase 03")
crates/client/src/api.rs:58   get_series_posts      todo!("phase 03")
crates/client/src/api.rs:64   get_all_series        todo!("phase 03")
crates/client/src/api.rs:72   get_portfolio_items   todo!("phase 03")
crates/client/src/api.rs:81   get_portfolio_item_by_slug todo!("phase 03")
crates/client/src/api.rs:287  get_todos             todo!("phase 03")
crates/client/src/api.rs:296  get_todo_by_slug      todo!("phase 03")
crates/client/src/api.rs:305  get_todos_by_tag      todo!("phase 03")
```

Each call path is `Resource::new(.., |_| api::get_blog_posts())` inside a page's
`<Suspense>` (`crates/client/src/pages/home.rs:88`, `blog_list.rs`, etc.). On the
server render these bodies execute and hit `todo!()` → the page render panics; on a
hydrate/CSR build the function instead POSTs to `/api/GetBlogPosts`, whose server
handler also panics. So today only the activity surfaces actually work. No
per-route rendering mode can be assigned to a route whose loader panics — this
phase is the precondition for the entire plan, and completing it closes the
Postgres migration plan's only outstanding gap (its server side already migrated;
this is the client-facing tail).

## Out of scope

- Choosing or changing any route's rendering mode / `SsrMode` (Phases 02–04).
- The CSR data-source split — keep these as Leptos server functions for now;
  Phase 05 introduces the REST-vs-server-fn abstraction.
- Schema changes or new migrations — the tables and columns already exist
  (`0003_blog.sql`, `0004_portfolio.sql`, `0005_todo.sql`).
- Vector search / embeddings (already shipped via the activity + search paths).
- Touching `app.rs`, `home.rs` markup, or components (other phases own those).

## Plan

1. **Inventory the data already available server-side.** The backend was migrated
   to Postgres; the query logic these stubs need almost certainly already exists in
   `crates/server/src/services/db.rs`, `crates/server/src/services/rows.rs`, and
   the brick caches (`bricks/blog`, `bricks/portfolio`, `bricks/todo`). Decide per
   function whether to (a) call an existing server service function, or (b) inline a
   `sqlx::query` mirroring the activity reference impl. Prefer (a) — reuse the
   migrated query and its row decoder — to avoid a second source of truth for the
   SQL.
2. **Follow the activity reference pattern exactly** (`crates/client/src/api.rs:86-121`):
   ```rust
   #[cfg(feature = "brick-blog")]
   #[server(GetBlogPosts, "/api")]
   pub async fn get_blog_posts() -> Result<Vec<plinth_shared::BlogListItem>, ServerFnError> {
       #[cfg(feature = "ssr")]
       {
           let db = expect_context::<sqlx::PgPool>();
           // reuse the migrated query helper or inline the SELECT
           query_blog_list(&db).await.map_err(|e| ServerFnError::new(e.to_string()))
       }
       #[cfg(not(feature = "ssr"))]
       { unreachable!("server fn body only runs under ssr") }
   }
   ```
   For functions taking params (`slug`, `tag`, `post_slug`, `series_slug`), bind the
   `_ = arg;` placeholder removal: the arg is now used. Under `not(ssr)` keep a
   `let _ = arg;` before `unreachable!()` to silence unused-variable warnings.
3. **`get_site_content(key)`** → read the `site_content` row by key (see
   `services/declarative_content.rs` / the `core_cache` for the migrated query) and
   return `Option<SiteContent>`.
4. **Blog set** (`get_blog_posts`, `get_blog_post_by_slug`, `get_blog_posts_by_tag`,
   `get_series_nav`, `get_series_posts`, `get_all_series`):
   - Reuse the blog brick's migrated queries. Tag filtering joins
     `blog_post_tags` → `tags` (the Phase-03 query-rewrite shape:
     `JOIN ... WHERE bpt.post_id = $1`); do **not** reintroduce any SurrealQL.
   - Map `Vec<String>` tag columns from `TEXT[]`, not JSON.
   - Add explicit `ORDER BY published_at DESC, id DESC` (or the brick's canonical
     order) to every list query — Postgres guarantees no order without it.
5. **Portfolio set** (`get_portfolio_items`, `get_portfolio_item_by_slug`): reuse the
   portfolio brick query; map `tech_stack` (`TEXT[]`).
6. **Todo set** (`get_todos`, `get_todo_by_slug`, `get_todos_by_tag`): reuse the todo
   brick query; same tag-join + ordering rules.
7. **Confirm the `Resource` call sites compile unchanged** — these functions keep
   their existing signatures (`pages/home.rs:88`, `blog_list.rs`, `blog_post.rs`,
   `portfolio.rs`, `todo_list.rs`, etc., already call them). No page markup edits.
8. **Build under both feature sets and check the WASM tree:**
   ```
   cargo leptos build
   cargo tree -p plinth-client --target wasm32-unknown-unknown | rg -i 'sqlx|pgvector|fastembed|plinth-forge' && echo "LEAK" || echo "clean"
   ```

## Acceptance criteria

- [ ] `rg 'todo!\("phase 03"\)' crates/` returns zero hits.
- [ ] `cargo leptos build` succeeds; `cargo clippy --workspace --all-targets -- -D warnings` exits 0.
- [ ] The WASM tree check above prints `clean` (no `sqlx`/`pgvector`/`fastembed`/`plinth-forge`).
- [ ] Smoke run against a migrated test DB: start the server, seed one blog post
      (two tags), one portfolio item, one todo via the admin API, then
      `curl -s localhost:3000/posts`, `/projects`, `/todos` and assert each returns
      HTTP 200 with the seeded title present in the SSR HTML (not "Could not load").
- [ ] `curl -s localhost:3000/posts/<slug>` round-trips the two tags in order.
- [ ] No server-function body references SurrealQL / `->tagged->` / `RELATE`
      (`rg -i 'surrealql|->tagged->|RELATE ' crates/client/ crates/server/src/` → zero).

## Files likely touched

- `crates/client/src/api.rs` (the 12 stub bodies; add `ssr`-gated query helpers or
  call into server services).
- Possibly `crates/server/src/services/db.rs` or a brick module — **only** to make
  an already-migrated query helper `pub(crate)`/reachable from the server-fn body;
  no new query logic if it already exists.

## Pitfalls

- **`expect_context::<sqlx::PgPool>()` panics if context is missing.** It is
  provided by `leptos_routes_with_context` AND the `file_and_error_handler`
  fallback in `main.rs` (both call `provide_context(db.clone())`). If you add a new
  render entrypoint, it must provide the pool too — but this phase shouldn't add
  one.
- **Forgetting the `not(ssr)` arm.** Without `#[cfg(not(feature = "ssr"))] { unreachable!() }`
  the function won't compile for the WASM target (no `sqlx` there). Copy the
  activity pattern verbatim.
- **Tag arrays.** `Vec<String>` ↔ `TEXT[]`; using `sqlx::types::Json` here will
  silently mis-decode. Use `row.try_get::<Vec<String>, _>("tags")`.
- **Nullable columns.** `Option<T>` maps to `… NULL`; use `IS NULL`, never `= NULL`.
- **Reusing vs duplicating SQL.** If the blog brick cache already holds the list in
  memory (it is a Kameo actor), prefer asking the actor or calling the shared query
  helper over writing a fresh `SELECT` — two copies of the ordering/tag logic will
  drift. Check `bricks/blog/cache.rs` first.
- **`ServerFnError` conversion.** Map `sqlx::Error` with `ServerFnError::new(e.to_string())`,
  matching the activity impl; don't add a new error type.

## Reference

- Reference implementation to mirror: `crates/client/src/api.rs:86-265`
  (`GetActivityList` / `GetActivityItemById` + `query_activity_*` helpers).
- Migrated backend queries: `crates/server/src/services/db.rs`,
  `crates/server/src/services/rows.rs`, `crates/server/src/bricks/{blog,portfolio,todo}/cache.rs`.
- The originating migration phase whose tail this closes:
  the retired Postgres migration query-rewrite phase.
- Next phases consuming this: [02-ssg-static-routes.md](./02-ssg-static-routes.md),
  [03-streaming-home.md](./03-streaming-home.md).
