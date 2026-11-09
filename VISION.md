# Plinth vision

Plinth is a self-hosted website platform in Rust: a personal site and blog
engine built on Dioxus 0.7 server-side rendering with WASM hydration,
PostgreSQL plus pgvector for storage and semantic search, and a NixOS module
for declarative deployment. It is used to run personal sites and blogs, and —
through the same renderer — to publish static project and company landing
sites from a small TOML file.

## Problem

Hosting a personal site usually means choosing between a static generator that
cannot search and a CMS with a database you cannot maintain. Plinth keeps the
good parts of both: SSR for fast first loads, a real database with tags,
vectors and RSS, authoring in Markdown or Typst through a CLI, and a NixOS
module so a self-hoster can deploy and harden it reproducibly. The second half
of the problem is portfolio sprawl: a person with a dozen repositories needs
consistent project pages without hand-writing one site per repository, so
`plinth-project` renders those from `plinth-project.toml`.

## Principles

- **Bricks over a monolith.** Content types are optional Cargo features
  (`brick-blog`, `brick-portfolio`, `brick-todo`, `brick-activity`), with DB,
  config, tags, site content, health check, image proxy, auth and middleware
  always present. A deployment carries only the content it uses.
- **Deployment is declarative.** The NixOS module owns systemd hardening and
  build reproducibility (the reverse proxy stays consumer-owned); there is no imperative
  install path.
- **Privacy-respecting defaults.** Analytics is Plausible and optional, with no
  tracking when unconfigured; images are served by the site's own Immich-backed
  proxy with explicit cache headers rather than third-party CDNs.
- **Machine-readable output is part of a site.** Every static build emits
  `sitemap.xml`, `robots.txt`, `llms.txt`, `projects.json`, plus `CNAME` when a
  canonical domain is configured, so search engines and LLM clients read the same site humans see.
- **Design references are editable artefacts.** Layouts live in OpenPencil
  `.fig` files under `design/` with recorded hashes and audits, not only as
  hand-written CSS.
- **Generated output is normalised and drift-checked.** Generator output is
  formatted and compared in CI instead of being committed unchecked.

## Scope

- In scope: blog, portfolio, todo and activity bricks; Markdown/Typst publishing
  through the CLI including image upload and embeddings; tag junctions, RSS
  feeds, semantic search over pgvector; Immich image proxying; admin, search,
  image and activity APIs; the `plinth-project` guided static renderer for
  project, portfolio and company sites (including scalable-text accessibility
  handling); the `plinth-person` identity/linking models; the `pcomfy` ComfyUI/
  Immich CLI; and the supporting Nix flake, NixOS module and mdBook.

## Non-goals

- Not a multi-tenant SaaS. It is self-hosted, one site per deployment.
- Not a general CMS. Content types are fixed bricks with typed CLIs, not
  user-defined schemas.

## Direction

The unreleased CHANGELOG and recent commits are about the static side of the
platform: project-grid sections with descriptions and links, sitemap/robots/
`llms.txt`/`projects.json`/`CNAME` output, a check that retired forge hosts
cannot return to the Nix inputs, Dioxus and CLI contract updates to 0.7.10,
consolidated GitHub Actions CI with Pages deployment and trusted Attic
publication, shared forge/cache/feed/tag helpers, and generated-output drift
checks. Design work is moving guided landings onto shared Tartan layouts with
labelled spatial evidence. The Leptos crates remain only as a rollback and test
surface scheduled for removal after the staging window.

## Place in the fleet

Canix consumes Plinth as a flake input, ships a `plinthAuthoring` Home Manager
program that wraps `plinth-cli`, and keeps the
`plinth-project-foss-sweep` and visual-review skills that onboard owned
repositories with a `website/plinth-project.toml`. Several owned repositories
include that manifest, and Canix's own `website/` personal and
public content is deployed through Plinth's site registry.
