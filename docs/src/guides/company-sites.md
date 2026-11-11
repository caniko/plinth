# Company Sites

`plinth-company` turns a TOML file into a modular company website: static HTML,
local fonts, CSS, a small progressive-enhancement script, and declared assets.
It uses the same renderer, development server, diagnostics, and publishing
workflow as `plinth-project`.

## Start a company site

```bash
plinth-company init --title "Your Company" --description "What you do, and why."
plinth-company check
plinth-company dev
plinth-company build
plinth-company preview
plinth-company publish --out dist/company
```

The command finds `website/plinth-company.toml`, then `plinth-company.toml` in
the current directory. Use `--config path/to/site.toml` to choose another file.
`init` refuses to overwrite an existing configuration unless given `--force`.
`dev` watches configuration and declared assets and reloads the browser.
`publish` produces a local bundle and `plinth-company-manifest.json`.
Build, development, publish, and audit output cleanup rejects directories that
contain the configuration or declared source inputs, including symlink aliases.
Keep generated output in a subdirectory such as `website/public` or `dist/company`.

Install from this repository with `nix build .#plinth-company`, or
`cargo install --path crates/project --bin plinth-company`. A small build needs
`brick-company,brick-hero`; additional existing project bricks can be enabled
individually. The Nix package includes feature grids, workflow steps, trust
panels, prose content, and screenshot galleries as well as the company sections.

## Composition and identity

The `[site]`, `[theme]`, `[[nav]]`, `[[footer_links]]`, `[[assets]]`, and
`[[pages]]` tables follow [Project Sites](./project-sites.md). Sections render
in configuration order; pages can contain any combination of enabled bricks.
Each company section has a unique `id`, a `heading`, and an optional `eyebrow`.
IDs share a page-wide namespace with existing project bricks and generated
member cards (`<team-id>-member-<member-id>`). `main-content` and
`company-navigation` are reserved. Raw prose HTML and custom-rendered markup
must also use distinct IDs.
Without an eyebrow, desktop headings keep their editorial column; on mobile,
headings and content use the full available width.
An empty team, quote, partnership, metrics, or case-study section is omitted.
Navigation and hero links to empty company sections are omitted too.

`[company]` selects the editorial company presentation. It provides an optional
`legal_name`, `location`, `logo_src`, `favicon_src`, and `social_image`
(defaulting to `[site] favicon` and `og_image`), plus `language` (default
`en`). The site title is the visible organization name.
Company copy is plain text; `\n` in headings creates deliberate line breaks.
Use the existing `content` brick for rich prose.

The default palette is warm paper, charcoal ink, and muted green. Override the
existing theme colors or choose a theme preset. Inter is bundled locally with
its SIL Open Font License. To use another font, provide both `font_family` and
`font_url` in `[theme]` and declare any local font file in `[[assets]]`.

```toml
[site]
title = "Your Company"
description = "Good work, with good people."
canonical_domain = "company.example"

[company]
legal_name = "Your Company Ltd"
location = "Your city · Working everywhere"
language = "en"

[company.motion]
motif = "contour"
duration_secs = 32
intensity = 0.3
enabled = true
```

Serve the generated directory at the site's origin root. Navigation, font,
script, and stylesheet URLs are root-relative. For multi-page sites, use
`/#mission` to link to a home-page section from any page, or `/people/` for a
separate team page. Set `canonical_domain` for canonical links, Organization
JSON-LD, social metadata, sitemap URLs, and the Pages `CNAME` file.

## Sections

| Type | Content |
|------|---------|
| `hero` | Existing hero fields: title, tagline, eyebrow, note, and CTAs; company presentation adds ambient artwork and location |
| `mission` | Required `statement`; optional `vision` and `values` with title/description |
| `team` | Ordered member IDs, optional `intro`; `layout = "full"` or `"compact"`, and compact sections accept `size = "small"` (default), `"medium"`, or `"large"`; optional `link` with `label` and `href`; an omitted/empty list selects all members |
| `testimonials` | `quotes` with quote/name and optional role, organization, source |
| `partnerships` | `partners` with name, relationship, optional logo_src, description, href |
| `metrics` | `items` with value, label, and required context |
| `case_studies` | `studies` with title, client, problem, approach, outcome; optional image_src/href |
| `contact` | Heading, optional intro, and a required label/href |

Members live in a top-level list and can appear on more than one page. Portraits
and personal URLs are optional; initials provide a deliberate fallback.

```toml
[[members]]
id = "maya"
name = "Maya Holm"
role = "Strategy & design"
bio = "Connects business intent to the experience people have."
# portrait_src = "/assets/maya.webp"
[[members.links]]
label = "Say hello"
href = "mailto:maya@example.com"

[[pages]]
slug = "index"
title = "Your Company"
[[pages.sections]]
type = "mission"
id = "mission"
heading = "Good work,\nwith good people."
statement = "We turn complex ideas into useful experiences."
[[pages.sections.values]]
title = "Built together"
description = "Honest conversations and shared ownership."
[[pages.sections]]
type = "team"
id = "team"
heading = "The people behind the work."
members = ["maya"]
[[pages.sections]]
type = "testimonials"
id = "testimonials"
heading = "In good company."
[[pages.sections.quotes]]
quote = "An example quote. Replace it with a real, attributed testimonial."
name = "Example client"
organization = "Example organization"
[[pages.sections]]
type = "partnerships"
id = "partners"
heading = "Better, together."
[[pages.sections.partners]]
name = "Example partner"
relationship = "partner"
description = "Explain this relationship."
[[pages.sections]]
type = "contact"
id = "contact"
heading = "Let’s talk."
label = "Get in touch ↗"
href = "mailto:hello@example.com"
```

