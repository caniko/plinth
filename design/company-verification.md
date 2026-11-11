# Company presentation verification

The company reference uses synthetic North & Co content. The renderer also
has a dark, multi-page Vector Works fixture, a minimal Ribbon fixture, and an
asset-backed Field Studio composition.
None of these fixtures has a production deployment or authentication boundary.

## Surface matrix

| Reference | Local route/state | Viewport | Evidence |
|-----------|-------------------|----------|----------|
| 01 · North desktop | `/` | 1440 × 900 | `design-desktop.png`, `local/north-desktop.png` |
| 02 · North mobile | `/` | 390 × 844 | `design-mobile.png`, `local/north-mobile.png` |
| 03 · Team desktop | `/#team` | 1440 × 900 | `design-team-desktop.png`, `local/team-desktop.png` |
| 04 · Team mobile | `/#team` | 390 × 844 | `design-team-mobile.png`, `local/team-mobile.png` |
| 05 · Quotes and partnerships | `/#testimonials` | 1440 × 900 | `design-testimonials-desktop.png`, `local/testimonials-desktop.png` |
| 06 · Reduced motion | `/`, reduced-motion preference | 390 × 844 | `design-reduced-mobile.png`, `local/reduced-mobile.png` |
| 07 · Expanded navigation component | `/`, menu open | Component 390 × 300 | `design-menu-mobile.png`, full viewport `local/menu-mobile.png` |
| 08 · Motion notes | Annotation board | — | Contour/Orbit/Ribbon, pause, reduced motion, visibility, keyboard behavior |
| 09 · Field composition desktop | `/#mission`, Field fixture on port 1125 | 1440 × 900 | `design-mixed-desktop.png`, `local/mixed-desktop.png` |
| 10 · Field composition mobile | `/#mission`, Field fixture on port 1125 | 390 × 844 | `design-mixed-mobile.png`, `local/mixed-mobile.png` |

Evidence paths are relative to `target/company-evidence/`. Additional local
captures cover mission, contact, partnership, Vector home, and `/people/`.
The manifest records seven route/viewport pairs; preference and component
states are recorded above because they are not distinct routes.

## OpenPencil-to-local comparison

Capability and reading structure match: both hero actions stay visible,
members do not require personal URLs or portraits, quotes are attributed,
and partner/customer/affiliation relationships are distinguished. Empty
modules and their navigation links are omitted. The mobile menu is usable
by keyboard and restores button focus on Escape.

The geometry gate binds the source hero heading to x/y/height with a 2px
tolerance at both primary viewports. It also checks team, quotation, and partner
grid gutters/widths and section reading order. Field's eyebrow-free mission is
bound to x=517.333/y=96/width=842.667 on desktop and x=24/y=72/width=342 on mobile;
mission statements and contact actions align with their headings. Shared prose
receives 72px desktop / 48px mobile padding, a divider, and no extra first-heading
margin. Responsive checks cover 320, 390, 768,
1024, and 1440px. Inter is served locally; optical sizing is fixed to retain
the reference's font metrics. The saved design has zero major/critical text
sibling-overlap and frame/text parent-overflow findings.

The screenshots are not claimed to be raster-identical. The company renderer
now uses the reference's plain initials portraits. Navigation spacing and text
tracking are fluid in the implementation. SVG antialiasing differs between the
editor and Chromium. The new composition frames specify shared layout and
typography; prose wrapping and card text metrics retain renderer-specific differences.
These finish differences do not hide missing actions or content.

## Passing implementation gates

- `cargo test --locked -p plinth-project`: 77 tests.
- Company package feature profile: 62 tests with
  `--no-default-features --features brick-company,brick-hero,brick-feature-grid,brick-content,brick-workflow-steps,brick-trust-panel,brick-screenshot-grid`.
- `cargo clippy --locked -p plinth-project --all-targets -- --deny warnings`.
- Clippy also passes with no default features, with `brick-company` alone,
  and with the minimal `brick-company,brick-hero` profile.
- The no-default-feature suite passes; `init` omits the hero when that brick
  is disabled. The native workspace suite passes with the legacy WASM client
  excluded: `cargo test --locked --workspace --exclude plinth-client -j 4`.
  Database integration tests use the development PostgreSQL instance and
  isolated test databases. In the tool harness, start, test, and stop PostgreSQL
  within one shell invocation so its process remains alive during the tests.
