# Phase 03 — Streaming SSR for the home page

> **Recommended Codex model: GPT 5.5 medium**
>
> Moderate complexity, leaf-ish role. The home page already has the right shape
> (independent `<Suspense>` boundaries per section), so the work is confirming the
> `/` route uses out-of-order streaming, ensuring the shell flushes before the
> slowest section resolves, and not regressing the existing fallbacks. The design
> space is bounded and there is a working in-repo reference (the page renders today,
> just not optimally streamed). A medium model handles this; high would be
> over-spend. The one subtlety — keeping each section's `Resource` independent so a
> slow activity refresh doesn't block the blog strip — is a localized, testable
> property, not a cross-system invariant.

## Working tree

`/data/nvme0/can/Projects/solo/plinth` — depends on Phase 01 (the home page's
`intro`, blog, and portfolio sections call server functions that must return real
data). **Touches `crates/client/src/app.rs`'s `/` route**, shared with Phase 02; if
Phase 02 landed first, rebase `app.rs` before editing.

## Goal

The home page (`/`) renders with out-of-order streaming SSR: the HTML shell, header,
footer, and `intro` paint immediately, and each content strip (Recent Posts,
Projects, Recent Activity) streams in independently as its `Resource` resolves —
the slowest section (activity, which may trigger a forge refresh) never blocks
delivery of the rest of the page.

## Why this matters now

`HomePage` (`crates/client/src/pages/home.rs`) aggregates four async sources:
`get_site_content("home-intro")`, `get_blog_posts()`, `get_portfolio_items()`,
`get_activity_list()`. Each already sits in its own `<Suspense>` with a real
fallback (`home.rs:41`, `:96`, `:166`, `:243`). Under the default render path these
can serialize or block the whole-page response until all resolve. The activity
section is the worst case: a stale cache read can fire a single-flight forge
re-fetch. Out-of-order streaming turns this latent structure into actual
time-to-first-byte wins: the shell + tagline ship first, the strips fill in. This is
the streaming-SSR mode the plan calls for, and the home page is its best (and likely
only) home — every other page is either static (Phase 02) or a single dynamic query.

## Out of scope

- Other pages — single-query pages don't benefit from streaming; leave them on
  their assigned mode.
- Static generation of `/` (it embeds live activity — never static).
- Restructuring `home.rs`'s markup beyond what streaming requires (the `<Suspense>`
  boundaries already exist; do not collapse them into one).
- Islands (Phase 04).

## Plan

1. **Set the `/` route to out-of-order streaming.** In `app_routes()`
   (`crates/client/src/app.rs`), give the home route the streaming `SsrMode`
   (out-of-order is the Leptos default, but make it explicit:
   `ssr=SsrMode::OutOfOrder`) so the mode table (Phase 02) is unambiguous. Verify
   the exact 0.8 enum variant name against the pinned `leptos_router`.
2. **Confirm the shell flushes early.** `shell()` in `crates/server/src/main.rs`
   wraps `<App/>` in `<body>`. Out-of-order streaming requires the response to begin
   before suspended resources resolve. Verify `leptos_routes_with_context` uses the
   streaming renderer for this route (Leptos picks the renderer from `SsrMode`);
   confirm no middleware buffers the full body (e.g. a `compression`/response layer
   that waits for `content-length`).
3. **Keep each section's `Resource` independent.** Audit `home.rs` so the four
   `Resource::new` calls do not depend on one another's output (they currently
   don't). Each `<Suspense>` should resolve on its own; the activity section in
   particular must not be awaited before the blog section streams.
4. **Preserve the existing fallbacks** (`tagline` for intro, "Loading…" strips).
   Streaming sends the fallback first, then the resolved content; the current
   `EitherOf3` match arms already cover `Ok(empty)`, `Ok(data)`, `Err`.
5. **Decide in-order vs out-of-order for SEO-critical content.** Out-of-order is
   best for TTFB; if the intro block is SEO-critical and must appear in source order
   for crawlers, consider `<Suspense>` vs `<Transition>` or an in-order boundary for
   just the intro. Document the choice in the mode table.

## Acceptance criteria

- [ ] The `/` route declares an explicit streaming `SsrMode` matching the Phase-02
      mode table.
- [ ] A test asserts incremental delivery: requesting `/` yields the shell + `intro`
      bytes **before** the activity strip's bytes, and an artificially slow activity
      loader (injected delay) does not delay the blog/portfolio strips' bytes.
      (Assert on streamed chunk order / first-byte timing, not just final HTML.)
- [ ] With all sections fast, `/` final HTML is unchanged from today (no visual
      regression) — same sections, same fallbacks.
- [ ] `cargo leptos build` + `cargo clippy --workspace -- -D warnings` clean.
- [ ] Hydration still works: after load, client-side navigation from `/` to
      `/posts` works (router intact).

## Files likely touched

- `crates/client/src/app.rs` (the `/` route's `ssr=` attribute + mode table entry).
- `crates/client/src/pages/home.rs` (only if a `<Suspense>`/`<Transition>` boundary
  needs adjusting for streaming; markup otherwise unchanged).
- `crates/server/src/main.rs` (only to confirm/adjust the streaming renderer and
  that no layer buffers the response).

## Pitfalls

- **A buffering middleware defeats streaming.** A response-compression or
  body-collecting layer that computes `content-length` forces the whole body before
  the first byte ships. If TTFB doesn't improve, suspect the middleware stack
  (`crates/server/src/main.rs` layers) before the Leptos config.
- **Coupling the resources.** If a refactor makes one section await another's data,
  streaming collapses back to all-or-nothing. Keep them independent.
- **Wrong `SsrMode` variant name.** 0.8 enum spelling differs from older guides;
  verify against rustdoc.
- **`<Suspense>` vs `<Transition>`.** `<Transition>` keeps the old view during
  refetch (good for client nav) but streams differently on first paint; pick
  deliberately and note why.
- **Activity refresh on the home strip.** `get_activity_list()` can trigger the
  single-flight refresh actor; that's fine for streaming (it's the slow section by
  design) but make sure the test's injected delay simulates it rather than hanging.

## Reference

- Home page structure: `crates/client/src/pages/home.rs` (four independent
  `<Suspense>` sections).
- SSR/streaming wiring: `crates/server/src/main.rs` (`leptos_routes_with_context`,
  `shell`, middleware layers).
- Prereq: [01-ssr-data-path.md](./01-ssr-data-path.md). Shares `app.rs` with
  [02-ssg-static-routes.md](./02-ssg-static-routes.md).
