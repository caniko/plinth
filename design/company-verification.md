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

Rerun on 2026-11-11 on the branch rebuilt from the parked work on top of the
`[blog]` brick (local, Atlas):

- `cargo test --locked -p plinth-project`: 111 tests (64 library, 13 + 13
  binary, 6 blog, 15 company).
- Company package feature profile: 90 tests with
  `--no-default-features --features brick-company,brick-hero,brick-feature-grid,brick-content,brick-workflow-steps,brick-trust-panel,brick-screenshot-grid,brick-blog`
  (one trunk test needed a qualified `WorkflowSteps` path to compile here).
- No default features: 49 tests.
- `cargo clippy --locked -p plinth-project --all-targets -- --deny warnings`,
  also with no default features, with `brick-company` alone, with the
  `brick-company,brick-hero` profile, with the package profile, and with
  `brick-blog` alone.
- `nix develop .#company-tests -c bash scripts/test-company.sh`: 27 passed
  (26 desktop/mobile checks, including WCAG AA scans, real navigation, all
  three motifs, reduced motion, pause, local portrait/logo/custom-font
  readiness, disabled motion, gallery keyboard interaction, no-JavaScript
  fallback, stable geometry and the 15 KiB enhancement budget, plus the headed
  `visibility` background-tab check).
- `mdbook build docs`, `treefmt --fail-on-change` and `git diff --check`.

Evidence from this run (local captures, design exports from the saved
document, the Playwright log and `manifest.sha256`) is archived at
`ProjectState/evidence/plinth-company/20261111/`; the browser test writes the
captures to `target/company-evidence/local/`.

Earlier runs also recorded zero layout shift and no long tasks over
one-second motion samples (mobile at 4x CPU throttling); the current suite
keeps those assertions.

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

Simit generates the `company-browser` required gate (Xvfb, `devShells.company-tests`)
with evidence uploads; CI is generated by a Simit revision that pins that
upload like every other one (caniko/simit#151). Pull-request CI runs
`nix flake check`, which builds `plinth-company` and the asset-bearing
`plinth-company-site` fixture through `mkCompanySite`; those Nix outputs were
not built locally for this run.

The visual-rubric verdicts below are from 2026-10-12 and were not rerun.
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
