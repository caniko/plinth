# Dioxus Program Cutover Plan

Date: 2026-07-13

Reference implementation:
[`dioxus-cutover-plan.md`](dioxus-cutover-plan.md)

## Outcome

Finish the Dioxus 0.7.9 transition across every first-party consumer, retire
the superseded Leptos/HTMX/Iced presentation paths after their individual
rollback windows, and move reusable Nix/DX build mechanics into `rs-harbor`.
Each product keeps its own domain model, backend, route contract, deployment,
and rollback policy. `tartan-ui` remains the shared presentation boundary;
`rs-harbor` remains the shared build boundary.

This is a program plan over the existing project-specific plans. It does not
replace their route, authorization, persistence, visual, accessibility,
performance, or production gates.

## Terminology And Inventory Boundary

No dependency, crate, package, source symbol, or repository named `Dioxide`
exists in the owned repository corpus. This plan therefore treats “Dioxide”
as “Dioxus.” If `Dioxide` is a separate unpublished component, it must be
added to the inventory and dependency graph before execution.

The first-party Dioxus inventory is closed as follows:

| Repository | Current role | Program state |
|---|---|---|
| `codeberg.org/caniko/rs-harbor` | Reproducible Rust/Nix packaging | Explicit web/fullstack builders, exact bindgen resolver, real fixture, and docs are landed; the site publisher is now isolated in `./site` and the release/pin remains open |
| `codeberg.org/caniko/tartan-ui` | Framework-neutral contracts plus shared Dioxus components | Target-neutral default and explicit `web`/`server` forwarding are landed; consumers still need to pin a released revision |
| `codeberg.org/caniko/plinth` | Fullstack SSR/hydrated site and CMS | The default production package now uses `mkDioxusFullstackPackage` with a Plinth-owned wrapper/CLI; dev/minimal/CSR and the legacy rollback seam remain until their retirement release |
| `codeberg.org/caniko/foundry-circle` | Dioxus operator console for the Foundry control plane | Uses the shared shell, dashboard, metric, and empty-state components; current Tartan revision is aligned and production route/visual evidence remains |
| `codeberg.org/caniko/queryfabric` | Dioxus SyQL editor surface | The Dioxus editor preserves QueryFabric's existing DOM/JavaScript contract but does not yet consume Tartan; inventory it as an editor-only consumer and keep query semantics product-owned |
| `github.com/memorycircuits/SynDB` | Fullstack scientific-data UI | Source cutover landed; isolated fullstack package/image and CI paths are green; owner-branch integration and production evidence remain |
| `gitlab.com/caniko/pink-raven` | Authenticated curation UI | Dioxus web output now uses the shared builder and hashed-asset smoke; production evidence, rollback, and legacy retirement remain |
| `gitlab.com/canikolabs/nomos/bikipy` (`Bekiper`) | Browser and desktop annotation application | Explicit web builder consumer is green; full editor parity, web observation, Native promotion, and Leptos/Iced retirement remain |

`gitlab.com/clg-gaming/regicide/vendor/veritasium` is excluded: the Dioxus
hit is an optional, disabled `dioxus-devtools` dependency inside a vendored
Bevy fork, not a first-party application or enabled root dependency.

Re-run the inventory before every program release. New first-party hits may
not bypass this plan; vendored or generated hits need a recorded exclusion.

### Execution checkpoint — 2026-07-13

The shared-builder and web-canary portions of this plan have now been
implemented. `rs-harbor` exports `mkDioxusWebPackage`,
`mkDioxusFullstackPackage`, `mkDioxusBuildPlan`, and
`resolveWasmBindgenCli`; its web and real fullstack fixtures now pass, including
server/public output and hashed JS/WASM asset assertions. Tartan UI's
shared Dioxus crate defaults to no renderer and its web/server target checks
pass. Bekiper and Pink Raven use the explicit web builder, and both real Nix
bundle builds pass (Bekiper records the pinned Binaryen SIGABRT warning that
the Dioxus CLI treats as a non-fatal optimizer failure). Plinth's packaged
server smoke, web/CSR builds, and feature checks pass.

