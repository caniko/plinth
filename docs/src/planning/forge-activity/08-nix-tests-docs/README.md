# Phase 08 — Nix packaging, end-to-end tests, and docs

> **Recommended model for merge/orchestration: GPT 5.5 medium**
>
> The merge step is mechanical but spans three concerns (Nix derivations, a sandbox-Postgres test harness, and mdBook authoring) and must reconcile changes from all of Phases 01–07. A low tier risks missing the cross-cutting `Cargo.toml` member edit or the `bin-features`/`lib-features` leptos lists, both of which silently break `nix build` rather than failing loudly. Medium handles the coordination plus the one genuine conflict surface (root `Cargo.toml`) without needing the design depth of a high tier.

This is the final phase of the "forge-activity" feature. It packages the new `crates/forge` crate and `brick-activity` feature into the Nix build, adds end-to-end integration tests proving the full add→refresh→rank→serve path, and writes the user-facing documentation. It is a **multi-sub-layer** phase: three sub-layers touch disjoint file trees and can run in parallel, then one merge pass runs `nix flake check`.

## Sub-layers

| # | Slug | Model | Touches | File |
|---|------|-------|---------|------|
| 01 | nix-packaging | GPT 5.5 medium | `flake.nix`, root `Cargo.toml` (members), `crates/forge/Cargo.toml` (dev-deps), `crates/server/Cargo.toml` / `crates/cli/Cargo.toml` / `crates/client/Cargo.toml` / `crates/shared/Cargo.toml` feature tables (verify only — Phases 01–07 added them) | [sub-01-nix-packaging.md](./sub-01-nix-packaging.md) |
| 02 | e2e-tests | GPT 5.5 medium | `crates/server/tests/forge_activity.rs` (new), `crates/server/tests/common/mod.rs` (extend), `crates/forge/tests/` (new, optional) | [sub-02-e2e-tests.md](./sub-02-e2e-tests.md) |
| 03 | docs | GPT 5.5 low | `docs/src/guides/activity.md` (new), `docs/src/configuration/*.md`, `docs/src/api/activity.md` (new), `docs/src/SUMMARY.md` | [sub-03-docs.md](./sub-03-docs.md) |

## Goal (phase-level)

This phase succeeds when, on a clean checkout with Phases 01–07 merged, `nix flake check` is fully green with `brick-activity` enabled: the new `crates/forge` crate compiles inside the Nix sandbox, the workspace clippy/fmt/test checks pass (including the new end-to-end activity tests against the sandbox Postgres), the mdBook docs build (`nix build .#docs`), and every new activity feature (CLI publish/curate, `[ranking]` config, forge tokens, the TTL, the `/api/activity` + `/feeds/activity.xml` endpoints) is documented and wired into `docs/src/SUMMARY.md`.

## Why this matters now

Phases 01–07 added a new workspace member (`crates/forge`), a new Cargo feature (`brick-activity`), new server routes, a Kameo refresh actor, CLI subcommands, and new config sections. None of that is exercised by CI until the Nix build knows the crate exists (crane filters sources explicitly — a new member is invisible until added), and none of it is provably correct until an end-to-end test drives the whole path with a mocked forge. Deferring this leaves the feature un-shippable: `nix flake check` would either skip the new crate or fail to find it, and a reviewer would have no documentation to validate the CLI/config surface against. This is the gate that turns "code exists" into "feature ships".

## Out of scope

- Any production behaviour change to the forge client, refresh actor, ranking SQL, CLI, server routes, or frontend — those are owned by Phases 02–07 and are assumed landed and correct.
- Adding the `brick-activity` feature *definitions* to each crate's `Cargo.toml` — Phases 01/03/05/06 already add the feature entries; sub-01 only **verifies** them and adds the **member** + **source-filter** + **leptos feature-list** wiring that no earlier phase owns.
- Performance benchmarking of the ranking query or refresh actor.
- Removing the `planning/forge-activity/` directory (keep as historical record).

## Merge plan

The three sub-layers touch disjoint files and can be executed in parallel, then merged:

1. **sub-01 (nix)** edits `flake.nix` and root `Cargo.toml`. It also adds `[dev-dependencies] wiremock` to `crates/forge/Cargo.toml`.
2. **sub-02 (tests)** adds `crates/server/tests/forge_activity.rs` and extends `crates/server/tests/common/mod.rs`. If it adds forge-crate unit/integration tests it touches `crates/forge/tests/` (new files, no conflict).
3. **sub-03 (docs)** edits only `docs/src/**`.

**Expected conflicts.** The only genuine shared file is the **root `Cargo.toml`**: sub-01 edits the `members` array and the `bin-features`/`lib-features` lists. No other sub-layer touches root `Cargo.toml`, so there is no in-phase conflict — but note that Phases 01/03/05 may *also* have appended `brick-activity` to `bin-features`/`lib-features`. The merge agent must verify those lists already contain `brick-activity` (idempotent — do not duplicate) and that `members` contains `"crates/forge"`. `crates/forge/Cargo.toml` is touched by sub-01 (`[dev-dependencies]`) and possibly Phase 02 (the crate body) — sub-01 only appends a `[dev-dependencies]` block, an append-only edit.

**Who runs `nix flake check`.** The **merge agent** runs the full `nix flake check` once all three sub-layers are merged. Sub-layers SHOULD each verify their own slice locally first (sub-01: `nix build .#plinth`; sub-02: `cargo test -p plinth-server --test forge_activity`; sub-03: `nix build .#docs`), but only the merge agent owns the green-gate.

**Merge order.** sub-01 must land before the merge agent runs the full check (the crate must be in the source filter for the test sub-layer to compile under crane). Run order: merge sub-01, then sub-02 and sub-03 in either order, then `nix flake check`.

## Phase-level acceptance criteria

Only checkable after all three sub-layers are merged:

- [ ] Root `Cargo.toml` `members` contains `"crates/forge"`; `bin-features` and `lib-features` both contain `"brick-activity"` exactly once.
- [ ] `flake.nix` `src` fileset includes `(lib.fileset.maybeMissing ./crates/forge)`.
- [ ] `nix build .#plinth` succeeds (the workspace, including `crates/forge` and `brick-activity`, compiles inside the sandbox).
- [ ] `nix flake check` is green: `plinth`, `plinth-clippy` (`--all-targets -- --deny warnings`), `plinth-fmt`, `plinth-test` (`--workspace --all-targets` against sandbox Postgres), and `wasm-bindgen-version-check` all pass.
- [ ] `cargo test -p plinth-server --test forge_activity` runs and passes (named in sub-02).
- [ ] `nix build .#docs` succeeds; `docs/src/SUMMARY.md` lists the new activity guide, config additions, and the activity API page.
- [ ] `mdbook build docs` (or `nix build .#docs`) emits no broken-link / missing-file warnings for the new pages.

## Reference

- Sub-layers (CONTEXT for sequencing only — each sub file is standalone): [sub-01-nix-packaging.md](./sub-01-nix-packaging.md), [sub-02-e2e-tests.md](./sub-02-e2e-tests.md), [sub-03-docs.md](./sub-03-docs.md).
- Upstream phases (must all be landed before this phase): `../01-shared-types-and-migration.md` … `../07-feed-and-search.md`.
- Design brief: the "forge-activity" plan root (this `planning/forge-activity/` directory).
