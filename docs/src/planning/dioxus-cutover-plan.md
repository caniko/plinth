# Full Dioxus Cutover Design And Plan

Date: 2026-07-13

Research basis: [`dioxus-cutover-research.md`](dioxus-cutover-research.md)

## Implementation Status

The delivery spine and production package are implemented: `plinth-web` owns
Dioxus SSR/WASM builds, typed routes and fullstack loaders, the explicit
cached/fresh/streaming policy, durable package-namespaced page cache, backend
invalidation events, direct-DB activity SSR reads, NixOS environment wiring,
and the CSR output. The Dioxus `server` feature graph no longer pulls Leptos;
the legacy client/server binary is explicitly isolated behind
`legacy-leptos` for rollback. `nix build .#plinth`, `plinth-project`, format,
and deny-warnings Clippy checks pass. The Nix workspace test currently has four
loopback WireMock checks that fail only inside the sandbox while the same
targeted tests pass outside it; this remains an explicit validation blocker
rather than a hidden skip.

## North Star

Ship the same Plinth product on Dioxus 0.7.9, then delete Leptos completely.
The completed cutover has one Dioxus web/fullstack application, one
framework-neutral Axum/backend library, unchanged database and HTTP contracts,
the same four optional bricks, reproducible Nix packages, and a reversible
NixOS package switch.

This is a framework cutover, not a redesign. Visual and behavioral changes are
accepted only when explicitly recorded and tested.

## Non-Negotiable Invariants

1. All 15 default routes, reduced-brick route sets, metadata, RSS, sitemap,
   images, public APIs, and admin APIs retain their externally visible contract.
2. The database schema and published content remain untouched by the cutover.
3. Static/publish-cadence pages do not query PostgreSQL on every request and are
   invalidated after successful writes.
4. Activity and todo pages remain request-fresh.
5. The home aggregate streams independent sections out of order.
6. Content pages return complete, indexable HTML without JavaScript. Titles,
   metadata, status codes, and structured data are in the initial response.
7. The WASM build contains no Axum, SQLx, Kameo, Tokio server runtime, ONNX, or
   forge client code.
8. `plinth-server`, `/api/*`, environment/config semantics, and NixOS service
   identity remain operationally stable until an explicit deprecation is made.
9. Every phase is independently reviewable and has an objective exit gate.
10. Leptos deletion happens only after the Dioxus package has passed staging;
    it is nevertheless part of the definition of done.

## Target Architecture

```text
Browser
  │
  ├─ initial request ──> plinth-web (Dioxus router + render policy)
  │                         │
  │                         ├─ Plinth PageCache ──> non-streaming FullstackState
  │                         │    about/support/blog/series/portfolio
  │                         │
  │                         ├─ fresh, non-streaming FullstackState
  │                         │    activity/todos/404
  │                         │
  │                         └─ streaming, uncached FullstackState
  │                              home only
  │
  ├─ SPA navigation ──> Dioxus use_loader ──> stable public JSON API
  │
  └─ assets ──> dx-bundled, hashed public assets and route WASM chunks

plinth-web (server build)
  └─ depends on plinth-server library
       ├─ AppState and bootstrap
       ├─ Axum public/admin/feed routers
       ├─ PostgreSQL + migrations
       ├─ Kameo actors and search
       ├─ security/limits/compression/tracing
       └─ PageInvalidator trait

plinth-web (WASM build)
  └─ depends on plinth-shared and browser-only transport
```

### Crate ownership

| Crate | Final responsibility | Required change |
|---|---|---|
| `plinth-web` | Dioxus app, typed routes, pages, components, loaders, browser code, production binary | rename/evolve `crates/dioxus-ui`; add `web`/`server`/brick features |
| `plinth-server` | backend-only library: state, bootstrap, routers, services, actors, middleware | remove Leptos/client dependency and binary-owned UI assembly |
| `plinth-shared` | cross-target domain/config/serialization contracts | retain; ensure every loader return type is WASM-safe |
| `plinth-cli` | stable admin API client and publishing | retain HTTP paths; update only documentation/branding fixtures |
| `plinth-project` | static project-site generator | replace the one Leptos meta-tag render and remove Leptos dependency |
| `tartan-ui-*` | reusable presentation primitives | use selectively; do not force product-specific behavior into it |
| `plinth-client` | none | delete after parity and staging cutover |

### Dependency direction

The final production binary lives in `plinth-web`. Its `server` feature enables
an optional dependency on `plinth-server`; its `web` feature enables browser
dependencies. `plinth-server` never depends on `plinth-web`.

This direction lets server-side Dioxus loaders consume `AppState` directly
without a circular dependency, while the WASM build calls the existing public
REST API.

### Cargo feature contract

The web crate declares all Dioxus features directly. Do not rely on feature
unification through `tartan-ui-dioxus`.

