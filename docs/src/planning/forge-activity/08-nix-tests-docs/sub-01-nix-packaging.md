# Phase 08 / Sub-01 — Nix packaging for the new `crates/forge` crate and `brick-activity`

> **Recommended Codex model: GPT 5.5 medium**
>
> This is small but high-leverage Nix surgery: crane filters workspace sources explicitly, so a new member is invisible to the sandbox build until added in two coordinated places (`flake.nix` fileset + root `Cargo.toml` members), and the cargo-leptos `bin-features`/`lib-features` lists must learn the new feature or the server binary and WASM client silently compile without it. A low tier risks editing only one of the two places and producing a build that "passes" locally (where the source tree is unfiltered) but fails under `nix flake check` (where it is filtered). Medium is right: no design, but multiple must-be-consistent edit sites and a real failure mode if any is missed. High is overkill — there is no architectural decision here.

## Working tree

`cwd = /data/nvme0/can/Projects/solo/plinth` (the plinth repo).

This sub-layer depends on Phases 01–07 having landed (the `crates/forge` crate exists with its `wiremock` dev-dependency added by Phase 02, and each crate's `Cargo.toml` already declares `brick-activity` in its `[features]` table). The only genuine new edits are to `flake.nix` and root `Cargo.toml`; the forge `[dev-dependencies]` block is verify-only (confirm Phase 02's entries are present, repair only on regression).

**Serialization note:** root `Cargo.toml`'s `bin-features` / `lib-features` lists may *also* have been edited by Phases 01/03/05 (each appended `brick-activity`). Before editing, check whether `brick-activity` is already present in those lists — this edit is idempotent; do not add a duplicate. Within Phase 08 no sibling sub-layer touches `flake.nix` or root `Cargo.toml`, so there is no in-phase conflict; `crates/forge/Cargo.toml`'s `[dev-dependencies]` is owned by Phase 02 and is verify-only here, so if sub-02 later adds forge-crate test deps it merges keys into that single existing block (see Pitfalls).

## Goal

This sub-layer succeeds when the new `crates/forge` workspace member is included in the crane source filter and the workspace `members` list, the `brick-activity` feature is compiled into both the server binary and the WASM client by cargo-leptos, the forge dev-dependencies (`wiremock` + tokio test macros) that Phase 02 added are confirmed present for the test sub-layer, and `nix build .#plinth` compiles the full workspace (including `crates/forge` and `brick-activity`) inside the Nix sandbox.

## Why this matters now

crane (`craneLib.fileset.commonCargoSources`) catches `Cargo.toml` and `.rs` files, but the project's established convention is to list every workspace member explicitly in the `src` fileset so non-`.rs` files (fixtures, future `migrations`) are not silently filtered out of the sandbox. Without the `./crates/forge` line, the new crate's sources can be dropped from the build sandbox and `nix flake check` fails to find the crate. Without `"crates/forge"` in `members`, cargo does not treat it as part of the workspace at all. Without `brick-activity` in `bin-features`/`lib-features`, `cargo leptos build` (which `buildPlinth` runs) compiles the server and client *without* the activity feature even though the crate compiles — the feature ships dead. This sub-layer is the gate that makes every other phase's code actually reachable from the Nix-built artifact and from CI.

## Out of scope

- Adding the `brick-activity` feature **entries** to per-crate `[features]` tables — Phases 01/03/05/06 own those. This sub-layer only **verifies** them and fixes the workspace-level wiring (`members`, source filter, leptos feature lists).
- Adding the `crates/forge` `wiremock` / tokio test dev-dependencies — Phase 02 owns those. This sub-layer only **verifies** they are present (and repairs only if a regression dropped them).
- Writing tests (sub-02) or docs (sub-03).
- Any change to `commonArgs`, `buildInputs`, `nativeBuildInputs`, the toolchain, or `buildPlinth`'s `installPhase` — the new crate is a library, ships no extra binary, and pulls in no new system dependency beyond what the workspace already links (`reqwest`/`openssl` are already in `buildInputs`).
- Adding a new `flake.nix` check for the non-default feature — clippy/test already run `--all-targets`/`--workspace` with default features, and `brick-activity` is in the defaults.

## Plan

1. **Add the crate to the workspace members.** Edit `/data/nvme0/can/Projects/solo/plinth/Cargo.toml` line 3:

   ```toml
   members = ["crates/shared", "crates/client", "crates/server", "crates/cli", "crates/forge"]
   ```

   Verify with `grep -n 'crates/forge' Cargo.toml`.