Relationship values are `partner`, `customer`, `investor`, and `affiliation`.
They remain visibly distinct. Testimonials are semantic quotes with attribution;
optional source links lead to their original context. Metrics require context
so their values have a clear meaning.

### Team on the landing page, a team page, or both

Without `layout`, a team section renders the full cards: portraits or initials,
name, role, biography, and member links. `layout = "compact"` keeps
portraits or initials, name, and role, and omits biographies and member links.
Compact sections also accept `size = "small" | "medium" | "large"` (default
`small`). `small` is the shortest strip, with round avatars beside the name and
role; `medium` uses small square tiles; `large` uses large square tiles. `size` applies only to compact layouts and is
rejected on a full team. Either layout can carry a `link` to a dedicated team
page.

```toml
# Compact strip on the landing page, full cards on /people/.
[[pages.sections]]
type = "team"
id = "team"
eyebrow = "THE PEOPLE BEHIND THE WORK"
heading = "Different perspectives.\nA shared direction."
members = ["maya"]
layout = "compact"
size = "small"
[pages.sections.link]
label = "Meet the whole team →"
href = "/people/"

[[pages]]
slug = "people"
title = "Our people"
[[pages.sections]]
type = "team"
id = "people"
heading = "The people behind the work."
members = ["maya"]
```

Use only the first section, without its `[pages.sections.link]`, for a
landing-only team, or only the second for a team page. Use `layout = "full"` (or omit it) when the landing page should show
the complete cards. A page without a team section has no team content.

For an offering section, reuse `feature_grid`; for a working process, reuse
`workflow_steps`. Case studies connect the challenge, approach, and result.
Culture, history, careers, and FAQs can use the `content` brick.
Company heroes use ambient artwork. Project-hero preview images, per-hero logos,
and person bylines produce an explicit `company.hero.unsupported` diagnostic;
use the company logo, team, gallery, or content sections for that material.

## Motion

Choose one site-wide motif: `contour` (nested breathing contours), `orbit`
(slowly revolving ellipses), or `ribbon` (flowing parallel curves).
`duration_secs` accepts 8–120 seconds; `intensity` controls opacity, from 0 to
0.6. `enabled = false` preserves the static artwork and removes its control.

Motion uses CSS transforms on decorative SVG layers. Content never moves.
The script provides pause/resume, responds to reduced-motion preferences, and
suspends animation while the document is hidden. Without JavaScript the
artwork remains static and navigation stays available. On mobile the menu
has an expanded state, keyboard focus, and Escape-to-close behavior.

## Fixtures and verification

`website/company-examples/north.toml` is a synthetic two-person consultancy.
`vector.toml` is a dark product company with a separate people page, metrics,
and case studies. `ribbon.toml` is a minimal third-motion reference. `mixed.toml`
combines company modules with prose, feature cards, and a gallery, plus local
portrait/logo assets and a custom-font declaration. Their
people, quotations, relationships, and figures are illustrative.

```bash
plinth-company dev --config website/company-examples/north.toml
cargo test --locked -p plinth-project
cd e2e
npm ci
npm run test:company # includes a headed visibility check; needs a display
```

The browser suite verifies responsive geometry, accessibility, navigation,
asset readiness, gallery keyboard interaction, pause/resume, reduced motion,
disabled motion, real background-tab suspension, and all motion variants, and
writes settled captures under `target/company-evidence/local/`.
Set `CHROMIUM_PATH` to an installed Chromium executable, or use Playwright's
installed browser. For the complete CI environment, run
`nix develop .#company-tests -c bash scripts/test-company.sh`. This
uses Xvfb for the headed check when available. Simit's `company-browser` gate
runs the same script in GitHub Actions. The visibility test uses an isolated raw
CDP context because Playwright's ordinary page attachments emulate focus.
`plinth-company audit site` also shares the project-site
visual-rubric workflow.

Build and audit destinations are checked before cleanup or writes. Configs,
assets, and the symlink entries needed to reach them are protected. Audit render
and capture trees must be separate; report files must not overwrite authoring
inputs, rendered files, capture images, or other reports. The default report
inside the screenshot directory remains supported. Shared capture checks its
job, manifest, report, capture tree, and optional cache before rendering.

The GitHub browser gate retains `target/company-evidence/` and
`e2e/test-results/` as `company-browser-evidence` for 14 days, including failed
runs. Playwright keeps screenshots and traces for failures. The gate is
declared in `simit.toml`; regenerate and check the workflows with:

```bash
simit init ci --platform github --ci-provider actions --runtime nix
simit init ci --check
```

The editable reference is `design/company-site.fig`; `design/README.md` records
its artboards and local parity evidence. Company presentation is opt-in within the
shared `plinth-project` library: one engine owns serving, output, and CLI
behavior, while `brick-company` owns company contracts and presentation.

## Nix site output

The shared packaging helper exposes `mkCompanySite` alongside `mkProjectSite`.
It accepts the same domain, static asset, documentation, and footer parameters.
`checks.<system>.plinth-company-site` builds an asset-bearing example through
this helper and checks its packaged executable, HTML, fonts, images, and domain.

```nix
let
  companySites = import "${plinth}/nix/project-site.nix" {
    inherit pkgs;
    inherit (pkgs) lib;
    plinthCompany = plinth.packages.${pkgs.system}.plinth-company;
  };
in companySites.mkCompanySite {
  pname = "company-site";
  domain = "company.example";
  configPath = ./website/plinth-company.toml;
}
```
