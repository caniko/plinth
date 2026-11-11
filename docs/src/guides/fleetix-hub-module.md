# Fleetix Hub Module

`plinth-desktop` is a Dioxus 0.8 component library that hosts embed as a
workspace for writing and publishing posts to Plinth sites. It renders only the
workspace body: the host keeps its sidebar, topbar and status bar, passes the
site list, and appends the module stylesheet.

The crate lives in `crates/desktop/` as its own Cargo workspace because the rest
of Plinth is pinned to Dioxus 0.7.10 and the root `Cargo.lock` is a Nix contract
input. The module has no renderer feature; its `desktop` feature exists only for
the preview example, which enables `dioxus/desktop`.

## `sites.toml`

```toml
[[site]]
id = "personal"
name = "tartanoglu.com"
kind = "personal"
api_url = "https://example.com"
api_key_file = "/run/agenix/plinth-api-key"
public_url = "https://tartanoglu.com"

[[site]]
id = "pink-raven"
name = "Pink Raven"
kind = "project"
root = "/path/to/repo/website"
base_url = "https://pinkraven.example"
```

- `personal`: posts go through the Plinth admin API. `api_key_file` is read at
  load time; the key never appears in the file or in `Debug` output.
- `project`: posts are Markdown files under `<root>/posts/`. `root` must contain
  `plinth-project.toml`.
- `public_url` (personal) and `base_url` (project) are optional: the address
  readers use. Post links, the slug hint and receipts are built from it; a
  personal site without `public_url` falls back to `api_url`.

`load_sites` returns `Result<Vec<Site>, ConfigError>`.

## Post files

Project posts use the fixed contract the `plinth-project` blog brick reads:

```markdown
---
title: "My post"
description: "One-line summary"
date: "2026-11-10"
tags: ["rust", "release"]
published: true
---
<body>
```

Strings are emitted as JSON-escaped double-quoted YAML scalars, tags as a flow
sequence, and `date` as local-today `YYYY-MM-DD`. Publishing writes
`<root>/<posts_dir>/<slug>.md` atomically and refuses to overwrite an existing
file (`SlugTaken`). `posts_dir` comes from the `[blog]` table in
`plinth-project.toml` (default `posts`); without that table the static build
renders no posts, so the module refuses to publish and says so.

## Hub integration

```toml
# Cargo.toml — the Dioxus pin must match the host's `=0.8.0-alpha.3`.
plinth-desktop = { git = "https://github.com/caniko/plinth.git", rev = "<commit>", default-features = false }
```

```rust
use plinth_desktop::{
    BlogWorkspace, STYLE, WORKSPACE_HINT, WORKSPACE_ICON_SVG, WORKSPACE_LABEL, load_sites,
};

// Load once at startup; keep the key files readable for the hub user.
let sites = load_sites(std::path::Path::new("/etc/fleetix/sites.toml"))
    .unwrap_or_default();

enum Workspace {
    // …
    Sites,
}

impl Workspace {
    fn label(self) -> &'static str {
        match self {
            Workspace::Sites => WORKSPACE_LABEL,
            // …
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Workspace::Sites => WORKSPACE_HINT,
            // …
        }
    }
}

// Point the nav at the module icon.
let icon = match workspace {
    Workspace::Sites => WORKSPACE_ICON_SVG,
    // …
};

// Dispatch renders the module in the host's main stage.
Workspace::Sites => rsx! {
    BlogWorkspace {
        sites: sites.clone(),
        drafts_dir: Some(drafts_dir.clone()),
    }
},
```

Append the stylesheet once, after the hub theme:

```rust
let css = format!("{}\n{STYLE}", include_str!("theme.css"));
rsx! { style { dangerous_inner_html: css } }
```

The module's CSS uses the hub tokens when present and falls back to the same
palette. Layout is chosen with container queries, so the host's sidebar and
inspector widths are honoured.

When `drafts_dir` is set, the in-progress draft is autosaved per site to
`<drafts_dir>/<site id>.json` about a second after each edit, restored when the
site is selected, and deleted after a successful publish.

Drafts already saved on a site are listed with its posts. Markdown drafts have
an Open action that loads them into the editor, keeping the editor's previous
content one Undo away; sending an opened draft replaces it, so it can be edited,
renamed or published. Replacing only works while the site still holds exactly
the draft that was opened (the server's draft id, or the file's text on a project
site); otherwise nothing is written and the draft must be opened again. A plain
new post never overwrites anything: a taken slug, draft or not, is refused.
Typst drafts are listed but edited with `plinth-cli`. On a project site the
draft is the `published: false` file; it is moved aside, checked, and only then
replaced, so a post published meanwhile is never touched.

## Personal API traffic

- `GET {api_url}/api/posts` — existing posts (published).
- `GET {api_url}/api/admin/drafts` — unpublished posts with their source
  (`id`, `slug`, `title`, `description`, `tags`, `content`, `content_format`,
  `published_at`). Servers without this route (404/405) are treated as having
  no drafts.
- `POST {api_url}/api/admin/articles` — `PublishArticleRequest` with
  `content_format = "markdown"`, creating a post. The receipt URL is
  `{api_url}/posts/{slug}`.
- `PUT {api_url}/api/admin/drafts/{id}` — the same body, replacing that draft
  in one transaction. Every replace gives the draft a new id, so a stale copy
  gets 410 Gone, which the module reports as `DraftChanged` ("Keep as a new
  post" then moves the text to a free slug). A 404 stays a rejection. Author, featured and series fields the module does not send keep
  the draft's values.

Admin calls send `Authorization: Bearer <key>`. 401/403 map to `Unauthorized`,
connection and timeout failures to `Unreachable`, `409 Conflict` (the slug
already exists) to `SlugTaken`, and other non-2xx responses to `Rejected`.

## Preview

```bash
cd crates/desktop
cargo run --features desktop --example preview
PREVIEW_SIZE=760x900 PREVIEW_FIXTURE=offline cargo run --features desktop --example preview
```

The example starts an in-process mock of the personal API and a throwaway
project site, then renders the module inside a minimal stand-in hub shell.
`PREVIEW_FIXTURE` accepts `empty` (no sites), `offline` (personal site on a
closed port), `badkey` (key rejected), `readonly` (project `posts/` not
writable), `fresh` (project site without posts) and `dense` (seven sites, 43
posts). `PREVIEW_SIZE` takes `WIDTHxHEIGHT` (default `1440x900`),
`PREVIEW_DELAY_MS` holds each publish, `PREVIEW_API_LATE_MS` starts the mock API
late, and `PREVIEW_DRAFTS_DIR` keeps drafts across launches.

The preview needs WebKitGTK/GTK, which the library targets do not. Build or run
it inside the hub devshell:

```bash
cd /path/to/infernix-hub
nix develop -c cargo run \
  --manifest-path /path/to/plinth/crates/desktop/Cargo.toml \
  --features desktop --example preview
```
