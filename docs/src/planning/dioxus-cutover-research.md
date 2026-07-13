# Dioxus Cutover Research Dossier

Date: 2026-07-13

## Goal And Trigger

Design a complete cutover from Leptos 0.8 to the pinned stable Dioxus 0.7.9
line without changing Plinth's content model, public HTTP contracts, database
schema, feature-gated brick model, deployment identity, or user-visible route
behavior.

This is a continuation of commit `0ddf94f` (`Add Dioxus UI package`), not a
greenfield migration. That commit added `crates/dioxus-ui`, pinned Dioxus
0.7.9, enabled a web shell, and added the pinned `tartan-ui` dependency.

The implementation plan informed by this dossier is
[`dioxus-cutover-plan.md`](dioxus-cutover-plan.md).

## Current Reality

### Repository scope

The framework boundary is larger than `crates/client`:

- `crates/client` is the Leptos isomorphic UI, router, data-loading layer, and
  browser entry point.
- `crates/server` is both the reusable Axum/backend library and the Leptos
  production binary. `AppState` contains `LeptosOptions`; admin handlers call
  Leptos static-route invalidators directly.
- `crates/project` has a Leptos dependency solely to render one generator meta
  tag in its otherwise string-based static renderer.
- `Cargo.toml`, `flake.nix`, `modules/plinth.nix`, CI, scripts, docs, and the
  E2E harness all encode Leptos build/runtime assumptions.
- `public/htmx.min.js` is included by the Leptos shell, but no source file uses
  an `hx-*` attribute. It is dead runtime weight and should not cross the
  boundary.

The cutover must therefore end with no production Leptos dependency anywhere
in the workspace, not merely a second frontend package.

### Route and rendering contract

`crates/client/src/app/routes.rs` and
`crates/client/src/app/mod.rs::ROUTE_RENDERING_MODES` define 15 public page
routes in the default all-bricks build:

| Route | Current data | Current rendering contract |
|---|---|---|
| `/` | site content + blog + portfolio + activity | out-of-order streaming SSR |
| `/about` | site content | static/incremental |
| `/support` | site content + donation config | static/incremental |
| `/posts` | blog list | static/incremental |
| `/posts/:slug` | post + series navigation | static/incremental, enumerated slug |
| `/posts/tag/:tag` | tagged posts | static/incremental, enumerated tag |
| `/series` | series list | static/incremental |
| `/series/:slug` | series posts | static/incremental, enumerated slug |
| `/projects` | portfolio list | static/incremental |
| `/projects/:slug` | portfolio item | static/incremental, enumerated slug |
| `/activity` | ranked/refreshed activity | request-time SSR |
| `/activity/:id` | refreshed activity item | request-time SSR |
| `/todos` | mutable ranked todos | request-time SSR |
| `/todos/tag/:tag` | mutable tagged todos | request-time SSR |
| `/todos/:slug` | mutable todo item | request-time SSR |

The route table names `brick-blog`, `brick-portfolio`, `brick-todo`, and
`brick-activity`, but the current implementation gates the complete route set
on **all four at once**. Disabling any one brick falls back to only `/`,
`/about`, and `/support`; it does not independently expose the remaining brick
routes. That conflicts with the documented modular-brick intent and must be
frozen as a 16-combination truth table before the cutover deliberately fixes it.

### Browser behavior and document contract

The UI has a small but important interactive surface:

- responsive mobile navigation;
- persisted light/dark theme using local storage and system preference;
- a reduced-motion-aware animated canvas with resize and animation-frame
  lifecycle management;
- retry/reload error UI;
- client-side navigation and route parameters.

The document contract includes dynamic titles and descriptions, blog OpenGraph
and Twitter metadata, article JSON-LD, canonical URLs, favicon variants,
Plausible configuration, RSS links, and server-rendered trusted HTML produced
by the publishing pipeline. Late head rendering is not acceptable for indexed
content.

### HTTP and backend contract

`crates/server/src/router.rs` owns stable public, admin, feed, and sitemap
endpoints. The CLI calls the admin endpoints directly. These routes must remain
ordinary documented HTTP APIs; replacing them with framework-private RPC
encodings would be a breaking change.

The backend also owns:

- PostgreSQL migrations and queries;
- Kameo cache and vector-search actors;
- Immich proxying;
- RSS and sitemap generation;
- rate limiting, compression, security headers, body limits, and request
  tracing;