```toml
[features]
default = ["web", "brick-blog", "brick-portfolio", "brick-todo", "brick-activity"]
web = ["dioxus/web", "tartan-ui-dioxus/web", "dep:web-sys"]
server = ["dioxus/server", "tartan-ui-dioxus/server", "dep:plinth-server"]
devtools = ["dioxus/devtools", "tartan-ui-dioxus/devtools"]
release-wasm-split = ["dioxus/wasm-split", "dioxus-router/wasm-split"]
brick-blog = ["plinth-shared/brick-blog", "plinth-server?/brick-blog"]
# equivalent weak forwarding for the other bricks
```

The workspace `dioxus` dependency enables `lib`, `fullstack`, `router`,
`launch`, and `asset` explicitly. WASM splitting is a separate experimental
release profile with direct, pinned `dioxus-router` ownership and both
`dioxus/wasm-split` and `dioxus-router/wasm-split`. Do not make the normal
development graph depend on a disconnected split call graph.

### Axum composition and context

Move binary-local router and middleware modules into the `plinth-server`
library. Split current `async_main` into:

1. configuration loading and validation;
2. backend bootstrap returning `AppState` plus a shutdown/runtime guard;
3. framework-neutral public/admin/feed router construction;
4. top-level server launch owned by `plinth-web`.

The Dioxus `ServeConfig` context provider supplies a clone of `AppState` to SSR
loaders. Server functions, if later added, use ordinary Axum extractors and a
dedicated internal path namespace. Existing APIs are merged before the Dioxus
page fallback and retain all current middleware.

### Isomorphic data loading

Each page has one typed loader module with two implementations behind target
features:

- server: consume `AppState` and call the route's explicitly assigned backend
  data source, without loopback HTTP;
- web: call the stable public JSON endpoint with a relative or configured API
  base;
- both: return the same `plinth-shared` type and normalized application error.

Do not flatten the current freshness semantics into a generic "use the actor"
rule. Freeze this source matrix in Phase 0 and implement it route by route:

| Loader family | SSR source | Browser source | Freshness contract |
|---|---|---|---|
| site content | site-content service/cache | existing site-content API | explicit post-commit invalidation |
| blog, tags, series | existing blog query/cache boundary | existing article/tag/series APIs | explicit post-commit invalidation of every affected route |
| portfolio | existing portfolio query/cache boundary | existing portfolio API | explicit post-commit invalidation, including batch-sync slugs |
| todo | current direct/query boundary | existing todo API | request-fresh page; no page-render cache |
| activity list/item | **direct PostgreSQL query plus non-blocking `PokeRefresh`** | existing activity API/actor | request-fresh page while preserving stale-while-revalidate forge refresh |
| home aggregates | each slice's assigned source above | corresponding stable API | independent boundaries; no aggregate page cache |

The activity asymmetry is intentional. The current SSR path reads PostgreSQL
directly so a write is visible on the next request, then pokes the
`ActivityCache` to consider a forge refresh; the REST path reads through the
actor. Preserve it until a separately tested backend redesign replaces it.

Use `use_loader` for route data that must serialize from SSR to hydration.
Group related home data into independent loader boundaries to preserve
out-of-order streaming without data waterfalls. Browser-only state (theme,
canvas, storage) starts in `use_effect` after hydration.

Do not migrate stable public/admin endpoints into Dioxus server functions.
Server functions are reserved for future UI-private RPC where their generated
encoding is itself the contract.

### Rendering policy

Create a pure, exhaustively tested `RenderPolicy::for_path` function and three
Dioxus `FullstackState` instances over the same application/context factory:

| Policy | Routes | Dioxus configuration |
|---|---|---|
| `CachedContent` | `/about`, `/support`, all blog/series/portfolio routes | Plinth-owned page cache in front of non-streaming Dioxus rendering |
| `FreshContent` | all activity/todo routes and unmatched paths | non-streaming; never page cached; real 404 behavior after Phase 0 approval |
| `StreamingHome` | `/` | out-of-order streaming; never page cached |

The custom page handler dispatches the request to the selected state. This is a
Phase 1 proof, not an assumption: no page port begins until a fixture app proves
that all states hydrate with one client bundle, preserve status/headers, and
can coexist with the backend router.

### Static invalidation

Replace direct `plinth_client::invalidate_*` calls with a backend-owned trait:

```text
PageInvalidator
  invalidate_site_content(key)
  invalidate_blog(slug, tags, series)
  invalidate_portfolio(slug)
```

Every invalidation method returns `Result`, and `plinth-server` admin handlers
invoke it only after a successful transaction and actor-cache invalidation.
`plinth-web` supplies the Dioxus implementation; backend tests supply a
recorder/no-op. A post-commit deletion failure cannot roll back the database:
record a structured error/metric, enqueue bounded retry, and keep the response
semantics explicit. Operators must be able to identify and retry a failed path.

For updates/deletes, invalidate the union of old and new tag, series, slug, and
listing routes. Portfolio batch sync invalidates every successfully changed or
deleted slug plus the listing. Declarative site-content changes at startup use
the same invalidator. Tests cover request-vs-invalidation races and prove that
the mapper cannot escape its cache namespace with encoded/unicode segments,
`..`, collisions, or trailing-slash aliases.