SynDB's dirty `rapid` worktree remains untouched. An authorized isolated
worktree now exists at `/tmp/syndb-dioxus-closeout` on branch
`codex/dioxus-cutover`; it builds the fullstack package and OCI image against
the local rs-harbor helper worktree, compiles `syndb-ci`, and builds the docs.
The owner must still integrate that branch and update the SynDB lock to a
released rs-harbor revision before publishing. Production observation,
rollback evidence, and legacy retirement remain open for every consumer and
are not inferred from these build results.

### Implementation checkpoint — 2026-07-14

The cycle-isolation implementation is now present in the rs-harbor worktree:
the root library no longer imports Plinth (directly or through its former
`nix-pklx` convenience input), `./site` owns the optional Plinth-powered
publisher, and the Pages workflow builds/deploys that nested flake. Root and
nested `nix flake check --no-build` evaluations pass, but the site lock still
points at the last published rs-harbor revision until this worktree is
released.

Plinth now exposes `plinth-dioxus-helper` and composes it into the default
`plinth` package alongside the product CLI. A real local build produced the
Dioxus server, hashed browser assets, Tailwind stylesheet, and Plinth's
render-cache-aware wrapper. The dev, minimal, CSR, cross, and legacy paths are
deliberately retained as rollback seams; they still need separate canaries
before retirement. No consumer lock has been advanced to an unreleased
rs-harbor revision, and no production observation or rollback window is
closed by these local builds.

### Audited revision snapshot

| Repository | Revision / branch | Worktree and planning consequence |
|---|---|---|
| `rs-harbor` | `trunk` + uncommitted implementation | generic helper work, cycle-isolated `./site`, and real fixtures are present; commit/release still required |
| `tartan-ui` | `fb1010b` + clean worktree | shared shell, navigation, card, preview, feedback, loading, and empty-state primitives are available to consumers |
| `plinth` | `trunk` + uncommitted implementation | default production uses the shared fullstack helper; profile/CSR/retirement gates remain |
| `foundry-circle` | `trunk` + uncommitted implementation | shared shell/dashboard primitives are consumed; production route and visual evidence remain |
| `queryfabric` | `trunk` + clean editor consumer | Dioxus SyQL editor preserves its existing browser contract; no Tartan presentation dependency yet |
| `SynDB` | `635f7b7` / `rapid` + `codex/dioxus-cutover` | owner checkout remains dirty; isolated closeout is validated, but integration and released rs-harbor pin remain |
| `pink-raven` | `trunk` + uncommitted implementation | shared web builder and hashed-asset check are present; production evidence remains |
| `bikipy` | `trunk` + uncommitted implementation | explicit `mkDioxusWebPackage` canary builds; parity/observation remain |

The migrated first-party applications and Tartan resolve Dioxus/Dioxus Router 0.7.9 and
`wasm-bindgen` 0.2.126 at this snapshot. Plinth, Foundry Circle, Pink Raven,
SynDB, and Bekiper now pin Tartan `fb1010b`, which contains the generic Dioxus
component surface. Production release, lock refresh, and observation remain
open for each owner checkout.

### Implementation checkpoint — 2026-07-17

The explicit inventory now includes Foundry Circle and QueryFabric. Foundry
Circle and Plinth are aligned to the current Tartan revision used by Pink
Raven, SynDB, and Bekiper. Plinth's shared shell/navigation, home card grid,
loading states, and empty state now consume Tartan primitives; Pink Raven's
search and plan previews use `MediaPreview`; SynDB's route flash surface uses
`FeedbackBanner` while preserving its existing test id and route contract.

QueryFabric remains an editor-only Dioxus consumer for now: its DOM and
JavaScript contract is not generic enough to move into Tartan without a second
independent consumer. Product-specific editors, graph views, annotation
surfaces, plan semantics, and authorization remain outside Tartan.

