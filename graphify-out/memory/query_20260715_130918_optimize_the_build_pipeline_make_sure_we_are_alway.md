---
type: "query"
date: "2026-07-15T13:09:18.209254+00:00"
question: "Optimize the build pipeline make sure we are always on the same version so that we can get cache hits on sccache! Design and plan"
contributor: "graphify"
outcome: "useful"
source_nodes: ["Forgejo CI Workflow", "Atlas Nix Trusted Runner", "Canix Attic Cache", "Nix Flake Toolchain", "Flake Check Job", "Release Cache Job"]
---

# Q: Optimize the build pipeline make sure we are always on the same version so that we can get cache hits on sccache! Design and plan

## Answer

Expanded from original query via graph vocabulary: [atlas, attic, build, cache, cargo, flake, forgejo, nix, runner, rust, toolchain, version]. Findings: CI only triggers main/poc/develop while the sole remote branch is trunk; Nix jobs use atlas plus per-job install-nix instead of atlas-nix-trusted; native Plinth derivations do not enable RUSTC_WRAPPER; no rust-toolchain file pins local Cargo; Crane cargoArtifacts are reused across differing flags and profiles; Pkl import-from-derivation makes evaluation build. Plan: fix trunk triggers and trusted Atlas routing, use one persistent-store validation job and host Attic push, introduce one rust-toolchain.toml consumed by native/cross/Dioxus builders, assert Dioxus CLI/crate and wasm-bindgen and sccache versions, create compilation-class-specific cargo artifacts, add a versioned sccache namespace and telemetry, remove evaluation-time derivations with checked generated Pkl sidecars, then roll out rs-harbor to Plinth to canix with package-scoped Atlas gates before a full Crossbow closure.

## Outcome

- Signal: useful

## Source Nodes

- Forgejo CI Workflow
- Atlas Nix Trusted Runner
- Canix Attic Cache
- Nix Flake Toolchain
- Flake Check Job
- Release Cache Job