Do not use Dioxus 0.7.9's built-in incremental renderer for production. Its
cache is privately owned by `FullstackState`; a zero-capacity memory cache also
prevents the filesystem lookup entirely, the no-TTL filesystem path never
returns a fresh render, and the memory-age calculation is reversed. These
pinned-source behaviors make explicit publish invalidation unverifiable.

Instead, implement a small Plinth-owned `PageCache` around complete,
non-streaming Dioxus responses. It atomically stores body, status, selected
headers, bundle fingerprint, and creation metadata in the writable cache
namespace; it owns lookup, single-flight rendering, deletion, and metrics.
`PageInvalidator` and request lookup share one safe route-key mapper. This is
not a second application renderer: a miss invokes the ordinary non-streaming
Dioxus state and stores its completed response. Phase 1 must prove cold-process
filesystem reuse with PostgreSQL stopped, then deletion followed by a fresh DB
render. A future fixed Dioxus cache can replace this only behind the same tests.

### Assets and styling

- Move the Tailwind scan source from `crates/client/src` to `crates/web/src`.
- Let `dx` own the browser JS/WASM bootstrap and content-hashed assets.
- Replace the Leptos-specific `/pkg/` cache rule and static-extension allowlist
  with assertions over the actual Dioxus manifest: hashed `/assets` and `/wasm`
  files are immutable, while stable/unhashed public URLs revalidate. Include
  `.js` and `.wasm` deliberately rather than falling through to HTML policy.
- Use `asset!`/`document::Stylesheet` for application-owned static assets only
  after confirming that externally stable paths (favicons, robots, image proxy)
  retain their current URLs.
- Retain Tailwind v4 and typography/rich-content classes.
- In Nix and CI, build Tailwind with the pinned `tailwindcss_4` binary before
  `dx`; prevent `dx`'s root-`tailwind.css` auto-detection from downloading or
  spawning its own tool. Phase 1 asserts a networkless bundle performs no tool
  acquisition. Developers may opt into the automatic watcher outside Nix.
- Remove `htmx.min.js` and its shell tag after confirming the zero-use search.
- Port favicons, alternate feeds, Plausible, color-scheme, CSP requirements,
  and pre-hydration theme script into the Dioxus index/document path.
- Treat `inner_html` as trusted pre-rendered publishing output and preserve the
  current boundary tests.

### Runtime and package layout

Keep `$out/bin/plinth-server` and `$out/site` for operational continuity. The
wrapper exports `DIOXUS_PUBLIC_PATH=$out/site`. The Plinth runtime page cache
lives in a separate state directory such as
`$STATE_DIRECTORY/render-cache`, configured through a Plinth-owned variable,
not mixed with immutable asset symlinks.

Namespace that render cache by an application/bundle fingerprint and garbage
collect retired namespaces only after their rollback window. Cached HTML from
build A must never reference assets absent from build B. A deploy/rollback gate
warms A, switches to B, and proves that every cached response references only
the selected build's asset manifest.

The current bootstrap deliberately uses Tokio's `LocalRuntime` and
`spawn_local`. The delivery-spine proof must either retain a compatible local
runtime around Dioxus serving or replace every local task with a proven
`Send`-safe owner before switching launchers. Include the declarative-content
embedding backfill in this test; a plain multithreaded `#[tokio::main]` is not
assumed equivalent.

Introduce `PLINTH_SITE_ADDR` and switch the NixOS module atomically. If a
compatibility variables and writable site-root preparation for the old binary
are retained through staging/observation, delete them only in the final cleanup
phase so the completed repository contains no framework branding.

## Dioxus Feature Disposition Matrix

Every documented 0.7 capability was considered. “Adopt” means it belongs in
the cutover; “reserve” means the architecture permits it but adding it is not
required for parity; “reject” means it is intentionally outside this web CMS
cutover.

