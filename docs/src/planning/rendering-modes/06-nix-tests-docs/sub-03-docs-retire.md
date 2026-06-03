# Phase 06 · Sub-03 — Rendering docs + retire the superseded plans

> **Recommended Codex model: GPT 5.5 low**
>
> Mechanical, low-risk leaf work: write one stable docs page documenting the
> per-route mode taxonomy (the decisions already made in Phases 02–04), wire it into
> `SUMMARY.md`, and perform the bookkeeping retirement of two plan directories whose
> work is now complete. No design decisions; the content is a transcription of
> landed behavior. Low tier is correct — but the retirement deletions must be done
> carefully (verify durable knowledge is preserved before `git rm`).

## Working tree

`/data/nvme0/can/Projects/solo/plinth` — after sub-01 (nix) and sub-02 (tests) are
green. Adds `docs/src/architecture/rendering.md`, edits `docs/src/SUMMARY.md`, and
removes the `postgres-migration` + `forge-activity` planning directories.

## Goal

The rendering architecture is documented in stable docs as a single source of truth,
and the two prior plans this set completes (`postgres-migration`, `forge-activity`)
are retired from the published planning tree with their durable knowledge confirmed
preserved.

## Why this matters now

The per-route rendering taxonomy now lives only in `app.rs` attributes and these
planning files — planning files that this set will itself retire. A future
contributor needs a stable, present-tense page explaining which routes use which
mode and why. And the two prior plans are now fully shipped (their only gap, the
client server-fn data path, was Phase 01), so leaving them published misrepresents
them as active work.

## Out of scope

- Retiring this (`rendering-modes`) plan — that is the job of this plan's own
  `verify` pass, run after this phase's merge is green. Do not delete
  `docs/src/planning/rendering-modes/` here.
- Re-deriving rendering decisions — document what landed.
- Deleting any plan whose acceptance criteria did **not** all pass (if `verify` of a
  prior plan is not clean, keep it published and report).

## Plan

1. **Write `docs/src/architecture/rendering.md`.** Present-tense, product-focused.
   Contents:
   - The per-route mode table (matches `app.rs` `ssr=` attributes / the mode source
     of truth): SSG routes, streaming home, dynamic SSR routes, islands hydration
     boundary, and the CSR build target.
   - The decision rule ("publish-cadence content → SSG; multi-source aggregate →
     streaming; user/ranked dynamic → SSR; interactive widgets → islands; serverless
     deploy → CSR build").
   - How static-route regeneration is triggered (admin publish → invalidate).
   - How to build each target (`cargo leptos build` for SSR/islands;
     `nix build .#plinth-csr` for CSR) and when to use CSR.
   - The WASM-safety invariant (server-only deps never enter the client tree).
2. **Wire it into `SUMMARY.md`** under the Architecture section (sibling to
   `architecture/overview.md`, `architecture/actor-system.md`).
3. **Confirm prior-plan durable knowledge is already in stable docs:**
   - `forge-activity`: `docs/src/api/activity.md`, `docs/src/guides/activity.md`,
     and the `[ranking]`/`[forge]`/`[feeds]` config docs already exist (shipped by
     forge-activity Phase 08/sub-03). Verify, don't rewrite.
   - `postgres-migration`: the durable outcome (Postgres + pgvector + sqlx stack,
     tag-junction model, ordering rules) is embodied in the code and the brick docs.
     Fold any genuinely-durable maintainer note that exists *only* in the
     postgres-migration phase files (e.g. the tag-array-vs-junction write rule, the
     `IS NULL` Postgres gotcha) into `docs/src/development/` or the new rendering /
     architecture page if not already covered. Do **not** migrate execution
     scaffolding (phase sequencing, model routing, branch names).
4. **Retire the prior plans:**
   - Remove their sections from `docs/src/SUMMARY.md` ("Plan: Postgres migration" and
     "Plan: Forge activity" blocks, lines ~44–66 of the current file).
   - `git rm -r docs/src/planning/postgres-migration docs/src/planning/forge-activity`
     (check `git ls-files` first; if any are untracked, `rm -rf` the remainder — see
     the retire-docs-planning lesson on tracking state).
5. **Stale-reference sweep.** `rg -n 'postgres-migration|forge-activity|Plan: Postgres|Plan: Forge'`
   across `docs/src` (source, and `docs/book` if generated output is committed) and
   resolve every hit. Rebuild mdBook if `docs/book` is committed, then re-sweep.

## Acceptance criteria

- [ ] `docs/src/architecture/rendering.md` exists, documents the per-route mode table
      matching `app.rs`, and is linked from `SUMMARY.md`.
- [ ] `docs/src/planning/postgres-migration/` and `docs/src/planning/forge-activity/`
      no longer exist and are gone from `SUMMARY.md`.
- [ ] `rg 'postgres-migration|forge-activity' docs/src/SUMMARY.md` returns zero hits.
- [ ] `rg -n 'Plan: Postgres|Plan: Forge|postgres-migration|forge-activity' docs/src/`
      returns zero stale references (the only remaining mentions, if any, are in this
      `rendering-modes` plan's README retirement note, which is acceptable until this
      plan itself retires).
- [ ] forge-activity/postgres-migration durable knowledge confirmed present in stable
      docs (api/activity.md, guides/activity.md, development docs) — list what was
      verified vs newly folded in the PR description.
- [ ] If `docs/book` is committed, it is rebuilt and contains no deleted-plan text.

## Files likely touched

- `docs/src/architecture/rendering.md` (new).
- `docs/src/SUMMARY.md` (add rendering page; remove both prior-plan sections; this
  set's own section stays until its `verify`).
- `docs/src/development/*` (only if folding a postgres-migration maintainer note not
  already covered).
- Deleted: `docs/src/planning/postgres-migration/`, `docs/src/planning/forge-activity/`.

## Pitfalls

- **Deleting before preserving.** Verify durable knowledge is in stable docs *before*
  `git rm`. Lost knowledge is unrecoverable; redundancy is recoverable.
- **Tracking-state assumption.** `git rm -r` fails on untracked files; check
  `git ls-files <dir>` and fall back to `rm -rf` for untracked remainders.
- **Stale-ref false negatives after a `cd`.** Run the `rg` sweeps with absolute paths
  (or `git -C`), since the shell cwd persists between commands.
- **Retiring this plan by accident.** Do not touch `docs/src/planning/rendering-modes/`
  — it retires on its own `verify`.
- **mdBook search index churn.** After deleting plan pages and rebuilding, the
  content-addressed `searchindex-*.js` filename changes — that's expected churn, not
  a leftover. Grep file *content* for the slugs, not filenames.

## Reference

- Retirement workflow + lessons: the `retire-docs-planning` skill (preserve durable
  knowledge, prune nav, delete files, sweep stale refs).
- Stable docs already carrying forge-activity knowledge: `docs/src/api/activity.md`,
  `docs/src/guides/activity.md`, `docs/src/configuration/*`.
- Plans being retired: [`../../postgres-migration/`](../../postgres-migration/),
  [`../../forge-activity/`](../../forge-activity/).
- Mode source of truth being documented: `crates/client/src/app.rs` (`app_routes()`).