The Plinth project-site config, site build, and `plinth-site-beauty` desktop and
mobile visual audit now pass; the generated evidence is under
`target/site-audit/`.

## Contracts Learned From Plinth

The following decisions are promoted from Plinth into program invariants:

1. Pin Dioxus crates and `dx` to the same stable patch. Keep
   `default-features = false` and make each app own its target features.
2. Build browser and server graphs separately. A browser graph containing a
   product server crate, SQLx/database drivers, actor runtimes, secrets, or
   native model clients fails the cutover gate. Pinned Dioxus Fullstack may
   contribute internal Axum/Tokio edges; record those separately and gate on
   product server-code exclusion instead of an impossible crate-name ban.
3. Keep domain types and authorization in framework-neutral crates. Dioxus
   components receive authorized, serializable view data.
4. Preserve stable HTTP APIs and custom Axum middleware. Use Dioxus server
   functions only for UI-private typed RPC whose generated encoding is an
   accepted contract.
5. Freeze routes, status codes, metadata, freshness, invalidation, asset URLs,
   browser storage, and no-JavaScript behavior before porting views.
6. Treat streaming as a route-specific product decision. Resolve metadata and
   status before the first committed chunk.
7. Do not depend on Dioxus 0.7.9 incremental rendering where explicit
   invalidation or durable cache ownership is required. Application caches
   stay application-owned.
8. Make route splitting a measured opt-in. It must produce real chunks and
   pass direct-load, refresh, navigation, N-1 asset, and rollback tests.
9. Build Tailwind/assets without network access in Nix. Stable public URLs and
   content-hashed bundle URLs have different cache contracts.
10. Derive the exact `wasm-bindgen` CLI from `Cargo.lock`; do not duplicate a
    hand-maintained version/hash implementation in every repository.
11. Keep old and new artifacts through a rehearsed observation window, then
    delete the legacy stack in a separate release.
12. Share build mechanics through `rs-harbor`, presentation primitives through
    `tartan-ui`, and product behavior nowhere outside its owning repository.

## Dependency And Release Order

```text
Plinth stop-ship repair + proven package contract
                         |
                         +-----------------------+
                                                 v
meta-harbor Dioxus kind ---> rs-harbor web/fullstack builders + fixtures
                                      ^          |
rs-harbor site-input decoupling ------+          +--> Bekiper web canary
                                                 |
tartan-ui explicit target matrix ----------------+
                                                 |
                                      +----------+----------+
                                      |                     |
                                      v                     v
                           Plinth shared packaging   Pink shared packaging
                                      |                     |
                                      +----------+----------+
                                                 |
                                                 v
                                  SynDB isolated closeout

Bekiper product parity: Web cutover --> Leptos retirement
                       Native proof  --> Iced retirement (separate horizon)
```

`rs-harbor` currently inputs Plinth to build its own site while Plinth consumes
`rs-harbor`; that site-only lock relationship is a cycle, not the desired
library dependency direction. Before the first shared-builder release, remove
or isolate the Plinth site input so updating build helpers cannot recursively
force a consumer update. After that decoupling, consumers pin tested
`rs-harbor` and `tartan-ui` revisions and lock updates move downward through
the graph. Product implementation tracks may run in parallel after the shared
contracts land, but production observation windows do not overlap on the same
host.

## `rs-harbor` Ownership And API

### Existing seam

`rs-harbor.lib.mkDioxusPackage` already runs an offline Dioxus web bundle and
Bekiper consumes it. Preserve that API and output layout during the first
release. Its current check validates only derivation shape and does not build
the Dioxus derivation; that is not sufficient evidence. The helper is web-only
despite accepting a generic platform, labels its artifact as `trunk-builder`,
and assumes the flake package attribute equals `pname`. Treat each as an
explicit compatibility defect, not as behavior to copy into the generalized
API.

### Target public helpers

