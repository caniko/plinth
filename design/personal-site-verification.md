# Personal site design run

Run `personal-site-2026-11-10`. Editable source of truth:
[`personal-site.fig`](./personal-site.fig) (OpenPencil). Machine-readable
contract and checkpoint: [`personal-site-run.json`](./personal-site-run.json).
Evidence (not tracked): `/data/nvme0/can/ProjectState/evidence/plinth-personal-site/`.

## Brief

Scope: design plus implementation. Surfaces: the Dioxus personal site served at
`can.tartanoglu.com` (`crates/dioxus-ui`, `input.css`, the cross-package build in
`flake.nix`) and the static placeholder at `dejana.tartanoglu.com`
(`canix: root/modules/server/public_ingress.nix`).

Users: readers arriving from a link to a post or project, mostly on a phone or
a laptop; the site owner checking that a published post shows up.

| # | Critical task | Baseline friction | Success check |
| --- | --- | --- | --- |
| T1 | Recognise whose site this is and what it covers | Home hero says "Plinth" and describes the platform; site name sits unstyled in the corner | Home shows the configured name, tagline and description above the fold at 1440 and 390 |
| T2 | Move between Posts, Projects, About | Nav links run together without spacing; nav ignores `site.nav`; mobile shows the desktop list and a hamburger whose menu lists different links | Configured nav renders once per viewport, current page is marked, mobile menu opens and closes and works before hydration |
| T3 | Scan recent posts and open one | No dates; three generic cards with clipped borders sit above the list | Post rows show title, description, date and reading time; the row links to the post |
| T4 | Read a post or the About page | Prose is readable but the shell around it is unstyled | Post renders with metadata, readable measure, tag links |
| T5 | Find the author elsewhere | Footer is only "Built with Plinth" | Footer lists configured social links and the feed |
| T6 | Land on the static page and know it is intentional | Browser-default serif at 6rem, no mark | Name, line and hostname in the shared sans stack with the plinth mark, light and dark |

Must stay available: every route, the theme toggle, feeds, tags, tech stacks,
reading time, author. Nothing is removed except the three home cards that
duplicated the navigation.

Visual character: amber on black by default (the product's existing dark
identity), white with blue links as the light theme, one 896px column, rules
instead of boxes for lists, boxes only for project cards.

## Root causes found at baseline

- F1 (P1) The Tartan stylesheet link renders as
  `href="/assets/This should be replaced by dx…"`. The deployed aarch64
  package is built with plain `cargo` plus `wasm-bindgen`
  (`crossPlinthArgs` in `flake.nix`), so `asset!("/assets/tartan-ui.css")` is
  never linked. Header, navigation and card grid are unstyled.
- F2 (P2) `dark:` utilities follow `prefers-color-scheme` while the
  `.dark .tartan-*` rules and the toggle use a class; the toggle starts as
  "dark" but nothing applies it.
- F3 (P2) Shell ignores `SiteConfig.nav`, `social`, `footer`, `tagline`.
- F4 (P2) Mobile shows both navigations; the menu needs WebAssembly to open.
- F5 (P2) Home hero is hardcoded platform copy.
- F6 (P3, data) Production lists a seeded "Sample Project". Not changed by
  this run; production data is out of scope.

## Frames

| Frame | Node | Size |
| --- | --- | --- |
| 01 Home · desktop · dark | `0:51` | 1440 × hug |
| 02 Home · mobile · dark | `0:149` | 390 × hug |
| 03 Mobile menu · open state | `0:204` | 390 × hug |
| 04 Posts · desktop · dark (with empty-state alternative) | `0:231` | 1440 × hug |
| 05 Home · desktop · light | `0:296` | 1440 × hug |
| 06 Dejana static · desktop · light | `0:343` | 1440 × 900 |
| 07 Dejana static · mobile · dark | `0:356` | 390 × 844 |

Node IDs are from the saved file; re-read them after future saves.

Tokens (dark / light): background `#000000` / `#FFFFFF`; card `#0A0A0A` /
`#FFFFFF`; rule amber-900 at 50% (`#3D1903`) / gray-200; heading amber-100 /
gray-900; body amber-200 / gray-700; lead amber-300 / gray-700; meta and
labels amber-500 / gray-600; links amber-300 / blue-600; chip border
amber-900, chip text amber-400. The static page keeps its own paper palette
(`#f6f4ef`, teal `#2f6f73`) and the fleet dark palette.

The design is drawn in Noto Sans. The site uses the system UI stack, so the
face differs per device.

## Journal

See `personal-site-run.json` for rounds, candidates and the next action.