| Dioxus capability | Disposition | Plinth use or reason |
|---|---|---|
| RSX, components, typed props, reconciliation | Adopt | replace all Leptos page/component views |
| Signals | Adopt narrowly | mobile menu, theme, local interactive state |
| Context/global state | Adopt | client-safe `SiteConfig`; server `AppState` through fullstack context |
| Effects and memos | Adopt | browser theme/canvas lifecycle and derived view state |
| Async futures/resources | Adopt through loaders | background/browser-only tasks only |
| `use_loader` | Adopt | primary isomorphic route data primitive |
| `use_server_future` / server-cached values | Reserve | only where loader semantics do not fit; avoid duplicate fetch primitives |
| Stores/lenses/reactive collections | Reserve | use only if a large mutable UI collection appears; current pages are read-mostly |
| Suspense boundaries | Adopt | independent home sections and route loading states |
| Error boundaries and typed render errors | Adopt | route/layout fallback, 404/5xx status propagation, safe user messages |
| Events and forms | Adopt where present | mobile menu/theme now; no admin UI forms to invent |
| Mounted data / web escape hatches | Adopt narrowly | canvas element/lifecycle where direct browser access is required |
| JavaScript `eval` | Reject | unnecessary and weakens CSP; use typed web APIs |
| Document title/meta/link/script APIs | Adopt | SEO, feeds, JSON-LD, Plausible, favicons |
| First-party asset pipeline and hashing | Adopt | UI CSS/images/WASM; preserve stable public URLs where externally referenced |
| Image optimization/conversion | Reserve | evaluate for bundled UI images; never rewrite Immich proxy semantics |
| SCSS | Reject | project uses Tailwind/CSS; no benefit from adding another pipeline |
| Scoped CSS/CSS modules | Reserve | useful for future isolated components, not a prerequisite to port existing Tailwind |
| Automatic Tailwind | Opt-in only outside Nix | convenient interactive watcher, but forbidden in Nix/CI because `dx` may download tooling; production CSS comes from pinned `tailwindcss_4` |
| Typed `Routable` enum | Adopt | one source of truth for paths and navigation |
| Static/dynamic/catch-all route segments | Adopt | current 15 routes and 404 |
| Query/hash route segments | Reserve | supported for future filters/anchors; do not invent URL changes |
| Nested routes/layouts/outlets | Adopt | shared shell and brick route grouping |
| Typed `#[redirect]` | Reserve narrowly | client-side aliases only; canonical HTTP redirects remain explicit Axum 3xx responses |
| `#[child]` child routers | Evaluate | possible brick modularity, but 0.7.9 supports only simple static child prefixes; flat compile-time gating remains acceptable |
| Typed links/navigation/history | Adopt | remove raw internal anchors where SPA navigation is desired |
| Hash-router mode | Reject | production has server routing and canonical clean URLs |
| WASM route splitting/lazy loading | Experimental opt-in | requires both Dioxus and router split features, split CLI mode, and a suspense boundary; retain only when measured bundle/latency improves |
| Fullstack client/server feature split | Adopt | required to keep server dependencies out of WASM |
| Custom Axum integration | Adopt | preserves backend routers and middleware |
| SSR and hydration | Adopt | default production delivery path |
| Hydrated CSR/SPA navigation after SSR | Adopt | router-driven navigation after the initial server response |
| Out-of-order HTML streaming | Adopt selectively | home aggregate only; other content remains complete without JavaScript |
| Dioxus incremental server cache | Reject for pinned 0.7.9 | opaque ownership plus confirmed memory/filesystem defects; Plinth owns a completed-response cache instead |
| Static site generation | Defer | needs a canonical database export/snapshot producer |
| Server functions | Reserve | future UI-private RPC only; stable HTTP API remains explicit Axum |
| Server-only Axum extractors | Reserve | available with future server functions |
| Fullstack error/status integration | Adopt | real 404/5xx responses before stream commit |
| Fullstack middleware annotations | Reserve | global policy remains explicit Tower/Axum; use only for RPC-local policy |
| Multipart forms / uploads / downloads | Reserve | existing CLI and image proxy contracts already work |
| Binary/text/JSON streams and SSE | Reserve | no current page requires them; feeds remain RSS HTTP responses |
| Typed WebSockets and `use_websocket` | Reserve | potential live admin/status UI; do not add a connection without a product need |
| Authentication integration guidance | Preserve existing integration | Dioxus has no built-in auth; Plinth's API-key/Axum middleware remains authoritative |
| WASM/serverless server target | Reject | Postgres, ONNX, Kameo, NixOS runtime, and local state require native server |
| Dioxus Primitives/accessibility components | Evaluate selectively | use for future complex menu/dialog behavior only when parity/a11y improves |
| Renderer accessibility integration | Adopt and test | semantic HTML/ARIA remain primary; verify the Dioxus event/DOM layer with axe and keyboard tests |
| `tartan-ui-dioxus` | Adopt selectively | shared tokens/shell primitives; product components remain in Plinth |
| Hot reload | Adopt | standard `dx serve` development loop |
| Subsecond hot patching | Opt-in development | useful for tip-level work; never a correctness or CI dependency |
| Devtools | Development-only | explicit non-production Cargo feature |
| Integrated debugger | Optional development | document for supported editors; not a build gate |
| Dioxus logger | Reject as global replacement | preserve existing tracing/OTLP setup; browser logging can integrate separately |
| `dx fmt` / HTML translation | Optional tooling | useful during port, never source of truth over `cargo fmt` |
| `dx bundle` | Adopt | production web/server asset assembly |
| First-party `llms.txt` documentation | Optional tooling | useful during migration research; not bundled into or trusted by the product |
| One-line CLI installer / `dx self-update` | Reject | Nix pins `dioxus-cli` 0.7.9 and owns upgrades reproducibly |
| CLI telemetry | Disable | Nix/CI/dev-shell policy should set telemetry off |
| Web renderer | Adopt | sole production renderer |
| Desktop WebView renderer | Reject for cutover | no desktop product requirement |
| Mobile WebView renderer | Reject for cutover | responsive web remains the target |
| Mobile manifest/plist, simulators, ADB, iPad, widgets, native FFI | Reject for cutover | useful only for mobile artifacts, which Plinth does not ship |
| Native/Blitz renderer | Reject | experimental and incompatible with required browser/SEO delivery |
| LiveView renderer | Reject | upstream deprioritized; persistent socket adds no value for content pages |
| Custom renderer | Reject | outside product scope |
| Internationalization utility | Reserve | site language config survives; no translation product requirement is added |
| SSR component tests | Adopt | fast markup/metadata/component parity tests |
| Playwright E2E | Adopt and expand | hydration, navigation, theme, accessibility, and route parity |

