# Phase 04 — Islands architecture (app-wide partial hydration)

> **Recommended Codex model: GPT 5.5 max**
>
> Frontier complexity in an app-wide-blast-radius role. `experimental-islands` is
> not a per-route attribute — it is a crate-wide compilation mode that inverts the
> default: `#[component]` trees become static, non-hydrating HTML, and only
> `#[island]` sub-trees ship and run WASM. Every existing interactive widget, every
> context provider (`SiteConfig`, the `Resource`-backed config), and the hydrate
> entrypoint must be re-reasoned under that inversion. The failure mode is
> insidious: the app compiles and renders correct-looking HTML, but the theme
> toggle silently never hydrates, or a context an island needs isn't available
> across the island boundary, and nothing errors. Mediocre work ships a page that
> looks done and is subtly broken. This earns `max` and the full pre-mortem below.
> Do not attempt it before Phases 02 and 03 have stabilized the route table.

## Working tree

`/data/nvme0/can/Projects/solo/plinth` — depends on Phase 01 (data path) and should
run **after** Phases 02 and 03 so the route taxonomy is stable. Edits
`crates/client/src/app.rs`, `crates/client/src/components/*`,
`crates/client/src/lib.rs`, the client `Cargo.toml` features, and the
`[[workspace.metadata.leptos]]` block — all shared with other phases. This is a
serialization point; land it last among the client-rendering phases.

## Goal

The app builds in Leptos islands mode: page bodies render as static HTML that ships
**no per-page WASM**, and only the two genuinely interactive widgets — `ThemeToggle`
and the `Header` mobile-menu toggle — are `#[island]`s that hydrate independently.
A content-only page such as `/about` hydrates zero islands; the toggle and mobile
menu still work everywhere after load. Islands compose with the `SsrMode::Static`
(Phase 02) and streaming (Phase 03) assignments — static/streamed HTML that contains
islands.

## Why this matters now

Today the whole `App` hydrates: every page ships and boots the full WASM bundle even
though almost all of Plinth is read-only display. The only client-reactive code is
`ThemeToggle` (`crates/client/src/components/theme_toggle.rs` — `signal` + `Effect` +
`on:click` + `localStorage`) and `Header`'s `menu_open` mobile toggle
(`crates/client/src/components/header.rs:9,54,71`). Islands mode is the
partial-hydration strategy that matches this reality: ship static HTML for the
content, hydrate only the toggle and the menu. This is the largest payload/perf win
available and the "Islands" mode the plan calls for — but it is also the one mode
that is an architectural commitment rather than an additive annotation, hence its
own phase and risk treatment.

## Out of scope

- Converting display-only components to islands "to be safe" — that defeats the
  entire purpose. Only `ThemeToggle` and the mobile menu become islands.
- Changing data fetching (Phase 01) or route modes (Phases 02/03). Islands changes
  *hydration granularity*, not how HTML is generated.
- The CSR build (Phase 05) — islands is an SSR-side concept; CSR is separate.
- Removing the inline `theme_script` in `shell()` (it sets the initial dark class
  before WASM loads — keep it; it prevents a flash before the toggle island boots).

## Plan

1. **Spike first, behind a branch.** Islands is `experimental-islands`; confirm the
   pinned Leptos 0.8 supports it and read its current rustdoc + the leptos
   `examples/islands*` for the exact `#[island]` macro semantics, context behavior
   across island boundaries, and the cargo-leptos `islands = true` requirement.
2. **Enable the feature.** Add `experimental-islands` to `leptos` for the client
   lib build (alongside `hydrate`) and set `islands = true` in the
   `[[workspace.metadata.leptos]]` block in `Cargo.toml`. Determine whether the
   server bin also needs the islands feature on its `leptos`/`leptos_axum` (it
   typically does for the matching render path) and wire `bin-features` accordingly.
3. **Convert `ThemeToggle` to an island.** Change `#[component]` →
   `#[island]` in `theme_toggle.rs`. Its signal/Effect/`on:click`/`localStorage`
   logic is exactly island-shaped (self-contained client state). Verify it still
   reads the initial theme set by the inline `theme_script`.
