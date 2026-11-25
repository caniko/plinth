# Changelog

## [Unreleased]

### Added

- Install sections take `platforms` (`linux`, `macos`, `windows`) on the section and on each route. A section that claims platforms fails validation unless every route names its platforms, every claimed platform has a route, and no route names an unclaimed one. Pages with tagged routes show the visitor's OS first, with a "Show all platforms" toggle.
- A `plinth-desktop` Dioxus 0.8 workspace module for writing and publishing posts to Plinth sites from the Fleetix Hub desktop app.
- Admin blog drafts: `GET /api/admin/drafts` lists unpublished API posts with their source text, and `PUT /api/admin/drafts/{id}` replaces one (edit, rename or publish it), keeping the author, featured and series fields the request omits and never overwriting a published post, a declarative draft or a newer copy.
- Broker sites restore a palette stored by a tartan-ui app (`tartan-palette` on `<html data-palette>`) before first paint, with the same script as tartan-ui's `THEME_INIT_SCRIPT`.
- Minimal look parity with the bekiper boards: segmented system/light/dark icon toggle, plain lowercase brand, bare showcase items, ruled workflow columns with zero-padded indexes, one shared container and a capped hero image (only with `[theme] broker_css`).
- `plinth-company`: modular company sites (mission, team, testimonials, partnerships, metrics, case studies, contact) on the project renderer, with local Inter, ambient motion that respects reduced motion and hidden tabs, `mkCompanySite`, and a `company-browser` Playwright gate.
- Per-page `scalable_text` for project pages that honor browser text-size preferences.
- Build, dev, publish and audit output cleanup refuses directories that contain the config, declared inputs or the posts directory, including through symlinks.
- Opt-in `[blog]` project-site posts: Markdown files rendered as an ordered index, post pages, RSS feed, sitemap/`llms.txt` entries, and a navigation link.
- Project-grid sections with descriptions and links drawn from the site's project references.
- Static-site sitemap, robots policy, `llms.txt`, project JSON index, and optional canonical-domain `CNAME` output.
- A check preventing retired forge hosts from returning to the Nix inputs.

### Changed

- Rust API (0.x breaking change for struct literals): install route models and config gained `note` and `link_label`; the install section model and `SectionConfig::Install` gained `guide_label`. Existing TOML remains compatible.
- Format generated portfolio Nix with Alejandra and normalize generator output in the drift check.
- Update Dioxus and its CLI contract to 0.7.10.
- Consolidate CI on GitHub Actions with aggregate workspace checks, Pages deployment, and trusted Attic publication.
- Use the repository's Rust toolchain file through Harbor and complete the Harbor lockfile input closure.
- Fetch Tartan UI from GitHub at the existing Dioxus-compatible revision.
- Share forge response handling, cache expiry, feed construction, and transactional tag helpers.
- Keep generated dependency, build, graph, and browser-test state out of source control.

### Fixed

- Let sites customize install-route notes and link labels, keep notes visible on mobile, and treat blank values as omitted.
- Vertically align navigation links with the primary button on pages without a split hero.
- Keep keyboard focus in the screenshot lightbox, return it to the opening thumbnail on close, and open images directly in browsers without modal support, using stills when reduced motion is requested.
- Return `409 Conflict` instead of a generic 500 when publishing a blog post whose slug already exists.
- Set the project site's canonical domain so deployed sitemap and robots metadata use absolute public URLs.
- Fetch the historical transitive Harbor pin over HTTPS so hosted runners can archive the complete flake without SSH credentials.
- Allow private, build-scoped compiler-cache fallback on unmanaged build hosts.
- Preserve the Pages artifact link and isolate generated workflow cancellation groups.
- Prevent early pipe termination from masking a retired-host match in the Nix input check.