1. `mkDioxusBuildPlan`
   - Purely normalizes the platform, package/bin selection, client/server
     features, DX arguments, Cargo tail arguments, output layout, and artifact
     metadata.
   - Rejects invalid combinations such as server-only features in a web-only
     profile or `wasmSplit = true` without the required feature set.
   - Maps a 0.7.9 split profile to the verified experimental `--wasm-split`
     flag; CLI flags are version-aware rather than guessed from the Nix
     argument name.
2. `mkDioxusWebPackage`
   - Builds a CSR or hydrated web distribution with the pinned toolchain,
     offline Cargo vendor config, DX, optional measured split mode, and a
     declared public-output location.
   - Replaces the implementation of `mkDioxusPackage`; the old name remains a
     compatibility wrapper with identical arguments and output for one full
     consumer migration cycle.
3. `mkDioxusFullstackPackage`
   - Models distinct client and server package/bin/features, including DX's
     separate-entry-point support.
   - Installs the native server and adjacent public tree under an explicit
     layout and exposes both paths through passthru metadata.
   - Emits or validates a public manifest so callers can distinguish hashed
     immutable assets from stable URLs that must revalidate.
   - Supports caller-provided extra binaries, install hooks, and wrappers
     without learning product-specific environment variables or cache policy.
4. Shared `resolveWasmBindgenCli`
   - Extract the lockfile parser/resolver currently embedded in the Trunk
     helper and use it from both Trunk and Dioxus builders.
   - Fail with the required version and available Nix attributes when no exact
     CLI exists. A loose fallback is forbidden for release bundles.
5. `mkDioxusVersionCheck`
   - Proves that Dioxus, Dioxus Router, and DX use the same patch line and
     exposes resolved Dioxus and `wasm-bindgen` versions as package metadata.

`mkDioxusWebPackage` and `mkDioxusFullstackPackage` accept either a complete
toolchain bundle or explicit Rust/Crane members. The WASM toolchain gains the
date, extensions, and extra-target controls already supported by
`mkToolchain`; it may not silently follow a rolling nightly. Clang/mold inputs
are platform-aware, and the Binaryen compatibility wrapper is enabled only
for pinned Dioxus/DX versions that require it.

Source filtering, git dependency output hashes, vendor overrides, application
wrappers, render caches, router composition, backend startup, database state,
and deployment modules remain caller-owned.

The fullstack helper may provide generic public-tree discovery and optional
`DIOXUS_PUBLIC_PATH` wrapper injection. Product-specific variables, wrapper
names, cache directories, and runtime policy remain caller-owned.

Do not claim generic desktop/mobile/native packaging from the existing helper:
it assumes a web `public/index.html` output. Add platform-specific public APIs
only after a real fixture proves each output, signing/resource contract, and
host requirement.

### Compatibility sequence

1. Add the pure build plan and shared lock resolver without changing the old
   helper.
2. Add web and fullstack helpers plus real fixtures.
3. Reimplement `mkDioxusPackage` as a compatibility call to
   `mkDioxusWebPackage`; prove Bekiper's output diff is empty.
4. If `meta-harbor` gains `dioxus-builder`, accept both old
   `trunk-builder` metadata and the new kind for one cycle.
5. Move all consumers to explicit helper names, then deprecate—but do not yet
   remove—the compatibility name.

### Required `rs-harbor` tests

- shape/evaluation tests for every valid and invalid profile;
- a real, hermetic web fixture whose check depends on and inspects the built
  derivation;
- a real fullstack fixture proving server executable, public index, hashed
  assets, direct route, and adjacent-output lookup;
- offline git-vendor and workspace-source fixtures;
- exact Dioxus/DX and `wasm-bindgen` match plus intentional mismatch failure;
- pinned/date-aware WASM toolchain and platform-aware linker-input checks;
- client/server feature-tree checks with forbidden dependency lists;
- opt-in split fixture or a recorded pinned-upstream blocker where 0.7.9's
  splitter fails;
- passthru/artifact metadata and backward-compatible Bekiper output checks;
- Nix sandbox proof that no tool or asset is downloaded during the build.

## Complete Dioxus Feature Disposition