4. **Convert the mobile menu to an island.** `Header`'s `menu_open` toggle is the
   interactive part; the nav links are static. Either (a) make the whole `Header` an
   island, or (b) extract just the mobile-menu button + collapsible panel into a
   `MobileMenu` `#[island]` and keep `Header` a static `#[component]`. Prefer (b) —
   it keeps the static nav out of WASM. Note the islands constraint that islands
   take limited/serializable props and `children` support differs from components;
   design the split so the island owns its own state and receives only plain data
   (the `nav_items` list, `show_support` bool).
5. **Re-reason context across island boundaries.** `use_site_config()` reads
   `SiteConfig` from context. Under islands, context provided by a static parent may
   not automatically cross into an island — confirm how the island obtains
   `SiteConfig` (pass as a prop, or re-provide). The `App`'s `Resource`-based config
   provider (`app.rs:28-56`) needs review under islands: the hidden `<Suspense>`
   that re-provides config on the client assumes full hydration; verify it still
   functions or move config to props for the islands that need it.
6. **Adjust the hydrate entrypoint.** `lib.rs`'s `hydrate()` calls
   `leptos::mount::hydrate_body(App)`. Under islands, hydration is driven per-island
   by the framework; confirm whether `hydrate_body` is still the right entry or
   whether islands mode supplies its own bootstrap. Follow the leptos islands example.
7. **Confirm composition with Phases 02/03.** Build and load a static route
   (`/about` → zero islands), a static route containing the header island (`/posts`),
   and the streamed `/` — all must render and the toggle/menu must work on each.

## Acceptance criteria

- [ ] `cargo leptos build` with `islands = true` succeeds; `cargo clippy --workspace
      -- -D warnings` clean.
- [ ] `/about` (content-only) ships static HTML with **no page-body WASM**; browser
      devtools/network shows only the island runtime + the toggle/menu island
      payloads load, and only when those islands are present.
- [ ] The theme toggle changes theme and persists to `localStorage` after load on
      every page; the mobile menu opens/closes on a narrow viewport.
- [ ] No flash-of-wrong-theme on first paint (the inline `theme_script` still runs
      before islands hydrate).
- [ ] Client-side navigation and the streamed home page still work (islands do not
      break the router or Phase 03's streaming).
- [ ] WASM-safety check still passes: `cargo tree -p plinth-client --target
      wasm32-unknown-unknown` excludes `sqlx`/`pgvector`/`fastembed`/`plinth-forge`.

## Files likely touched

- `crates/client/src/components/theme_toggle.rs` (`#[component]` → `#[island]`).
- `crates/client/src/components/header.rs` (extract `MobileMenu` island; keep static
  nav).
- `crates/client/src/components/mod.rs` (export the new island if extracted).
- `crates/client/src/app.rs` (context-vs-props for `SiteConfig`; the config
  `Resource`/`Suspense` re-provide under islands).
- `crates/client/src/lib.rs` (`hydrate()` entrypoint under islands).
- `Cargo.toml` (client `experimental-islands` feature; `[[workspace.metadata.leptos]]`
  `islands = true`; possibly `bin-features`).

## Risk profile

- **Silent non-hydration.** The app compiles, the HTML looks right, but an island
  never boots (wrong macro, missing feature, context unavailable) — the toggle is
  dead and no error fires. Highest-likelihood failure.
- **Context starvation across the island boundary.** `SiteConfig` provided to a
  static parent doesn't reach an island; the island falls back to `Default` and
  renders wrong nav/labels.
- **App-wide regression.** Because islands inverts the default for *every*
  `#[component]`, a subtle misconfiguration can break hydration app-wide, not just
  on one page.
- **Interaction with streaming (Phase 03).** Islands + out-of-order streaming on `/`
  is the least-trodden combination; either can mask the other's breakage.
- **cargo-leptos / nix build drift.** `islands = true` changes the build outputs;
  the nix package (Phase 06/sub-01) must match or `nix flake check` fails late.

## Strategy

Commit ladder, each independently revertible:

1. **C1 — feature + metadata only** (`experimental-islands`, `islands = true`), no
   component changes. Build must still pass (all components still render; nothing
   hydrates yet because nothing is an island). Revert cost: trivial (one commit).
2. **C2 — `ThemeToggle` island.** Smallest interactive unit; prove one island
   hydrates end-to-end before touching the header. Revert cost: trivial.
3. **C3 — `MobileMenu` island + context/props fix.** The harder conversion (props,
   context). Revert cost: low.
4. **C4 — hydrate entrypoint + app.rs config reconciliation.** Revert cost: low but
   this is the one that can regress app-wide; keep it isolated so a revert is clean.

Do not squash these until all four are green — the boundary between C2 (works) and
C3/C4 (regresses) is the debugging signal.

## Rollback drill

Practice before C4 (the app-wide-risk commit). Time SLA: revert + green build in
< 5 min.

```
git log --oneline -4                 # identify C1..C4
git revert --no-edit <C4> [<C3> ...] # peel back to the last green island commit
cargo leptos build                   # must succeed
# load /about and /  — toggle + menu must work at whatever island level remains
```

If reverting C4 alone doesn't restore hydration, revert to C2 (single proven
island) and re-derive C3/C4 from the leptos islands example rather than patching.