2. **Add the crate to the crane source filter.** In `/data/nvme0/can/Projects/solo/plinth/flake.nix`, inside the `src = lib.fileset.toSource { ... fileset = lib.fileset.unions [ ... ]; }` block (around lines 211–237), add one line alongside the four existing `# Workspace members` entries:

   ```nix
   # Workspace members
   (lib.fileset.maybeMissing ./crates/client)
   (lib.fileset.maybeMissing ./crates/server)
   (lib.fileset.maybeMissing ./crates/shared)
   (lib.fileset.maybeMissing ./crates/cli)
   (lib.fileset.maybeMissing ./crates/forge)   # <-- ADD THIS LINE
   ```

   This mirrors the existing four members exactly. `maybeMissing` makes it tolerant if the directory is absent (it will not be, post-Phase-02). Do not remove or reorder the existing lines.

3. **Verify the per-crate feature definitions exist (do not add — verify).** Confirm each crate already declares `brick-activity` (added by upstream phases). If any is missing, the upstream phase regressed — flag it, but the established shape is:
   - `crates/shared/Cargo.toml`: `brick-activity = []` (leaf marker), appended to `default`.
   - `crates/client/Cargo.toml`: `brick-activity = ["plinth-shared/brick-activity"]`, appended to `default`.
   - `crates/cli/Cargo.toml`: `brick-activity = ["plinth-shared/brick-activity"]`, appended to `default`.
   - `crates/server/Cargo.toml`: `brick-activity = ["plinth-client/brick-activity", "plinth-shared/brick-activity"]`, appended to `default`.

   Check with:
   ```bash
   grep -rn 'brick-activity' crates/*/Cargo.toml
   ```
   Each crate that consumes activity types/UI must show a `brick-activity` line; `default` should include it everywhere it was already a default for blog/portfolio/todo.

4. **Add `brick-activity` to the cargo-leptos compile feature lists.** In `/data/nvme0/can/Projects/solo/plinth/Cargo.toml`, under `[[workspace.metadata.leptos]]` (the cargo-leptos config; `bin-features` at line ~132 and `lib-features` at line ~138), ensure both lists contain `"brick-activity"` (idempotent — skip if Phases 01/03/05 already added it):

   ```toml
   # The features to use when compiling the bin target
   bin-features = ["ssr", "brick-blog", "brick-portfolio", "brick-todo", "brick-activity"]

   # The features to use when compiling the lib target (client package)
   lib-features = ["hydrate", "brick-blog", "brick-portfolio", "brick-todo", "brick-activity"]
   ```

   This is the single most-missed wiring: `buildPlinth` runs `cargo leptos build`, which uses these lists (not crate defaults), so without it the activity routes/pages compile in `cargo build` but are absent from the Nix-built server binary and WASM bundle.

5. **Verify the forge dev-dependencies exist (do not add — verify).** Phase 02 added `wiremock` (plus tokio test macros) to `/data/nvme0/can/Projects/solo/plinth/crates/forge/Cargo.toml` so sub-02's mocked-forge tests compile. Confirm the `[dev-dependencies]` block is present and contains both entries; only add them if a regression dropped them (and if so, merge into the existing block rather than duplicating the header). The established shape is:

   ```toml
   [dev-dependencies]
   wiremock = "0.6"
   tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }
   ```

   Check with:
   ```bash
   grep -n 'wiremock\|tokio' crates/forge/Cargo.toml
   ```

   `wiremock` is transport-level so it is compatible with the locked `reqwest 0.12.28`. It binds a loopback port inside the sandbox (permitted); real outbound HTTP is not (so the test sub-layer must mock — see sub-02). It must stay strictly under `[dev-dependencies]`; no production dependency is added.

6. **Local verification (this sub-layer's slice):**
   ```bash
   nix build .#plinth 2>&1 | tail -40
   ```
   Expect a successful build. Then a fast feature sanity check outside Nix:
   ```bash
   cargo build -p plinth-server --features brick-activity
   cargo build -p plinth-forge
   ```

7. **Do NOT run the full `nix flake check` here** — that is the merge agent's job after sub-02/sub-03 merge (the test check needs sub-02's tests present, and clippy `--deny warnings` covers the whole tree). Running `nix build .#plinth` is sufficient to prove this sub-layer's wiring.

