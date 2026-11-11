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
| `stylesheets` | string array | `[]` | Extra stylesheet URLs, linked after the built-in `/style.css` so site CSS wins the cascade |
| `favicon` | string | — | Favicon URL (`<link rel="icon">`); `type="image/svg+xml"` is added for `.svg` paths |
| `og_image` | string | — | Social preview image. Also emits Open Graph (`og:title`, `og:description`, `og:type`, `og:image`, and `og:url` with `canonical_domain`) and `twitter:card` meta. A path becomes an absolute `https://<canonical_domain>` URL when `canonical_domain` is set |
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

## Blog posts

Add a `[blog]` table to publish Markdown posts from a directory beside the
config file:

```toml
[blog]
title = "Notes"      # optional, defaults to "Blog"
posts_dir = "posts"  # optional, relative to plinth-project.toml
```

Posts are `<posts_dir>/<slug>.md`, where the file stem is the URL slug
(`^[a-z0-9]+(-[a-z0-9]+)*$`) and the file starts with YAML frontmatter:

```markdown
---
title: "My post"           # required
description: "One-liner"
date: "2026-11-10"          # required, YYYY-MM-DD
tags: ["rust", "release"]
author: "Name"
published: true             # false keeps the post out of every output
---
Markdown body…
```

`title` and `date` are required; `description` and `tags` default to empty,
and `author` is optional. A build writes `blog/index.html` (newest first),
`blog/<slug>/index.html` inside the site chrome, and `blog/feed.xml` when
`canonical_domain` is set. Blog pages join `sitemap.xml` and `llms.txt`, and a
"Blog" navigation link appears once a published post exists. Invalid
frontmatter, a missing `title`/`date`, an invalid date or slug, or a page slug
under `blog/` all fail `check` and `build`. Sites without `[blog]` render
exactly as before.

## Guided landing composition

The renderer uses `tartan-ui-assets` for responsive grids, surfaces, wrapping
actions, control sizing, and focus treatment. `[theme]` maps Plinth colors onto
the shared semantic variables. Interactive Plinth pages consume the same pinned
`tartan-ui` revision through `tartan-ui-dioxus`.

A hero with `preview_src` uses a split desktop composition and stacks on narrow
screens. `preview_mobile_src` optionally supplies a narrow-screen illustration
through `<picture>`. `preview_still` adds a still image served under `prefers-reduced-motion` (only meaningful with `preview_src`). `preview_alt` describes the illustration; `eyebrow` and
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

## Broker theme, theme toggle and minimal look

`[theme] broker_css` points (relative to the config file) at a
[nix-theme-broker](https://github.com/caniko/nix-theme-broker) web stylesheet,
the `theme.css` of `packages.<system>.web-theme-blackbox`. It emits
`--tb-<group>-<role>` custom properties: light on `:root`, dark through
`@media (prefers-color-scheme: dark)` (skipped when `data-theme="light"`) and
`[data-theme=dark|light]`.

```toml
[theme]
broker_css = "broker-theme.css"
# Optional: used as the fallback if the broker stylesheet is missing a role.
accent = "#076678"
```

When `broker_css` is set the generator:

- copies the file to `/broker-theme.css` and links it before `/style.css`;
- aliases the palette to the broker: `--pp-paper` to `--tb-ui-background`,
  `--pp-surface` to `--tb-ui-surface`, `--pp-ink` / `--pp-ink-soft` to
  `--tb-ui-foreground` / `--tb-ui-foreground-muted`, `--pp-line` to
  `--tb-ui-border`, `--pp-accent` to `--tb-ui-accent`, `--pp-secondary` to
  `--tb-status-success`, `--pp-warning` / `--pp-rust` to `--tb-status-warning` /
  `--tb-status-error`, and derives `--pp-accent-soft` from the accent. The
  colours (or `preset`) configured in `[theme]` stay as `var()` fallbacks;
- adds a system / light / dark toggle to the nav and applies the minimal look
  below (`<body class="pp-minimal">`).

Sites without `broker_css` render exactly as before: no extra file, link,
script, class or rule changes.

The toggle is a segmented control of three icon buttons (system, light, dark;
inline Lucide SVGs, ISC licence) in a `role="group"` labelled "Theme", each with
an `aria-label` and `aria-pressed` for the active mode. It sets `data-theme` on
`<html>` (system removes it) and stores the choice in `localStorage` under
`tartan-theme`, the key tartan-ui uses. A three-line head script restores a
stored light/dark choice before first paint. It is the only JavaScript the theme
adds, has no dependencies, and the control stays hidden without JavaScript.

The minimal look uses 1px borders, one 6px radius, no shadows or gradients, a
4px spacing scale, and the accent only for primary actions (`btn-primary`, the
primary nav link) and text selection. It also introduces a `display` type
size, used only by the `hero` H1: 40px, or 32px at 480px and narrower.

Layout under the minimal look, all inside the `minimal` cascade layer:

- the brand is plain lowercase text without the dot;
- nav, footer and every section share one container (`--pc-width`, 1200px) and
  sections are separated by full-width 1px rules; site CSS that sets its own
  `max-width`/`padding` on a section overrides this (unlayered rules win);
- the split hero is about 45% text and 55% media, the media capped at
  `min(60vh, 33rem)` with `object-fit: contain`, the note under the media;
  it stacks at 700px and narrower;
- `screenshot_grid` items are a bare image plus caption in three columns, one
  column at 480px and narrower;
- `workflow_steps` are columns with a 1px top rule and a zero-padded index
  (`01`, `02`, ...) above the title, stacked at 700px and narrower.

With Nix, pass the broker stylesheet to `project-site.nix`; it is copied to
`website/broker-theme.css` next to the config, and the build fails when the
config does not set `broker_css = "broker-theme.css"`:

```nix
plinth.lib.${system}.mkProjectSite {
  pname = "example-site";
  domain = "example.com";
  brokerCss = nix-theme-broker.packages.${system}.web-theme-blackbox;
}
```

`brokerCss` accepts a CSS file or a directory containing `theme.css`, and is
also read from project definitions (`mkProjectDefinition { brokerCss = ...; }`),
where the site-check and visual-audit targets then expect `/broker-theme.css`.

## Scroll sequence

`screenshot_grid` and `workflow_steps` sections accept
`scroll_sequence = true` (default `false`). The section gets a
`data-scroll-sequence` attribute and each item (`.screenshot-slot`,
`.workflow-step`) fades in (opacity 0 to 1) and slides up (16px to 0) as it
enters the viewport; scrolling back reverses it.

```toml
[[pages.sections]]
type = "workflow_steps"
heading = "How it works"
intro = ""
scroll_sequence = true
```

It is pure CSS, with no JavaScript:
`@supports (animation-timeline: view())` applies `animation-timeline: view()`
with `animation-range: entry 0% cover 40%` and `animation-fill-mode: both`.
Browsers without scroll-driven animations, and visitors with
`prefers-reduced-motion: reduce`, get the static layout.

`plinth-project audit site` appends a rule to the audit copy of `style.css`
that pins the static layout, because full-page captures sit at scroll 0 and
would otherwise show below-the-fold items mid-fade. The shared visual-rubric
capture can emulate `prefers-color-scheme` (`light` / `dark`) but not
`prefers-reduced-motion`, so the reduced-motion layout is verified through
this pinned static capture rather than a separate media-emulated one.