- `npm --prefix e2e run test:company`: 26 desktop/mobile checks, including
  WCAG AA scans, real navigation, all three motifs, reduced motion, pause,
  local portrait/logo/custom-font readiness, disabled motion, gallery keyboard
  interaction, no-JavaScript fallback, and stable geometry.
- The headed `visibility` project adds one real background-tab check. It verifies
  `document.hidden`, suspension of the actual animation clock, foreground resume,
  and preservation of an explicit pause. Its isolated raw CDP context avoids
  Playwright's focus-emulation override; visibility is not mocked.
- Motion sampling records zero layout shift and no long tasks across all three
  motifs over one-second samples after readiness, including
  mobile at 4× CPU throttling. The gzipped enhancement budget is
  capped at 15 KiB by the browser test.
- `mdbook build docs`.
- `treefmt --fail-on-change` and `git diff --check`.
- Simit generation checks pass with the patched local producer:
  `cargo run --locked --manifest-path ../simit/Cargo.toml -- init ci --platform github --ci-provider actions --runtime nix --check --diff`.
- The parity evidence manifest validates all seven route/viewport pairs.

The implementation repairs included desktop hero height, static/variable font
metrics, mobile actions and navigation, page headings without a hero, reserved
output collisions, script-safe organization metadata, and a metrics-grid
class collision. Empty-section links now follow their destination page in
navigation, hero actions, and footers; shared gallery/copy interactions remain
available when those project bricks are composed into a company page. Output
cleanup now protects authoring inputs, symlink entries, and links traversed
through other link targets. Audit checks report/capture destinations and their
relationships before rendering; rejected operations preserve existing output.
Mixed-brick and
generated member IDs share validation. Unsupported project-hero fields receive
explicit diagnostics. Disabled motion creates no CSS animation object.
OpenPencil replacement nodes were explicitly reparented and
old nodes removed after inspecting the saved tree and exported images.

## External gates

`nix flake check --no-build --no-write-lock-file` was again rejected by the
tool permission gate on 2026-10-12, so full
Nix evaluation/build validation remains pending. The new `plinth-company-site`
check builds the asset-bearing fixture through `mkCompanySite`, then verifies
HTML, executable, fonts, images, and domain. Simit's generated CI includes the
`company-browser` required gate, using `devShells.company-tests` and Xvfb.
The patched Simit producer now supports required-gate artifacts. The generated
browser job uploads captures, failure screenshots, and traces with `always()`
and 14-day retention. Its schema, round-trip generation, CI, and release-gate
tests pass (131 integration tests); Clippy passes. Upstream `rustfmt` and
`cargo fmt --all -- --check` invocations were permission-blocked; formatting
validation of that patch remains pending. The installed 0.19.0 binary
predates this field, so regeneration currently requires the patched sibling
checkout. Hosted upload execution and the producer's release remain pending.
Regeneration also incorporates that producer's concurrency, strict formatter,
prebuild-timeout, and Pages setup updates.

The installed visual-rubric lacks `codex-acp`. A local runner built with that
feature completed real `plinth-site-beauty` evaluations on 2026-10-12 using
`openai/gpt-6.1-sol`; the configured `xai/grok-4.6/high` model was unavailable.
North desktop and mobile both received `pass` with empty anomaly lists. Desktop
was assessed as having clear hierarchy, confident typography, balanced spacing,
readable contrast, and prominent actions; mobile was assessed as fitting cleanly
without clipping, overlap, or responsive defects. These verdicts cover the
captured viewports, not the entire site or every design frame. Raw verdicts are
saved as `local/rubric-desktop.json` and `local/rubric-mobile.json`.

```sh
cargo run --locked --manifest-path ../visual-rubric/Cargo.toml --features codex-acp --bin visual-rubric -- configured --image target/company-evidence/local/north-desktop.png --preset plinth-site-beauty --name company-desktop --model openai/gpt-6.1-sol --json
# Repeat with north-mobile.png and --name company-mobile.
```

Deployed-to-local parity is **not-deployed**; no company production URL was
supplied. The local screenshots are fixture evidence.

## Regeneration

```sh
python3 scripts/export-company-design.py
npm --prefix e2e run test:company
python3 /path/to/openpencil-ui-parity/scripts/check_parity_manifest.py design/company-parity.json
```

The exporter reads IDs from the saved document, uses strict font resolution,
and never mutates the design. OpenPencil MCP owns design edits.