## Failure modes and recoveries

- **F1 — toggle renders but does nothing after load.** *Symptom:* clicking the
  theme button has no effect; no console error. *Cause:* `ThemeToggle` still
  `#[component]` (static, not hydrated) or `islands = true` not set. *Recovery:*
  confirm the `#[island]` macro applied and `islands = true` in cargo-leptos
  metadata; check the network tab for the island's WASM chunk loading.
- **F2 — island shows default/empty nav.** *Symptom:* mobile menu has no links or
  wrong site name. *Cause:* `SiteConfig` context didn't cross the island boundary.
  *Recovery:* pass `nav_items`/`show_support`/`site_name` to the island as plain
  props instead of reading from context inside it.
- **F3 — app-wide blank/unhydrated after C4.** *Symptom:* nothing interactive
  anywhere; possibly a hydration-mismatch console warning. *Cause:* `hydrate()`
  entrypoint wrong for islands mode, or server/client islands feature mismatch.
  *Recovery:* align `bin-features`/`lib-features` islands flags; follow the leptos
  islands example's bootstrap; rollback drill to C2.
- **F4 — flash of wrong theme.** *Symptom:* page paints light then jumps dark.
  *Cause:* the inline `theme_script` in `shell()` was removed or the toggle island
  re-initializes theme on a slower path. *Recovery:* keep `theme_script`; ensure the
  island reads, not overwrites, the already-applied class on boot.
- **F5 — `nix flake check` fails only in CI.** *Symptom:* local `cargo leptos build`
  green, nix build red. *Cause:* the nix build flags don't include the islands
  change. *Recovery:* this is Phase 06/sub-01's job; flag the dependency so the nix
  sub-layer picks up `islands = true`.

## Reference

- Leptos islands docs + `examples/islands*` for the pinned 0.8 version (verify the
  `#[island]` macro, context, and bootstrap semantics — do not rely on memory).
- Interactive widgets being converted: `crates/client/src/components/theme_toggle.rs`,
  `crates/client/src/components/header.rs`.
- Hydrate entry: `crates/client/src/lib.rs`. Config provider: `crates/client/src/app.rs:28-56`.
- Prereqs/serialization: [01-ssr-data-path.md](./01-ssr-data-path.md),
  [02-ssg-static-routes.md](./02-ssg-static-routes.md),
  [03-streaming-home.md](./03-streaming-home.md) (shared `app.rs`); nix follow-up in
  [06-nix-tests-docs/sub-01-nix.md](./06-nix-tests-docs/sub-01-nix.md).
