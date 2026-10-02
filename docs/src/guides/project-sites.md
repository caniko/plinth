# Project Sites

`plinth-project` builds static project landing sites from a
`plinth-project.toml` file. Each `[[pages]]` page renders to `{slug}/index.html`
(`index.html` for the home page), plus `style.css` and any `[[assets]]`.

```bash
plinth-project render --config website/plinth-project.toml --out public
```

## Site metadata

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `title` | string | required | Site title, shown in the nav bar and tab |
| `description` | string | required | Fallback `<meta name="description">` |
| `base_url` | string | `"/"` | Base URL used for absolute links |
| `canonical_domain` | string | — | Canonical domain (e.g. `"example.com"`). Used for absolute sitemap URLs and the GitHub Pages `CNAME` file |
| `footer_note` | string | `""` | Text displayed in the site footer |
| `primary_person` | string | — | ID of a `[[people]]` entry shown as the site author |

## Machine-readable output

Every build also emits companions for search engines and LLM traversal:

| File | Contents |
|------|----------|
| `sitemap.xml` | One `<loc>` per page; absolute when `canonical_domain` (or an absolute `base_url`) is set, path-only otherwise |
| `robots.txt` | `Allow: /`, plus a `Sitemap:` line when the origin is known |
| `llms.txt` | Site title/description, every page (title, URL, description), and every `[[projects]]` entry with its source/demo/links |
| `projects.json` | The `[[projects]]` table verbatim as JSON |
| `CNAME` | The canonical domain, only when `canonical_domain` is set |

Set `canonical_domain` for any site published to GitHub Pages: the `CNAME`
file flows through the Pages artifact automatically, so the custom domain
survives redeploys with no workflow changes.

## Guided landing composition

The renderer uses `tartan-ui-assets` for responsive grids, surfaces, wrapping
actions, control sizing, and focus treatment. `[theme]` maps Plinth colors onto
the shared semantic variables. Interactive Plinth pages consume the same pinned
`tartan-ui` revision through `tartan-ui-dioxus`.

A hero with `preview_src` uses a split desktop composition and stacks on narrow
screens. `preview_mobile_src` optionally supplies a narrow-screen illustration
through `<picture>`. `preview_alt` describes the illustration; `eyebrow` and
`note` provide supporting copy. Existing heroes without previews stay centered.

```toml
[theme]
preset = "gruvbox-hard-dark"
font_family = "Atkinson Hyperlegible Next"
font_url = "/assets/readable.woff2"

[[nav]]
label = "Enter workspace"
href = "/app"
primary = true

[[pages]]
slug = "index"
title = "Research library"

[[pages.sections]]
type = "hero"
eyebrow = "RESEARCH"
title = "Find references you can use"
tagline = "Search, compare, and keep source context attached."
subtitle = ""
preview_src = "/assets/preview.png"
preview_mobile_src = "/assets/preview-mobile.png"
preview_alt = "Illustrative search result with source context"

[[pages.sections]]
type = "feature_grid"
eyebrow = "START WITH A QUESTION"
heading = "What do you need to do?"
intro = "Choose a workbench."

[[pages.sections.features]]
title = "Find a reference"
description = "Search the authorized library."
action = { label = "Open search", href = "/search", primary = true }
```

Feature-grid headings are optional. With a section heading, card headings use
`h3`; otherwise they use `h2`. Workflow and trust sections also accept `eyebrow`.
On narrow guided landings, task-entry cards become compact linked rows while
retaining the action's accessible name. Navigation links marked `primary` remain
visible alongside the brand.

Font URLs refer to variable WOFF2 files (weights 100–900). Package fonts,
illustrations, and their licenses with `[[assets]]`; configure the host to serve
their output paths. Route destinations and domain copy belong to the consuming
project. The editable reference is `design/project-landing.fig`; its extraction
provenance is in `design/migration.json`.
