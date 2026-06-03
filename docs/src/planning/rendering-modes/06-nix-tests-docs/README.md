# Phase 06 — Nix, tests, docs + plan retirement

> **Recommended Codex model for merge/orchestration: GPT 5.5 medium**
>
> The merge runs the full `nix flake check` across the new build matrix (default
> SSR+hydrate/islands and the CSR target) and confirms the three sub-layers' outputs
> compose. Moderate orchestration: reconciling build flags the sub-layers each
> touched and arbitrating the `Cargo.toml`/`flake.nix` overlap. Not high — the
> sub-layers are disjoint by construction and each carries its own acceptance
> subcriteria.

## Sub-layers

| # | Slug | Model | Touches | Sub-layer file |
|---|------|-------|---------|----------------|
| 01 | nix | 5.5 medium | `flake.nix`, `Cargo.toml` (`[[workspace.metadata.leptos]]`) | [sub-01-nix.md](./sub-01-nix.md) |
| 02 | tests | 5.5 medium | `crates/server/tests/` (new rendering-mode e2e tests) | [sub-02-tests.md](./sub-02-tests.md) |
| 03 | docs + retire | 5.5 low | `docs/src/architecture/`, `docs/src/SUMMARY.md`, prior plan dirs | [sub-03-docs-retire.md](./sub-03-docs-retire.md) |

## Goal (phase-level)

The new rendering matrix is fully validated and documented, and the two prior plans
this set supersedes are retired. `nix flake check` is green for both the default
build (SSR + hydrate, islands enabled) and the CSR build; the e2e suite proves each
route's rendering mode behaves as specified; the rendering architecture is
documented in stable docs; and `postgres-migration` + `forge-activity` are removed
from the published planning tree with their durable knowledge preserved.

## Why this matters now

Phases 01–05 introduced new build outputs (islands flags, a CSR target) and new
runtime behaviors (static routes, streaming, partial hydration) that the existing
`nix flake check` and test suite do not exercise. Without this phase the modes are
plausibly-but-unverifiably correct, the build matrix can drift, and the prior plans
that this set completes stay published as if still active. This phase closes the
loop: validate, document, retire.

## Out of scope

- Implementing any rendering mode (Phases 01–05 own those).
- Re-opening mode decisions — this phase verifies and documents what landed.
- Retiring this (`rendering-modes`) plan itself — that happens on this plan's own
  `verify` pass, after the merge is green.

## Merge plan

The user dispatches the three sub-layers (parallel or sequential), then runs the
merge themselves:

1. Land sub-01 (nix) and sub-02 (tests) first — they are pure additions to
   `flake.nix`/`Cargo.toml` and `crates/server/tests/` respectively and rarely
   conflict (sub-01 may touch the `Cargo.toml` leptos block that Phase 04/05 also
   edited — rebase before landing).
2. Land sub-03 (docs + retire) last: it deletes the prior plan directories and edits
   `SUMMARY.md`, which is cleanest once the build/test sub-layers are stable.
3. Run the merge gate:
   ```
   nix flake check                 # default build matrix
   nix build .#plinth-csr          # CSR target (from Phase 05)
   ```
4. Resolve any `Cargo.toml`/`flake.nix` overlap between sub-01 and Phases 04/05 in
   favor of the union of required flags (islands + CSR + SSG all enabled in their
   respective outputs).

## Phase-level acceptance criteria

- [ ] `nix flake check` is green: workspace builds (server SSR + WASM client with
      islands), clippy + fmt pass, and all named tests pass against sandbox Postgres.
- [ ] `nix build .#plinth-csr` produces a static site directory (Phase 05 output
      wired into the flake).
- [ ] The rendering-mode e2e tests (sub-02) pass and cover: static routes serve
      without per-request SQL, the home page streams incrementally, islands hydrate
      selectively, dynamic routes return fresh data, and no `todo!("phase 03")` panic
      path remains.
- [ ] `docs/src/architecture/rendering.md` exists and documents the per-route mode
      table matching `app.rs`; `SUMMARY.md` links it.
- [ ] `docs/src/planning/postgres-migration/` and `docs/src/planning/forge-activity/`
      are removed from `SUMMARY.md` and deleted; their durable knowledge is confirmed
      present in stable docs; `rg` finds no stale links to them across `docs/src`.

## Reference

- Build entrypoint: `flake.nix` (`cargo leptos build`), `Cargo.toml`
  `[[workspace.metadata.leptos]]`.
- Existing e2e test conventions to mirror: `crates/server/tests/` (e.g.
  `activity_brick.rs`, `activity_feed_search.rs`, `common/mod.rs` sandbox-Postgres
  harness).
- Prior plans being retired: [`../../postgres-migration/`](../../postgres-migration/),
  [`../../forge-activity/`](../../forge-activity/).
- Plan-set whole-set acceptance criteria: [../README.md](../README.md).
