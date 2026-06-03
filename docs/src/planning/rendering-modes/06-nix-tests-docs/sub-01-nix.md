# Phase 06 · Sub-01 — Nix build matrix (islands + CSR)

> **Recommended Codex model: GPT 5.5 medium**
>
> Bounded leaf/sub-agent work on the flake, but with real consequence: the nix
> build is the CI gate, and the new modes changed the build outputs (islands flags,
> a CSR target). A medium model handles crane/cargo-leptos flag reconciliation; the
> risk is a flag mismatch between `Cargo.toml` and `flake.nix` that only surfaces in
> CI, so it's not trivial-tier.

## Working tree

`/data/nvme0/can/Projects/solo/plinth` — after Phases 04 (islands flags) and 05 (CSR
output) have landed. Edits `flake.nix` and the `Cargo.toml`
`[[workspace.metadata.leptos]]` block (shared with Phases 04/05 — rebase first).

## Goal

`flake.nix` builds and checks the full rendering matrix: the default server build
with islands enabled, and the `plinth-csr` static-site target, both reproducible and
green under `nix flake check`.

## Why this matters now

The flake builds via `cargo leptos build` with a fixed feature set
(`bin-features = ["ssr", …]`, `lib-features = ["hydrate", …]`). Phase 04 added
`islands = true` and `experimental-islands`; Phase 05 added a client-only CSR target.
Neither is reflected in the flake yet, so `nix flake check` either ignores the new
modes or fails. The flake is the CI source of truth — it must build what the code now
supports.

## Out of scope

- Changing rendering behavior (Phases 01–05).
- Deploy/NixOS-module changes beyond exposing the new package output.
- Test authoring (sub-02).

## Plan

1. **Reconcile the cargo-leptos feature block.** Confirm `[[workspace.metadata.leptos]]`
   carries `islands = true` and the island feature on `lib-features`/`bin-features`
   as Phase 04 set them; ensure the flake's `cargo leptos build` picks them up (it
   reads the workspace metadata, so usually no flake change beyond rebuild).
2. **Add the `plinth-csr` package output.** Build the `csr`-featured WASM bundle +
   static shell + CSS (per Phase 05's build-tool choice — Trunk or cargo-leptos CSR
   profile) into `$out` as a static site directory. Mirror the existing `plinth`
   package's source-filtering and crane `cargoArtifacts` reuse.
3. **Wire it into `checks`/`packages`.** Add `plinth-csr` to `packages`; if a build
   check is cheap, add it to `checks` so `nix flake check` exercises the CSR build.
4. **Keep the existing outputs intact** (`plinth`, `plinth-dev`, `plinth-minimal`,
   `plinth-cli`, `docs`). The islands change should be a rebuild of `plinth`, not a
   new package.
5. **Dev shell parity.** If Phase 05 chose Trunk, add `trunk` (and any wasm tooling
   not already present) to the dev shell so local CSR builds work.

## Acceptance criteria

- [ ] `nix flake check` is green: `plinth` builds with islands enabled (server +
      WASM client), clippy + fmt pass, tests pass against sandbox Postgres.
- [ ] `nix build .#plinth-csr` produces a static site directory containing the CSR
      WASM/JS + CSS + `index.html`.
- [ ] `nix build .#plinth` (default) still succeeds and its site assets include the
      island runtime.
- [ ] `nix develop` provides the toolchain for both SSR and CSR local builds.
- [ ] No feature-flag mismatch between `Cargo.toml` and `flake.nix` (the build uses
      the workspace metadata, not a divergent hard-coded feature list).

## Files likely touched

- `flake.nix` (`plinth-csr` package + check; dev-shell tooling if Trunk added).
- `Cargo.toml` (`[[workspace.metadata.leptos]]` reconciliation — coordinate with
  Phases 04/05).

## Pitfalls

- **Workspace-metadata vs flake feature drift.** `cargo leptos` reads
  `[[workspace.metadata.leptos]]`; hard-coding a different feature list in the flake
  silently diverges. Let the metadata drive.
- **CSR build tool not in the sandbox.** A Trunk-based CSR build needs `trunk` +
  `wasm-bindgen-cli` available in the nix build env, not just the dev shell.
- **`doCheck = false` on the leptos package.** Tests run in the dedicated
  `plinth-test` check (starts Postgres). Put CSR build verification where it belongs;
  don't try to run integration tests inside the WASM build.
- **Islands changes the site asset layout.** Verify the install phase still copies
  all `target/site/*` (island chunks included).

## Reference

- Current build: `flake.nix` (`buildPhaseCargoCommand = "cargo leptos build …"`,
  install phase, package set), `Cargo.toml` `[[workspace.metadata.leptos]]`.
- Upstream changes this depends on: [../04-islands.md](../04-islands.md) (islands
  flags), [../05-csr-profile.md](../05-csr-profile.md) (CSR target).