- graceful shutdown and OTLP observability.

Nothing in Dioxus replaces the database, caches, sessions, or application
services. Dioxus's own fullstack overview explicitly leaves those facilities to
third-party/application code.

### Existing Dioxus scaffold

`crates/dioxus-ui` is a 77-line client-only shell with four routes (`/`,
`/projects`, `/about`, catch-all) and placeholder empty states. It has no
Plinth data dependency, no fullstack/server feature, no backend integration,
no brick features, no Dioxus project configuration, and no Nix package.

`cargo check -p plinth-dioxus-ui` passes. That proves the shell compiles, not
that SSR or a production bundle works. Its direct `dioxus` dependency enables
`web`, `launch`, and `wasm-split`, while the `lib` surface is currently supplied
by Cargo feature unification through `tartan-ui-dioxus`. The production crate
must declare every Dioxus feature it relies on directly.

The pinned `tartan-ui` revision provides a generic application shell and a few
resource/dashboard components. It does not contain Plinth's blog, portfolio,
activity, todo, navigation, theme, rich-content, or metadata components. It is
a design-system seed, not a migration layer.

## Dioxus 0.7.9 Capability Evidence

Dioxus 0.7 is a viable target, with important fit boundaries:

- Fullstack supports SSR/hydration, type-safe server functions, custom Axum,
  typed routes, multipart forms, streams, SSE, WebSockets, asset management,
  WASM route splitting, and SSG. See the
  [official fullstack overview](https://dioxuslabs.com/learn/0.7/essentials/fullstack/).
- `dx` recognizes separate `web` and `server` Cargo features and can build
  separate client/server binaries, including separate workspace entry points.
  See [fullstack project setup](https://dioxuslabs.com/learn/0.7/essentials/fullstack/project_setup/).
- A custom Axum router can retain non-Dioxus routes and middleware while
  registering the Dioxus application. See the
  [Axum integration guide](https://dioxuslabs.com/learn/0.7/essentials/fullstack/axum/).
- `use_loader` is designed for isomorphic CSR/SSR data loading and propagates
  loading errors to suspense/error boundaries. Hydration requires identical
  server and client output. See the
  [SSR and hydration guide](https://dioxuslabs.com/learn/0.7/essentials/fullstack/ssr/).
- Out-of-order HTML streaming exists but is disabled by default, is configured
  on `ServeConfig`, requires JavaScript to reveal streamed boundaries, and
  cannot reliably add head elements or change status after the initial chunk
  is committed. See the
  [streaming guide](https://dioxuslabs.com/learn/0.7/essentials/fullstack/streaming/).
- Typed routing supports static, dynamic, catch-all, query, and hash segments,
  plus nests and layouts. See
  [route definitions](https://dioxuslabs.com/learn/0.7/essentials/router/routes/).
- Dioxus 0.7 can split and lazy-load WASM by route. In pinned 0.7.9 this is an
  experimental whole-leaf-router mode requiring both Dioxus/router features,
  the CLI split flag, and a suspense boundary; it is not a per-route-group
  annotation. See the
  [0.7 release notes](https://dioxuslabs.com/blog/release-070/).
- The asset pipeline hashes assets, prunes unused assets, and can optimize
  images, CSS, SCSS, and other files. See the
  [asset guide](https://dioxuslabs.com/learn/0.7/essentials/ui/assets/).
- Automatic Tailwind integration may download/start tooling, so the networkless
  Nix path must keep using the pinned standalone binary. See the
  [styling guide](https://dioxuslabs.com/learn/0.7/essentials/ui/styling/).
- Dioxus does not provide built-in authentication; Plinth retains its Axum
  API-key middleware. See the
  [authentication integration guide](https://dioxuslabs.com/learn/0.7/essentials/fullstack/authentication/).
- Dioxus supplies signals, contexts, effects, memos, async hooks, suspense,
  error boundaries, and stores with field/collection lenses. See
  [signals](https://dioxuslabs.com/learn/0.7/essentials/basics/signals/),
  [stores](https://dioxuslabs.com/learn/0.7/essentials/basics/collections/), and
  [error handling](https://dioxuslabs.com/learn/0.7/essentials/basics/error_handling/).
- Dioxus testing guidance covers SSR component rendering and Playwright E2E,
  but states that no complete hook-testing library is currently supplied. See
  [web testing](https://dioxuslabs.com/learn/0.7/guides/testing/web/).

### Important limitations for Plinth

1. **No Leptos-style island feature is present in the pinned Dioxus 0.7.9 Cargo
   feature surface.** The normal fullstack web path hydrates the application.
   Plinth must accept full hydration or build a custom island mechanism; the
   latter is not justified by the current small interactive surface. Route
   splitting and strict bundle budgets are the mitigation.
2. **Streaming is a server configuration, not a route annotation.** Plinth's
   mixed rendering policy requires a tested dispatcher between separately
   configured rendering states, or it must give up route-level behavior. The
   plan retains the behavior and makes the dispatcher an early proof gate.
3. **Dioxus SSG expects a build-time static route list and renders into the
   public output.** Plinth's dynamic slugs live in PostgreSQL and there is no
   deterministic content snapshot artifact available inside the Nix build.
   Default production must use runtime rendering plus a real runtime page
   cache; full SSG is deferred until a content-export producer exists.
4. **The 0.7.9 incremental renderer is not exposed through
   `FullstackState`.** Its renderer pool owns the cache privately. The public
   `IncrementalRenderer` type has `invalidate`, but that is not the instance
   used by `FullstackState`. The production design therefore does not pretend
   it has an invalidation adapter; it owns the outer response cache.
5. **Pinned-source expiry behavior needs a regression test.** In
   `dioxus-server-0.7.9/src/isrg/memory_cache.rs`, elapsed time is computed as
   `timestamp.signed_duration_since(now)`, so the configured time-based expiry
   path is not trustworthy without an upstream fix or proof. Two other source
   paths close the apparent workarounds: `memory_cache_limit(0)` bypasses the
   closure that reads the filesystem entirely, while filesystem
   `ValidCachedPath::freshness` returns `None` when `invalidate_after` is unset.
   The cutover therefore rejects the pinned built-in cache and puts a
   Plinth-owned filesystem completed-response cache in front of non-streaming
   Dioxus rendering. Its cold-process reuse and explicit invalidation are Phase
   1 gates, not assumptions.
6. **Full hydration changes the existing selective-hydration assertion.** This
   is an intentional framework trade, not a parity claim. The replacement gate
   is a measured initial WASM budget, lazy route chunks, no hydration errors,
   and fully rendered no-JavaScript content pages.

## Evidence Inventory

| Evidence | What it establishes |
|---|---|
| `git show 0ddf94f` | scope and intent of the existing Dioxus scaffold |
| `Cargo.toml`; every crate manifest | workspace/framework and feature topology |
| `crates/client/src/app/{mod.rs,routes.rs,invalidation.rs}` | route, render-mode, context, and invalidation contracts |
| `crates/client/src/{api,pages,components}/` | data calls, metadata, trusted HTML, and browser behavior |
| `crates/server/src/{lib.rs,setup.rs,router.rs,shell.rs}` | backend/UI coupling and Axum assembly |
| `crates/server/tests/rendering_modes.rs` | static caching, invalidation, streaming, islands, and dynamic freshness assertions |
| `crates/project/src/render/mod.rs` | second Leptos consumer and static generator behavior |
| `flake.nix`; `modules/plinth.nix`; `.forgejo/workflows/ci.yml` | build, package, runtime asset, service, and CI contracts |
| `e2e/tests/pages.spec.ts`; `scripts/test-home-streaming.sh` | current browser and streaming smoke coverage |
| `cargo tree -p plinth-dioxus-ui -e features` | effective current Dioxus feature set |
| pinned Cargo sources for `dioxus-{server,fullstack,router}-0.7.9` | exact feature and incremental-renderer behavior |
| `nix eval --raw nixpkgs#dioxus-cli.version` | nixpkgs supplies matching `dx` 0.7.9 |
| official Dioxus 0.7 documentation linked above | intended supported capability and constraints |

Discovery also found uncommitted `flake.nix` and `flake.lock` changes that are
unrelated to this plan. They must be preserved and rebased around, not
overwritten.

## Existing Plan Status

No existing Dioxus migration plan, phase directory, roadmap entry, or checklist
was found. The only prior migration artifact is the shell commit. No old plan
claims are carried forward.

## Work That Should Survive

The following are framework-neutral and should be moved or reused, not
rewritten:

- all shared domain types and serialization contracts;
- PostgreSQL schema, migrations, and content publishing format;
- Kameo actors and refresh behavior;
- public/admin/feed/image/search HTTP paths and response shapes;
- security middleware and observability policy;
- brick feature names and disabled-brick behavior;
- `plinth.toml` and NixOS module configuration semantics;
- page content, Tailwind theme, rich-content classes, logo/favicons, and
  accessibility intent;
- CLI behavior and deployed binary name `plinth-server`;
- current rendering-mode tests, rewritten against Dioxus where framework
  markers differ.

## Blockers And Missing Artifacts

The repository does **not** currently have a green Nix/WASM baseline, so no
cutover implementation should begin until the following preflight artifact is
repaired:

- **Invalid artifact:** `Cargo.lock` resolves `wasm-bindgen` 0.2.126 while the
  root `flake.nix` still packages `wasm-bindgen-cli` 0.2.125. The existing
  `wasm-bindgen-version-check` therefore fails. Exact reproducer:
  `nix build .#checks.x86_64-linux.wasm-bindgen-version-check --no-link
  --print-build-logs`.
- **Why it is foundational:** the browser bundle cannot be validated through
  the repository's own pinned Nix toolchain while its library and CLI versions
  disagree; accepting this failure would erase the baseline needed to
  distinguish migration regressions.
- **Upstream producer:** the root flake's WASM toolchain pin, maintained when
  the workspace lockfile changes. Update `wasmBindgenVersion` to `0.2.126`,
  then regenerate both `fetchCrate` and `fetchCargoVendor` SRI hashes from the
  expected hashes reported by successive Nix builds. Do not substitute a
  different lockfile or an unpinned host CLI.
- **Proof of repair:** rerun the focused command above, then
  `nix flake check`.

There is also a Dioxus-spike lint incompatibility to resolve deliberately:
`cargo clippy -p plinth-dioxus-ui --target wasm32-unknown-unknown -- -D
warnings` fails because Dioxus 0.7.9's `Routable` derive emits four generated
non-snake-case helper names. Phase 1 must prove the smallest scoped remedy
(derive/module-level allow, a confirmed upstream patch, or a version-aligned
fix) without globally weakening the workspace's deny-warnings policy.

One optional Dioxus capability is blocked by a genuinely missing foundational
artifact: database-backed SSG cannot enumerate or render dynamic content in a
pure Nix build because no canonical content snapshot/export exists. If full
offline SSG becomes a requirement, the upstream producer must be a new
`plinth export-site-data` workflow that emits versioned JSON for site content,
posts, tags, series, portfolio, activity, and todos. The future regeneration
command would be that exporter against a real Plinth database; validation would
deserialize the snapshot and compare its route manifest with `/sitemap.xml`.
Until that producer exists, SSG is not silently approximated with sample data.

## Risks And Constraints

- Framework replacement must not become a backend rewrite.
- Dioxus and `dx` must stay exactly version-aligned at 0.7.9 during the
  migration; 0.8 alpha is out of scope.
- Server-only dependencies must never enter the WASM target.
- Cached content routes must be invalidated only after successful admin writes;
  dynamic routes must never be cached by the page renderer.
- Streaming content metadata and 404 status must be committed before the first
  chunk.
- Trusted HTML remains an explicit publishing-pipeline boundary. Porting
  `inner_html` must not add a second sanitizer or weaken the existing contract.
- The Nix sandbox has no network. `dx` asset/build behavior must be proven in a
  sandbox before broad UI work.
- The existing package, service name, reverse-proxy behavior, database, and
  credentials remain rollback-compatible.

## Candidate Next Steps

1. Repair the documented Nix/WASM preflight blockers, then implement only the
   Phase 1 foundation spike: fullstack feature split, matching `dx`, custom
   Axum composition, SSR/hydration, Nix bundle, three-mode render dispatcher,
   and Plinth-owned completed-response cache.
2. Do not port page markup until the foundation spike passes native, WASM,
   Nix-sandbox, status-code, streaming, and invalidation tests.
3. Port vertical route slices against a frozen parity manifest, then switch the
   package on a staging host before production.

## Open Decisions For The User

The plan adopts these defaults unless explicitly changed:

- web/fullstack is the only production Dioxus platform;
- the stable public/admin HTTP API remains framework-neutral;
- full hydration is accepted with bundle/performance gates;
- Dioxus SSG is not on the production critical path;
- desktop, mobile, Native/Blitz, and LiveView are evaluated but not shipped;
- the final framework-owned crate is named `plinth-web`, while the production
  executable remains `plinth-server`.