## Phased Execution Plan

### Preflight — Restore a trustworthy baseline

Do not start Phase 0 with the repository's current red Nix/WASM check.

1. Reconcile `Cargo.lock`'s `wasm-bindgen` 0.2.126 with
   `wasmBindgenVersion` in `flake.nix` (currently 0.2.125).
2. Regenerate the `fetchCrate` and `fetchCargoVendor` hashes using the expected
   hashes from successive Nix builds; do not replace the lockfile or use an
   unpinned host CLI.
3. Prove the focused check and the complete baseline:

   ```bash
   nix build .#checks.x86_64-linux.wasm-bindgen-version-check \
     --no-link --print-build-logs
   nix flake check
   ```

4. Record the current Dioxus 0.7.9 `Routable` derive lint failure from:

   ```bash
   cargo clippy -p plinth-dioxus-ui --target wasm32-unknown-unknown \
     -- -D warnings
   ```

   Resolve it with the narrowest verified derive/module-level allow or an
   upstream/version-aligned fix. The workspace-wide deny-warnings policy stays
   intact.

Exit gate: the pre-existing Leptos/Nix baseline is green and the Dioxus shell
passes native and WASM-target Clippy with warnings denied. The upstream
producer and exact repair/validation contract are recorded in the companion
research dossier.

### Phase 0 — Freeze the parity contract

Deliverables:

- machine-readable route manifest containing path shape, required brick,
  rendering policy, expected status, loader(s), head metadata, and invalidation
  triggers;
- per-loader server/browser source and freshness matrix, including activity's
  direct-query plus `PokeRefresh` asymmetry;
- golden HTTP fixtures for public/admin APIs, feeds, sitemap, image headers,
  security headers, and canonical redirects;
- reserved-prefix and method fixtures proving unknown `/api/**`, feed/static
  paths, `HEAD`, and `OPTIONS` retain their 404/405/header behavior and never
  fall through to Dioxus's page handler;
- representative database seed covering every route, empty states, missing
  records, all content formats, tags, series, featured data, and disabled bricks;
- a 16-combination brick route truth table. The current router is all-or-nothing
  (disabling any brick leaves only core routes), despite the documented
  independent-brick intent. The target is independently gated routes, recorded
  as an approved behavior correction before implementation;
- baseline browser screenshots and performance measurements for desktop/mobile,
  with JavaScript enabled and disabled;
- baseline release WASM/JS byte sizes and home streaming marker timings.
- a checked-in numeric budget file derived from those measurements: maximum
  compressed initial/chunk bytes, required split savings, cold/warm TTFB and
  p95 tolerances, visual-diff threshold/masks, and permitted accessibility
  severities. No later phase may use an undefined "agreed budget."

The current detail-route/catch-all implementation appears to render not-found
UI without changing the HTTP status. Phase 0 must measure this rather than call
it 404 parity. Returning real 404s is the desired target, but it is an explicit,
approved HTTP behavior correction with before/after fixtures.

Exit gate:

- current Leptos package passes the frozen suite;
- every one of the 15 routes and every public/admin/feed path has an owner;
- the 16 brick combinations and every numeric performance/visual/a11y
  threshold have an approved expected value;
- any known current failure is recorded rather than normalized into the Dioxus
  implementation.

### Phase 1 — Prove the Dioxus delivery spine

Deliverables:

- first extract the minimum router, middleware, configuration, and bootstrap
  surface from the current binary into `plinth-server`; Phase 1 cannot compose
  Dioxus against modules that remain private to `main.rs`;
- rename `plinth-dioxus-ui` to `plinth-web` and add correct `web`, `server`,
  devtools, and brick feature forwarding;
- add `Dioxus.toml`/index configuration and pinned `dioxus-cli` 0.7.9 to the Nix
  dev shell;
- compile a real SSR/hydration page with one loader and one browser effect;
- compose a custom Axum route, a Dioxus page fallback, and existing middleware;
- implement and test the three-state `RenderPolicy` dispatcher;
- implement the external Plinth completed-response `PageCache` and prove
  filesystem reuse plus explicit invalidation;