## Acceptance criteria

- [ ] `grep -n '"crates/forge"' /data/nvme0/can/Projects/solo/plinth/Cargo.toml` returns the `members` line.
- [ ] `grep -n 'crates/forge' /data/nvme0/can/Projects/solo/plinth/flake.nix` shows `(lib.fileset.maybeMissing ./crates/forge)` inside the `src` fileset unions.
- [ ] `grep 'bin-features' /data/nvme0/can/Projects/solo/plinth/Cargo.toml` shows `"brick-activity"` present exactly once; same for `lib-features`.
- [ ] `grep -rn 'brick-activity' crates/*/Cargo.toml` shows a definition in shared (`= []`), client, cli, and server (chaining to `plinth-shared`/`plinth-client`).
- [ ] Verified: `crates/forge/Cargo.toml` `[dev-dependencies]` (added by Phase 02) contains `wiremock = "0.6"` and `tokio` with the `macros` + `rt-multi-thread` features (repaired only if a regression dropped them).
- [ ] `nix build .#plinth` exits 0.
- [ ] `cargo build -p plinth-server --features brick-activity` and `cargo build -p plinth-forge` both exit 0.

## Files likely touched

- `/data/nvme0/can/Projects/solo/plinth/Cargo.toml` — `members` array (add `"crates/forge"`); `bin-features` / `lib-features` (add `"brick-activity"` if absent).
- `/data/nvme0/can/Projects/solo/plinth/flake.nix` — `src` fileset (add the `./crates/forge` line).
- Verify-only (no edit unless a regression is found): `crates/{shared,client,cli,server}/Cargo.toml` feature tables; `/data/nvme0/can/Projects/solo/plinth/crates/forge/Cargo.toml` `[dev-dependencies]` (`wiremock`, `tokio`), owned by Phase 02.

## Pitfalls

- **Symptom:** `nix flake check`/`nix build` errors with "no matching package named `plinth-forge`" or filters away the crate.
  **Cause:** member added to `Cargo.toml` but not to the `flake.nix` `src` fileset (or vice versa).
  **Recovery:** both edits in steps 1 and 2 are mandatory and must be consistent. `commonCargoSources` alone is not relied upon by convention — add the explicit line.

- **Symptom:** `cargo build` is green and `nix build .#plinth` is green, but the running server has no `/api/activity` route / the home strip is missing in the WASM bundle.
  **Cause:** `brick-activity` not added to `bin-features`/`lib-features`; cargo-leptos compiled without the feature.
  **Recovery:** step 4. These lists, not crate `default`, drive the Nix artifact.

- **Symptom:** duplicate `"brick-activity"` entries in `bin-features` or a duplicate `[dev-dependencies]` header in `crates/forge/Cargo.toml`.
  **Cause:** an upstream phase already added it (Phases 01/03/05 for the feature lists, Phase 02 for the forge dev-deps); these are not idempotent if applied blindly.
  **Recovery:** grep first; only add if absent. The forge `[dev-dependencies]` block is verify-only here — if a regression repair is needed, merge keys into the existing section, never duplicate the header.

- **Symptom:** test sub-layer fails to compile with "unresolved import `wiremock`".
  **Cause:** Phase 02's forge `[dev-dependencies]` regressed (got dropped), or `wiremock` was placed under `[dependencies]` instead of `[dev-dependencies]` (which would pull a mock server into production and break the WASM/no-network constraint).
  **Recovery:** verify per step 5; if a regression dropped it, restore it strictly under `[dev-dependencies]` of `crates/forge`.

- **Symptom:** `plinth-test` check hangs or fails on network.
  **Cause:** a forge test made a real outbound request; the sandbox has no internet.
  **Recovery:** that is sub-02's concern, but if it surfaces here, confirm all forge HTTP in tests targets a `wiremock` `server.uri()`, never `api.github.com`/`codeberg.org`.

## Reference

- The crane source filter and the `bin-features`/`lib-features` facts are inlined above from the flake research; no need to read other files.
- Sibling sub-layers (CONTEXT only): [sub-02-e2e-tests.md](./sub-02-e2e-tests.md) consumes the `wiremock` dev-dep (added by Phase 02, verified by this sub-layer); [sub-03-docs.md](./sub-03-docs.md) is independent. The merge agent (see [README.md](./README.md)) runs the full `nix flake check` after all three merge.