The detailed capability-level decisions in Plinth's
[feature disposition matrix](dioxus-cutover-plan.md#dioxus-feature-disposition-matrix)
remain the program baseline. The exact Dioxus 0.7.9 Cargo surface is accounted
for below; availability never implies that every flag belongs in every binary.

| Exact feature(s) | Program disposition |
|---|---|
| `default` | Reject for production dependencies; it implicitly enables development/logger behavior. |
| `lib` | Adopt as the explicit common umbrella. |
| `asset`, `cli-config`, `document`, `hooks`, `html`, `macro`, `mounted`, `signals`, `warnings` | Adopt through `lib`; enable directly only for a deliberately smaller profile. |
| `minimal` | Reserve for isolated render/tests or a measured tiny client; no current production app needs a second base profile. |
| `launch` | Adopt in application entry crates; shared libraries do not own launch. |
| `router` | Adopt in every routed application; editor-only crates opt in only when they expose routes; Tartan UI stays router-neutral. |
| `web` | Adopt in every browser client profile. |
| `fullstack`, `server` | Adopt for Plinth, SynDB, and Pink Raven server profiles; Bekiper keeps its existing registry HTTP/WebTransport backend. |
| `ssr` | Use through `server`; direct use is reserved for component/snapshot renderers that do not need Fullstack. |
| `devtools` | Development-only opt-in and absent from release dependency trees. |
| `logger` | Reject as a global replacement; integrate with each product's tracing/telemetry policy. |
| `wasm-split` | Experimental, measured opt-in with `dioxus-router/wasm-split`; never folded into a normal `web` feature. |
| `desktop`, `mobile` | Reserve as separate products. Bekiper explicitly does not ship a permanent WebView desktop target. |
| `native` | Bekiper-only experimental preview until its independent renderer, accessibility, lifecycle, and performance gates pass. |
| `liveview` | Reject for these cutovers; a persistent server-render socket adds no accepted product value. |
| `third-party-renderer` | Reject until an owned renderer has a separate design and test contract. |

The supporting crates are explicit too:

| Exact `dioxus-router` feature | Program disposition |
|---|---|
| `default` | Reject implicit defaults; request `html` explicitly. |
| `html` | Adopt for application route rendering. |
| `streaming` | Route-specific opt-in after status/metadata-before-commit proof. |
| `wasm-split` | Experimental only, paired with Dioxus `wasm-split` and the DX experimental flag. |

| Exact `dioxus-fullstack` feature | Program disposition |
|---|---|
| `default` | Reject because it silently selects `ws`. |
| `web` | Adopt only in the browser half of a fullstack app. |
| `server` | Adopt only in the server half of Plinth, SynDB, and Pink Raven. |
| `native` | Reserve for a future promoted native transport; it is not Bekiper's current registry transport. |
| `ws` | Reserve until a product accepts a persistent socket endpoint and its operations contract. |
| `msgpack`, `postcard` | Reserve until measured payload/compatibility requirements justify a codec change. |

### Per-project feature profiles

| Project | Normal client | Normal server | Optional/experimental |
|---|---|---|---|
| Plinth | `web` plus brick features | `server` plus the same brick features | `devtools`; measured `wasm-split` |
| SynDB | explicit `web` | explicit `server` | `devtools`; split only after bundle measurement |
| Pink Raven | explicit `web` | explicit `server` | `devtools`; move split out of normal `web` |
| Bekiper | explicit `web` | no Dioxus server profile | `web-split`; later `native` preview |
| Tartan UI | no implicit target default | target feature forwarded by consumer | `devtools`, `wasm-split`, and tested renderer forwarders only |

`tartan-ui-dioxus/web` must not be enabled in a workspace dependency shared by
server builds. Each consumer forwards the matching Tartan target from its own
target feature. Tartan's current default-web behavior receives a compatibility
release before becoming explicit/no-default.

## Phased Program Execution

### Phase 0 — Freeze inventory and baselines

- record the five first-party consumers and the vendored exclusion;
- capture each repository revision, branch, dirty-worktree state, Dioxus/DX
  version, target feature trees, bundle outputs, and existing plan status;
- record Plinth's packaged 500, malformed standalone cache path, broken CSR
  stylesheet reference, stable-but-immutable JS/WASM, aarch64 output-path
  error, and missing production middleware/shutdown behavior as baseline
  failures rather than post-cutover cleanup;
- record Pink Raven's source/plan drift: Dioxus routing is unconditional and no
  production `UiMode` exists, despite the old plan requiring a same-revision
  configuration rollback;
- record the `rs-harbor` -> Plinth site input and Plinth -> `rs-harbor` build
  input as a site-only lock cycle that must be removed or isolated;
- record SynDB's dirty `rapid` checkout as owner-owned and leave it untouched;
  use the authorized `codex/dioxus-cutover` worktree for implementation, then
  validate owner integration with `git status --porcelain` and the lock revision
  before release;
- freeze route/auth/data/asset/performance evidence in each product's local
  plan; do not replace missing evidence with compile success;
- record current package output paths so helper adoption can be diffed.

Exit: every hit has an owner, disposition, local acceptance suite, and rollback
boundary.

### Phase 1 — Repair and prove the Plinth reference seam

Repair the reference before extracting its build pipeline:

- place `Extension(PageCache)` outside the middleware that extracts it and
  serve with `into_make_service_with_connect_info::<SocketAddr>()`;
- fix shell quoting so the standalone wrapper derives a real package-namespaced
  cache path, and prove an explicit NixOS `STATE_DIRECTORY` still overrides it;
- make the CSR stylesheet install path match its document reference;
- content-hash generated assets or make stable names revalidate, then assert
  the manifest and headers from the packaged server;
- restore HSTS/security headers, the 2 MiB request limit, compression, request
  tracing, graceful shutdown, actor teardown, and telemetry shutdown;
- repair and race-test page-cache single-flight notification, resolve the
  actual aarch64 target output, and make the declared WASM optimization setting
  observable in the output;
- keep route-specific rendering, invalidation durability, and UI parity on the
  Plinth acceptance list; a healthy process alone does not close those gates.

Add a package-level PostgreSQL smoke that starts the realised binary outside
the source tree and proves health, `/`, real 404 status, a rate-limited API,
CSS/JS/WASM, a cached route, an authenticated mutation with immediate
invalidation, and orderly shutdown. The package check, Playwright suite, and
streaming test become CI dependencies rather than optional local scripts.

Exit: the current manual package is operationally safe, its public/server
layout is recorded as the extraction contract, and no generic builder code has
been copied from a known-broken artifact.

### Phase 2 — Land and release `rs-harbor` builders

First remove or isolate `rs-harbor`'s Plinth-powered site input so the generic
library has a one-way consumer relationship. Add the `dioxus-builder` artifact
kind if chosen, then implement the API and compatibility sequence above. Use a
local minimal fixture—not Plinth, which would recreate the lock cycle. Exercise
real web and fullstack derivations before any product deletes local build
logic, and publish the helper contract and migration notes.

Exit: real offline web/fullstack fixtures are green, mismatch tests fail as
designed, `mkDioxusPackage` compatibility is proven, and the helper release no
longer depends on a consumer site.

Implementation status: the root/site split and local checks are complete in
the current worktree. The remaining exit item is a released revision and
consumer lock update; the nested site lock intentionally remains on the last
published revision until then.

### Phase 3 — Normalize Tartan UI

- remove implicit target selection from the durable API through a compatible
  transition;
- forward only tested `web`, `server`, `desktop`, `mobile`, `native`,
  `devtools`, and split features; do not add app routing or backend behavior;
- compile the shared component gallery for every claimed target;
- pin the released revision independently in each consumer.

Exit: server profiles do not acquire `web` through Tartan and every advertised
forwarder has a real build.

### Phase 4 — Use Bekiper as the web compatibility canary

- update the existing `mkDioxusPackage` call to `mkDioxusWebPackage` while
  retaining its one-cycle package alias;
- prove a real package build, public-root/deep-link behavior, hydration,
  referenced assets, cache headers, and output-layout diff;
- finish the deployment rename from `wasmGuiPath`/`wasm_gui_path` to
  `webGuiPath` with a one-release `mkRenamedOptionModule` compatibility path;
- complete Dioxus Web parity against the full Iced behavior baseline, not the
  reduced Leptos shell, and cut the browser product over through its own
  observation and rollback window;
- remove Leptos/Trunk only in the subsequent web-retirement release.

Exit: the first real downstream proves the web helper and the production web
path; the Native/Iced transition remains explicitly separate.

### Phase 5 — Adopt shared packaging and finish Plinth

- remove global `tartan-ui-dioxus/web` leakage and forward Tartan features from
  Plinth's explicit web/server profiles;
- replace the manual native-server, WASM, wasm-bindgen, CSR, and aarch64
  assembly with `mkDioxusFullstackPackage`/`mkDioxusWebPackage` while
  preserving every flake output, CLI, wrapper, public path, render-cache
  namespace, cross target, NixOS module, and rollback generation;
- finish the planned cached/fresh/streaming dispatcher, synchronous/fallible
  targeted invalidation, request-race tests, stable REST-vs-server-function
  decision, security/observability parity, and every missing shell/page/SEO/
  accessibility behavior;
- run packaged, Playwright, no-JS, cache, asset, performance, staging,
  observation, and full-generation rollback gates;
- only then remove `crates/client`, the legacy binary/features,
  cargo-leptos metadata, Leptos variables, HTMX, old fixtures, and stale docs in
  a distinct retirement release.

Exit: Plinth is the proven fullstack/cross consumer, passes legacy-absence
gates, and has durable non-planning build/deploy/rollback guidance.

Implementation status: the default production package now uses the shared
fullstack helper and a Plinth-owned wrapper/CLI composition. Dev, minimal, CSR,
cross, and legacy paths remain as explicit rollback seams until their separate
canaries and retirement gates pass.

### Phase 6 — Reconcile and finish Pink Raven

Do not execute the old Phase 09 literally: current source routes documents
through Dioxus unconditionally and has no `PINK_RAVEN_UI_MODE`/`UiMode` switch.
First amend the local plan to choose either a real same-revision mode switch or
a tested previous-Nix-generation rollback, and record the superseded claims.

Then move `wasm-split` out of the normal `web` feature, reconcile the missing
`assets/` path with checked-in `static/`, and adopt the fullstack helper while
preserving wasm-opt/compression hooks and all service outputs. The upstream
producer for missing production evidence is the Pink Raven/canix release
workflow: record the exact source revision, run its Nix checks/builds, update
only the `pink-raven` canix lock, evaluate and switch atlas, verify public/OIDC/
authenticated/reversible-write flows, observe at least 24 hours plus a normal
curator session, and rehearse the chosen rollback. Until that evidence exists,
legacy deletion is blocked rather than inferred from source ownership.

Exit: production and rollback evidence is archived, then Leptos, HTMX, legacy
scripts, and the old bundle are removed in a separate release.

### Phase 7 — Close SynDB in an isolated worktree

Schedule SynDB last unless its owner provides an authorized clean worktree;
never rewrite or hide the unrelated dirty `rapid` changes. In that worktree:

The 2026-07-13 execution checkpoint created
`/tmp/syndb-dioxus-closeout` (`codex/dioxus-cutover`). Its `nix build
.#syndb-ui`, `.#oci-syndb-ui`, and `.#syndb-docs` gates, `nix flake check
--no-build`, rs-harbor helper checks, and `syndb-ci` compile/test gates pass
when the local rs-harbor worktree is supplied as an input override.

- make Dioxus and Tartan `web`/`server` features mutually explicit;
- ensure root `Dioxus.toml`, the public tree, and generated ETL asset are all in
  the builder source boundary;
- replace mutable `nix develop -c dx build`/`target/dx` Docker staging with the
  fullstack helper while preserving generated-asset assertions and exact OCI,
  Compose, and Helm layouts;
- run the missing public-host/browser/Helm evidence, replace stale package
  names, and move durable decisions into architecture/operations docs;
- close or retire the execution plan. Do not recreate its already-deleted
  legacy UI solely to mimic another product's rollback mechanism.

Exit for this isolated checkpoint: SynDB passes target isolation, immutable
package/image, CI, docs, and plan-closeout gates without touching the owner's
unrelated work. Public-host/browser evidence, Helm execution, release pinning,
and production observation remain owner/release gates.

### Phase 8 — Promote Bekiper Native as a separate product horizon

Resolve the pinned Dioxus Native/Blitz image-version conflict first. Then prove
full Iced parity, renderer lifecycle and device-loss recovery, AccessKit,
keyboard/IME/clipboard/window behavior, visual output, latency, resource use,
and platform packaging. Keep Iced as the rollback implementation through a
separate Native observation window; delete it only after Native is promoted.

Exit: Bekiper has production Dioxus Web and Dioxus Native, with neither Leptos,
Trunk, nor Iced in a production entrypoint.

### Phase 9 — Program retirement

- run a fresh owned-repository inventory and reconcile every new hit;
- remove the `mkDioxusPackage` compatibility name only after no consumer uses
  it and the announced deprecation window has elapsed;
- remove any temporary `trunk-builder` compatibility metadata after downstream
  package-test consumers accept the Dioxus kind;
- publish durable rs-harbor packaging docs and per-product architecture,
  testing, upgrade, deployment, and rollback docs;
- archive this program plan with exact revisions, commands, results, skipped
  gates, and externally blocked evidence.

## Verification And Rollout Gates

Every consumer records at least:

- every applicable native/server and `wasm32-unknown-unknown` build with
  warnings denied;
- `cargo tree -e features` evidence for target isolation and exact pins;
- hermetic Nix build with no network/tool download;
- packaged server/public output, asset hash/cache, deep-link, refresh,
  hydration, and N-1 asset tests;
- route/status/metadata/auth/API/browser-storage parity;
- accessibility, visual, browser console, and numeric performance budgets;
- production-shaped deployment plus rollback rehearsal;
- an observation window followed by a distinct legacy-retirement release.

Program rollout is lockfile-driven and reversible:

1. repair and package-smoke Plinth without changing shared helpers;
2. decouple the site-input cycle, then release and pin `rs-harbor`;
3. release and pin target-explicit `tartan-ui`;
4. prove the web helper in Bekiper, then adopt helpers without changing product
   routing in Plinth, Pink Raven, and finally a clean/isolated SynDB checkout;
5. cut products over using their own deployment controls and recorded rollback;
6. retire each legacy implementation only after its observation window.

A helper regression rolls back by restoring the previous flake lock. A UI
regression uses the product's rehearsed artifact/configuration/Nix generation
rollback. No Dioxus-only schema migration is allowed to cross that boundary.

## Program Definition Of Done

- The inventory has no unowned first-party Dioxus consumer.
- `rs-harbor` is the only owner of generic DX, offline Cargo/WASM,
  `wasm-bindgen` resolution, and web/fullstack output assembly.
- `tartan-ui` is target-explicit and contains only reusable presentation code.
- Plinth and SynDB are closed Dioxus cutovers with no active legacy web stack.
- Pink Raven serves only its Dioxus UI after its observation window.
- Bekiper serves Dioxus Web and its promoted Dioxus Native desktop app; Leptos,
  Trunk, and Iced production entry points are gone.
- Every adopted, reserved, experimental, and rejected Dioxus 0.7.9 feature has
  an explicit disposition; no release relies on accidental default features.
- All repositories pin compatible Dioxus, DX, Tartan, and rs-harbor revisions,
  pass their product gates, and retain durable upgrade/rollback guidance.