- build client and server through `dx` inside the Nix sandbox and package the
  result in the intended `$out/bin` + `$out/site` layout.

Required spike cases:

1. non-streaming route emits title/meta and a 404 status correctly;
2. the streaming home route emits title/meta before its first body boundary,
   emits a fast boundary before a delayed boundary, and resolves any
   status-producing loader before `commit_initial_chunk`;
3. one browser bundle hydrates pages rendered by all three states;
4. a cached route serves from a **fresh process/state** with PostgreSQL stopped
   after the first process warmed it;
5. invalidating its route file forces fresh content on the next request;
   the proof also covers concurrent miss single-flight behavior;
6. WASM dependency tree contains no server-only crate;
7. `DIOXUS_PUBLIC_PATH` works from the packaged wrapper and NixOS-style working
   directory;
8. the selected Tokio runtime starts Dioxus, actor tasks, declarative-content
   initialization, and embedding backfill without `spawn_local` panics;
9. build A's warmed render cache cannot serve HTML pointing at removed assets
   after switching to build B or rolling back;
10. `/assets`, `/wasm`, and any retained stable public paths receive correct
    hashed-immutable versus unhashed-revalidate cache headers.

Exit gate: all ten cases pass in a focused integration test and
`nix build`; otherwise stop and resolve the framework/build mismatch before
porting markup.

At minimum, the proof records these direct target builds (with the four brick
features appended individually and together):

```bash
cargo check -p plinth-web --target wasm32-unknown-unknown \
  --no-default-features --features web,brick-blog,brick-portfolio,brick-todo,brick-activity
cargo check -p plinth-web --no-default-features \
  --features server,brick-blog,brick-portfolio,brick-todo,brick-activity
dx bundle --package plinth-web --web --release
```

WASM splitting is a separate measured experiment, not the default command. It
adds a direct pinned `dioxus-router` dependency, enables both split features,
places a `SuspenseBoundary` above the router outlet, and runs:

```bash
dx bundle --package plinth-web --web --release \
  --features release-wasm-split --experimental-wasm-split
```

Dioxus 0.7.9 wraps every leaf route; it has no route-group split annotation.
The experiment must produce multiple chunks and pass direct-load, refresh, and
SPA-navigation tests. Normal `dx serve` remains on the connected graph.

### Phase 2 — Invert the backend ownership

Deliverables:

- complete the router/middleware extraction started by Phase 1 and remove its
  temporary compatibility shims;
- split bootstrap from serving and return `AppState` plus a deterministic
  shutdown guard;
- remove `LeptosOptions` from `AppState`;
- introduce backend-owned `PageInvalidator` with recorder/no-op tests;
- change successful admin handlers to call that trait instead of
  `plinth_client::invalidate_*`;
- make invalidation fallible, observable, and retryable after commit; invalidate
  old-plus-new blog relationships, every batch portfolio slug, and startup
  declarative-content changes;
- prove the route-to-cache mapper is namespace-safe and collision-free, and
  cover request/invalidation races;
- make `plinth-web` the production-binary owner under its `server` feature;
- preserve all current backend unit/integration tests without a UI dependency.

Exit gate:

- backend tests run without compiling `plinth-client`;
- API/feed/image/search behavior matches Phase 0 fixtures;
- actor startup/shutdown and OTLP teardown remain observable and orderly;
- the activity SSR loader remains direct-query plus `PokeRefresh`, and the
  dynamic freshness regression test still observes a direct database insert on
  the next request.

### Phase 3 — Port shell, document, and browser primitives

Deliverables:

- typed route enum with nested shared layout and compile-time brick gating;
- Dioxus header, footer, support CTA, error layout, not-found page, and
  client-safe site-config context;
- exact document shell: language, theme bootstrap, CSP-compatible scripts,
  Plausible, feeds, favicon set, viewport, color scheme;
- mobile menu and theme toggle using signals/effects;
- animated canvas port with reduced-motion, resize, cleanup, and no SSR access;
- Tailwind scan/source update and visual parity;
- a `SuspenseBoundary` above the outlet and an optional experimental
  all-leaf-route split profile, guarded by Phase 0's numeric size/latency
  thresholds; no nonexistent 0.7.9 route-group annotation.

Exit gate:

- shell SSR tests, hydration-error-free Playwright navigation, keyboard/mobile
  menu tests, theme persistence tests, reduced-motion canvas test, axe/a11y
  smoke, and visual diff all pass;
- no browser-only API runs during SSR.

### Phase 4 — Port core and publish-cadence vertical slices

Port complete vertical slices rather than copying all pages and wiring data
later:

1. `/about` and `/support` with site-content loader and invalidation;
2. `/posts`, `/posts/:slug`, `/posts/tag/:tag` with metadata, trusted HTML,
   tags, JSON-LD, related/series navigation, empty/missing/error states;
3. `/series` and `/series/:slug`;
4. `/projects` and `/projects/:slug`.

For each slice add:

- server-direct and browser-REST loader parity tests;
- SSR HTML and head assertions;
- cache hit/no-SQL assertion;
- successful publish/update/delete invalidation assertions covering old and
  new tags/series/slugs, portfolio batch sync (including partial success),
  listing pages, and an in-flight render racing a publish;
- 404 status and fallback assertion;
- CSR navigation assertion;
- enabled and disabled brick compilation/route assertion.

Exit gate: every cached route matches the Phase 0 fixture and remains complete
with JavaScript disabled.

### Phase 5 — Port request-fresh and aggregate slices

Deliverables:

- `/activity` and `/activity/:id`, preserving forge refresh behavior and
  ranking;
- `/todos`, `/todos/tag/:tag`, and `/todos/:slug`, preserving immediate
  freshness after writes;
- `/` with four independent suspense/loader boundaries and preserved ordering;
- dynamic error/empty/loading UI without caching;
- home metadata committed before streaming begins.

Exit gate:

- activity/todo tests see writes on the next request;
- the injected delayed-activity test proves intro/blog/portfolio arrive before
  activity;
- response-source/timing assertions prove home title/meta precede the first
  streamed body boundary and all status-determining work precedes the initial
  commit;
- a no-JavaScript request to non-streaming content routes remains complete;
- streaming limitations are documented for the home route rather than hidden.

### Phase 6 — Complete build, package, CI, and operations

Deliverables:

- replace `cargo-leptos` build phases with pinned `dx bundle` client/server
  builds while retaining Crane dependency caching where proven compatible;
- replace manual CSR wasm-bindgen assembly with a Dioxus web bundle and retain
  `.#plinth-csr` name/API-base semantics;
- build and install `plinth-cli` beside the Dioxus server in `.#plinth`; `dx`
  does not replace the CLI binary used by NixOS auto-publish hooks;
- update WASM optimization/version checks to match the actual `dx` output;
- package hashed assets and route chunks; add an asset-manifest assertion;
- assert that Tartan's `asset!`-registered CSS is present in that manifest and
  visibly applied in the packaged browser test;
- set `DIOXUS_PUBLIC_PATH`, the writable render cache path, and
  `PLINTH_SITE_ADDR` in the wrapper/NixOS module;
- through the observation window, keep the old derivation's
  `LEPTOS_SITE_ADDR`, writable `LEPTOS_SITE_ROOT`, and site-root preparation
  alongside the new variables so a full-generation rollback is executable;
- preserve hardening, credentials, Postgres ownership, auto-publish hooks,
  reverse-proxy behavior, and Attic publication;
- preserve every flake output and overlay contract (`plinth`, `plinth-dev`,
  `plinth-minimal`, `plinth-csr`, `plinth-aarch64-linux`, checks, apps, and
  rustdoc/docs outputs), including a proven aarch64 server + WASM build path;
- update CI, dev shell, scripts, README, AGENTS, mdBook docs, and environment
  variable reference;
- repair Forgejo/Woodpecker branch filters and release guards from `main` to the
  repository's actual production branch, `trunk`, then prove a real trunk CI
  run;
- add a reproducible Playwright CI job with PostgreSQL, the representative
  Phase 0 seed, the packaged server, and Chromium; `nix flake check` alone is
  not an E2E gate;
- update public project metadata and seeded fixtures that identify Leptos as
  the active frontend, while keeping historical post content unchanged;
- disable `dx` telemetry in reproducible/dev/CI environments.

Exit gate:

```bash
nix build .#plinth
nix build .#plinth-dev
nix build .#plinth-csr
nix build .#plinth-minimal
nix build .#plinth-aarch64-linux
nix build .#plinth-project .#plinth-person .#pcomfy
nix build .#docs .#site .#docs-full
nix flake check --keep-going --print-build-logs
nix eval .#overlays.default --apply 'overlay: builtins.isFunction overlay'
nix eval .#apps.x86_64-linux.default.program
```

`plinth-minimal` is the existing size-optimized package, not proof of reduced
brick combinations; the feature-matrix target checks from Phase 1 remain
separate required gates.

The packaged server must pass health, route, feed, header, invalidation,
streaming, and graceful-shutdown smoke tests from outside the source tree.

### Phase 7 — Migrate `plinth-project` and downstream presentation

Deliverables:

- replace the Leptos-rendered generator meta tag with deterministic escaped
  HTML or a small Dioxus SSR component only if Dioxus adds real value;
- remove the `leptos` dependency from `plinth-project`;
- preserve all generated HTML/CSS snapshots, project bricks, live reload,
  copy buttons, lightbox accessibility, and Codeberg Pages outputs;
- update generator metadata only if an intentional visible contract change is
  approved.

Exit gate: `plinth-project` tests, website build, Pages artifact diff, and
visual audit pass with no unexpected output change.

### Phase 8 — Staging, cutover, and rollback rehearsal

Deliverables:

- publish separate immutable `plinth-leptos` and `plinth-dioxus` derivations for
  the rehearsal window;
- deploy Dioxus to a staging domain/host using a production-like database copy;
- run parity, load, accessibility, no-JS, caching, invalidation, telemetry, and
  observability checks;
- rehearse a complete prior Nix service-generation rollback without database
  restore, including the old `LEPTOS_SITE_ROOT` and writable-site-root contract;
- switch production by changing only the Nix package/reference;
- monitor 5xx rate, latency, render-cache freshness, WASM/chunk failures,
  hydration errors, API errors, memory, and actor health.

Go/no-go gate:

- zero unresolved P0/P1 parity defects;
- all Phase 0 route/API fixtures pass;
- p95 server latency and first-content timings satisfy the numeric Phase 0
  budget;
- initial compressed JS+WASM is at or below the numeric Phase 0 budget and route chunks
  load reliably;
- rollback completes within the rehearsed window with the unchanged database.

### Phase 9 — Retire Leptos and close the migration

Delete:

- `crates/client`;
- old `crates/server/src/shell.rs` and Leptos route assembly;
- all `leptos*` workspace and crate dependencies/features;
- `[workspace.metadata.leptos]` and Leptos WASM profile/config;
- `cargo-leptos`, obsolete manual wasm-bindgen plumbing, `LEPTOS_*` variables,
  island markers/tests, and `scripts/test-home-streaming.sh` assumptions;
- `public/htmx.min.js` and its references;
- placeholder Dioxus migration messages and the temporary old package.

Update all durable documentation and then retire these planning documents once
their still-useful decisions have moved into architecture/rendering/testing
docs.

Final gate (hidden tracked files included):

```bash
git grep -n -i -E 'leptos|cargo-leptos|LEPTOS_' -- \
  ':!Cargo.lock' \
  ':!docs/src/planning/dioxus-cutover-*'
```

The command returns no production/config/documentation matches, `Cargo.lock`
contains no Leptos packages, all Nix checks pass, and the production site has
completed the observation window.

## Verification Matrix

### Functional and HTTP

- all routes: 200/404, direct load, refresh, SPA navigation, encoded params;
- all brick combinations: route presence/absence and compile success;
- public/admin APIs: method, path, status, JSON shape, auth, rate limit;
- feeds/sitemap/images: content type, cache headers, links, proxy behavior;
- declarative content and CLI publish/tag/todo/portfolio/activity flows.

### Rendering and SEO

- title, description, canonical, OpenGraph/Twitter, JSON-LD, feed links;
- full trusted HTML in response source;
- no-JS content pages;
- correct 404/5xx before stream commit;
- cached page survives DB outage and invalidates after successful write;
- dynamic page never returns stale page-render cache;
- home boundaries arrive out of order as designed.

### Browser and accessibility

- hydration produces no console errors/warnings;
- typed internal navigation and history work;
- mobile menu keyboard/focus/ARIA behavior;
- theme bootstrap has no flash and persists;
- reduced-motion and canvas cleanup;
- broken images, rich content, external link attributes, and focus styles;
- Chromium mobile/desktop visual regression and accessibility scan.

### Performance

- compressed initial JS/WASM and each lazy chunk recorded as CI artifacts;
- no server-only package in WASM `cargo tree`;
- cold/warm TTFB, first content, and full completion timings;
- cached/non-cached SQL query counts;
- memory and open connection stability under navigation/load.

### Operations and security

- Nix sandbox builds without network;
- immutable assets read from package, mutable render cache written only to state
  directory;
- CSP/security headers preserved and no `eval` dependency introduced;
- secrets absent from Nix store, logs, and browser bundle;
- OTLP/tracing, signal shutdown, and actor teardown verified;
- package rollback does not require database or content rollback.

## Rollback Design

Rollback is deliberately schema-neutral and generation-based:

1. Do not change the schema for the framework migration.
2. Keep the previous Leptos Nix derivation and module generation reachable
   through the observation window.
3. Keep API and configuration contracts compatible across both binaries.
4. Stop Dioxus and switch the complete NixOS generation (package, wrapper,
   environment, writable paths, and service definition), not only the package
   path; then start the old service.
5. Validate `/api/health`, `/`, `/posts`, `/projects`, and both feeds.

If a phase needs a schema or API change for an unrelated feature, split it into
a separate migration before or after this cutover so it does not poison the
rollback boundary.

## Definition Of Done

- Dioxus 0.7.9 owns every Plinth web page, SSR/hydration path, router, and
  browser asset.
- The backend, CLI, database, feeds, image proxy, and bricks retain their
  contracts.
- Static, dynamic, and streaming route policies have direct regression tests.
- Nix, NixOS, CI, staging, production, and rollback have all been exercised.
- `plinth-project` and the main application have no Leptos dependency.
- `crates/client`, cargo-leptos configuration, Leptos variables, island tests,
  and dead htmx asset are gone.
- Durable architecture/rendering/testing docs describe the new system and the
  temporary planning files are ready for retirement.
