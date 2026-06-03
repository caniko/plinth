# Phase 08 / Sub-03 — mdBook documentation for the activity feature

> **Recommended Codex model: GPT 5.5 low**
>
> This is mechanical documentation writing: three new/edited Markdown pages mirroring existing doc conventions, plus four lines added to `docs/src/SUMMARY.md`. The patterns (frontmatter tables, request-body tables, config-key tables) already exist verbatim in the repo and are copied with the activity field set substituted in. A low tier is sufficient and economical — there is no design, no code, and no failure mode worse than a broken SUMMARY link, which the acceptance check catches. Medium would be wasted budget.

## Working tree

`cwd = /data/nvme0/can/Projects/solo/plinth` (the plinth repo).

This sub-layer depends on Phases 01–07 having landed (it documents the shipped CLI subcommands, config sections, endpoints, and feed). It touches only `docs/src/**`. No sibling sub-layer touches docs, so there is no in-phase conflict. The mdBook builds via `nix build .#docs` (which runs `mdbook build docs`); local preview is `cd docs && mdbook serve`.

## Goal

This sub-layer succeeds when the documentation site has: a guide page for publishing and curating external contributions via the CLI (`plinth activity add/remove/update/list`), documentation of the `[ranking]` and `[forge]` config sections, the `GITHUB_TOKEN`/`CODEBERG_TOKEN` env vars, the `forge.refresh_ttl_secs`/`forge.refresh_backoff_secs` keys, and the `feeds.activity_limit` key, plus an API reference page covering the admin and public `/api/activity` endpoints and the `/feeds/activity.xml` feed — and every new page is wired into `docs/src/SUMMARY.md` so `mdbook build` emits no missing-file warnings and the pages are reachable from the table of contents.

## Why this matters now

The activity feature exposes a CLI surface, a config surface (a ranking strategy with tunable parameters, two secret tokens, a TTL), and an HTTP/feed surface that a site owner cannot use or operate without docs. The existing docs (`guides/publishing.md`, `configuration/plinth-toml.md`, `configuration/environment-vars.md`, `api/admin.md`, `api/search.md`) document the blog/portfolio equivalents; the activity feature must reach parity or it ships undocumented. Deferring means the owner has to read source to learn the CLI flags and the ranking knobs — and a reviewer has no spec to check the implementation against. This is the last user-facing gate before the feature is "done".

## Out of scope

- Editing any code, test, `Cargo.toml`, or `flake.nix` — docs only.
- Rewriting unrelated existing doc pages.
- Documenting internal implementation details of the refresh actor or ranking SQL beyond what an operator needs (the *strategies* and *params* are user-facing; the single-flight mechanism is an implementation note, mention briefly at most).
- The `planning/` directory itself (these phase docs are not part of the published nav; leave them as historical record).

## Plan

All new pages follow the existing doc style: a top-level `#` title, short intro paragraph, then fenced code blocks and Markdown tables with the columns the sibling pages use.

1. **New guide page: `docs/src/guides/activity.md`.** Mirror `docs/src/guides/publishing.md`'s structure (intro → workflow → CLI command tables). Document the four subcommands exactly as Phase 05 implemented them (the brief's CLI section):

   ````markdown
   # Curating External Activity

   The **activity** feature showcases pull requests and issues you have landed on *other people's* repositories across GitHub and Codeberg, ranked by impact × recency. It is distinct from the portfolio (your own projects).

   Each entry is fetched from the forge once, at add-time, by the CLI — which also computes the search embedding locally (the server never runs fastembed). The server then keeps the forge metadata fresh in the background (see [Configuration → Ranking & Forge](../configuration/plinth-toml.md)).

   ## Adding a contribution

   ```bash
   plinth activity add \
     --forge github \
     --repo owner/name \
     --pr 1234 \
     --impact 7 \
     --featured
   ```

   For an issue, use `--issue <n>` instead of `--pr <n>`.

   | Flag | Required | Description |
   |------|----------|-------------|
   | `--forge` | yes | `github` or `codeberg` |
   | `--repo` | yes | `owner/name` |
   | `--pr <n>` | one of | Pull-request number |
   | `--issue <n>` | one of | Issue number (mutually exclusive with `--pr`) |
   | `--impact <1-10>` | no | Curated impact score, SMALLINT 1..=10, default `1` |
   | `--featured` | no | Show in the home-page strip |

   On `add`, the CLI fetches the PR/issue metadata from the forge, embeds the title + body with fastembed (`AllMiniLML6V2`, 384-dim), and POSTs to `/api/admin/activity`.

   ## Updating impact or featured

   `<id>` is the numeric entry id (an `i64`) shown by `plinth activity list`.

   ```bash
   plinth activity update <id> --impact 9 --featured true
   ```

   ## Removing

   ```bash
   plinth activity remove <id>
   ```

   ## Listing

   ```bash
   plinth activity list
   ```

   ## Authentication & rate limits

   Public forge data works unauthenticated but is rate-limited (GitHub: 60 req/hour unauthenticated, 5000 authenticated; Codeberg: ~2000 req / 300 s, IP-based). Set `GITHUB_TOKEN` / `CODEBERG_TOKEN` to raise GitHub's limit and reduce throttling. See [Environment Variables](../configuration/environment-vars.md).
   ````

   (Match the exact flag names/spelling from Phase 05's `clap` derive: the subcommands are `add`, `remove <id>`, `update <id>`, and `list`. `remove` and `update` take a numeric `i64` id only — there is no id-or-url form. `update` maps to `PATCH /api/admin/activity/{id}`.)

2. **Config docs — edit `docs/src/configuration/plinth-toml.md`.** Append two new sections after the existing `[search]`/`[content]`/`[feeds]` tables, using the same `| Key | Type | Default | Description |` table format:

   ````markdown
   ## `[ranking]`

   Controls how activity entries are scored for the ranked `/activity` page, the home strip, and the public API. Score is computed at read time in SQL (no stored column), so changes take effect on the next request.

   | Key | Type | Default | Description |
   |-----|------|---------|-------------|
   | `strategy` | string | `"exponential"` | `"exponential"`, `"linear"`, or `"pure"` |
   | `half_life_days` | integer | `365` | exponential: score = `impact * 0.5 ^ (age_days / half_life_days)` |
   | `window_days` | integer | `730` | linear: score = `impact * max(0, 1 - age_days / window_days)` |

   `age` uses the reference date = `coalesce(merged_at, closed_at, created_at)`. `pure` uses `impact` alone, with the most-recent reference date as a tiebreaker. Results are ordered by score descending, then reference date descending.

   ## `[forge]`

   Controls how the server fetches and refreshes activity metadata from code forges. **Tokens are provided via environment variables only, never in this file** (see [Environment Variables](environment-vars.md)); there are no token keys here.

   | Key | Type | Default | Description |
   |-----|------|---------|-------------|
   | `refresh_ttl_secs` | integer | `3600` | An entry is refreshed in the background when its `fetched_at` is older than this many seconds |
   | `refresh_backoff_secs` | integer | `900` | Minimum seconds between refresh attempts for an entry that errored, to back off on rate limits |
   | `github_base_url` | string | `"https://api.github.com"` | GitHub API base URL (override for GitHub Enterprise / testing) |
   | `codeberg_base_url` | string | `"https://codeberg.org/api/v1"` | Codeberg/Forgejo API base URL (override for self-hosted Forgejo / testing) |

   ```toml
   [ranking]
   strategy = "exponential"
   half_life_days = 365
   window_days = 730

   [forge]
   refresh_ttl_secs = 3600
   refresh_backoff_secs = 900
   github_base_url = "https://api.github.com"
   codeberg_base_url = "https://codeberg.org/api/v1"
   # tokens via GITHUB_TOKEN / CODEBERG_TOKEN env vars, not here
   ```
   ````

   Also add the `feeds.activity_limit` key to the existing `[feeds]` table on this page (mirroring `blog_limit`/`projects_limit`): `| activity_limit | integer | 50 | Max entries in /feeds/activity.xml |`. (Match the actual default Phase 07 set.)

3. **Config docs — edit `docs/src/configuration/environment-vars.md`.** Add a new section (mirror the existing "Authentication"/"Database" tables):

   ````markdown
   ## Forge tokens

   Optional tokens for fetching activity from code forges. Public data works without them but is rate-limited. These tokens are read **only** from the environment — there is no TOML equivalent in `[forge]` — and they are never sent to the browser.

   | Variable | Description |
   |----------|-------------|
   | `GITHUB_TOKEN` | GitHub personal access token (raises rate limit to 5000 req/hour) |
   | `CODEBERG_TOKEN` | Codeberg/Forgejo access token |
   ````

   Document the refresh behaviour where it is operationally relevant — add a short "Activity refresh" note to the env-vars page: the server serves cached activity immediately and refreshes forge metadata in the background when an entry's `fetched_at` is older than `forge.refresh_ttl_secs` (default `3600`, i.e. 1 hour). Cross-link to the `[forge]` table on `plinth-toml.md`, which documents `refresh_ttl_secs` and `refresh_backoff_secs` (default `900`) as the canonical TOML keys — do not document the TTL as a fixed/hard-coded value.

4. **New API page: `docs/src/api/activity.md`.** Mirror `docs/src/api/admin.md` (request-body table) + `docs/src/api/search.md` (response JSON block + query-param table). Cover all endpoints from the brief:

   ````markdown
   # Activity API

   Endpoints for curated external contributions (PRs/issues you landed on other repos), ranked by impact × recency.

   ## Admin (Bearer auth)

   ```
   POST /api/admin/activity
   Content-Type: application/json
   Authorization: Bearer <your-api-key>
   ```

   Upserts by the natural key `(forge, repo_owner, repo_name, kind, number)`.

   **Request body** (`PublishActivityRequest`):

   | Field | Type | Required | Description |
   |-------|------|----------|-------------|
   | `forge` | string | yes | `"github"` or `"codeberg"` |
   | `repo_owner` | string | yes | Repository owner |
   | `repo_name` | string | yes | Repository name |
   | `kind` | string | yes | `"pr"` or `"issue"` |
   | `number` | integer | yes | PR/issue number (> 0) |
   | `impact` | integer | no | 1..=10 (default 1) |
   | `featured` | bool | no | Show in home strip (default false) |
   | `embedding` | float[] | no | 384-dim fastembed vector (supplied by the CLI) |

   ```
   DELETE /api/admin/activity/{id}
   PATCH  /api/admin/activity/{id}    # impact / featured / published
   ```

   ## Public

   ```
   GET /api/activity?limit=<n>&featured=<bool>
   ```

   Returns entries ranked by score (descending), then reference date. Reading a stale entry triggers a single-flighted background refresh; stale data is served immediately and the response never blocks on the refresh.

   ```
   GET /api/activity/{id}
   ```

   ## Feed

   ```
   GET /feeds/activity.xml
   ```

   RSS 2.0 (`application/rss+xml`, `Cache-Control: public, max-age=3600`). Each item links to the forge URL, ordered by ranking.

   ## Search

   Activity entries are unioned into the semantic search at `GET /api/search` (see [Search API](./search.md)); contributions surface alongside blog posts.
   ````

   (Match the exact `PublishActivityRequest` field names from Phase 01's `plinth-shared` type.)

5. **Wire all new pages into `docs/src/SUMMARY.md`.** The file currently has `# Guides` and `# API Reference` sections. Add the activity pages under the matching headings (do NOT add a `planning/forge-activity` entry — planning docs are not published nav):

   ```markdown
   # Guides

   - [Publishing Blog Posts](./guides/publishing.md)
   - [Image Handling](./guides/image-handling.md)
   - [Curating External Activity](./guides/activity.md)
   ```

   ```markdown
   # API Reference

   - [Admin API](./api/admin.md)
   - [Search API](./api/search.md)
   - [Image Proxy](./api/images.md)
   - [Activity API](./api/activity.md)
   ```

   The `[ranking]`/`[forge]`/env-var additions are edits to existing `configuration/*.md` pages already in SUMMARY — no new SUMMARY entry needed for those.

6. **Build and check links:**
   ```bash
   nix build .#docs 2>&1 | tail -20
   # or, for a faster local loop:
   cd docs && mdbook build && cd ..
   ```
   `mdbook build` warns about any SUMMARY entry pointing at a missing file, and any new page not referenced from SUMMARY. Resolve all warnings.

## Acceptance criteria

- [ ] `docs/src/guides/activity.md` exists and documents `plinth activity add` (with `--forge`, `--repo`, `--pr`/`--issue`, `--impact`, `--featured`), `remove`, `update`, and `list`.
- [ ] `docs/src/configuration/plinth-toml.md` contains a `## \`[ranking]\`` section documenting `strategy` (`exponential`/`linear`/`pure`), `half_life_days` (365), `window_days` (730); a `## \`[forge]\`` section documenting `refresh_ttl_secs` (3600), `refresh_backoff_secs` (900), `github_base_url`, and `codeberg_base_url` (and NO `github_user`/`codeberg_user` and NO token keys); and an `activity_limit` row under `[feeds]`.
- [ ] `docs/src/configuration/environment-vars.md` documents `GITHUB_TOKEN` and `CODEBERG_TOKEN` (env-only, never TOML keys), plus a note on the activity refresh TTL keyed to `forge.refresh_ttl_secs` (default `3600`).
- [ ] `docs/src/api/activity.md` exists and documents `POST /api/admin/activity` (request-body table), `DELETE`/`PATCH /api/admin/activity/{id}`, `GET /api/activity` (with `limit`/`featured` params), `GET /api/activity/{id}`, and `GET /feeds/activity.xml`.
- [ ] `docs/src/SUMMARY.md` lists `[Curating External Activity](./guides/activity.md)` under Guides and `[Activity API](./api/activity.md)` under API Reference.
- [ ] `nix build .#docs` (or `mdbook build docs`) exits 0 with no missing-file / unreferenced-page warnings for the new pages.
- [ ] No `planning/` page is added to SUMMARY by this sub-layer.

## Files likely touched

- `/data/nvme0/can/Projects/solo/plinth/docs/src/guides/activity.md` — new guide.
- `/data/nvme0/can/Projects/solo/plinth/docs/src/api/activity.md` — new API page.
- `/data/nvme0/can/Projects/solo/plinth/docs/src/configuration/plinth-toml.md` — `[ranking]`, `[forge]`, `feeds.activity_limit`.
- `/data/nvme0/can/Projects/solo/plinth/docs/src/configuration/environment-vars.md` — forge tokens + TTL note.
- `/data/nvme0/can/Projects/solo/plinth/docs/src/SUMMARY.md` — two new entries.

## Pitfalls

- **Symptom:** `mdbook build` warns "file not found" or a page is not in the rendered TOC.
  **Cause:** new page created but not added to `SUMMARY.md`, or a SUMMARY link with a typo'd path.
  **Recovery:** every page must have exactly one SUMMARY entry; paths are relative to `docs/src/` and start with `./`.

- **Symptom:** documented flags/keys do not match the shipped CLI/config.
  **Cause:** writing from the brief without checking Phase 05/04/07's final names (e.g. `update` vs `set-impact`, `half_life_days` vs `half_life`, `activity_limit` default).
  **Recovery:** grep the implemented code for the actual clap flag strings and serde field names before finalizing each table: `grep -rn 'activity' crates/cli/src crates/shared/src/toml_config.rs`.

- **Symptom:** a secret token, or a `github_user`/`codeberg_user` key, appears in the `[forge]` TOML table.
  **Cause:** copying the blog/portfolio pattern too literally, or inventing display keys no phase defines.
  **Recovery:** tokens are env-only (`GITHUB_TOKEN`/`CODEBERG_TOKEN`), never TOML. The `[forge]` table holds only `refresh_ttl_secs`, `refresh_backoff_secs`, `github_base_url`, and `codeberg_base_url`. There are no `github_user`/`codeberg_user` keys — do not document them.

- **Symptom:** the planning directory shows up in the published book.
  **Cause:** added a `planning/forge-activity` entry to SUMMARY.
  **Recovery:** do not add it; only the postgres-migration plan was historically listed, and that is a separate decision — keep activity planning out of nav.

## Reference

- Style templates (summarized inline above, no need to open): `docs/src/guides/publishing.md` (guide + tables), `docs/src/configuration/plinth-toml.md` / `environment-vars.md` (config tables), `docs/src/api/admin.md` (request-body table) / `api/search.md` (response JSON + params). SUMMARY structure: `docs/src/SUMMARY.md`.
- Docs build: `nix build .#docs` runs `mdbook build docs` (flake `docs` derivation); `book.toml` sets `src = "src"`, `build-dir = "book"`.
- Sibling sub-layers (CONTEXT only): [sub-01-nix-packaging.md](./sub-01-nix-packaging.md) ensures `nix build .#docs` is reachable; [sub-02-e2e-tests.md](./sub-02-e2e-tests.md) is independent. The merge agent (see [README.md](./README.md)) runs the full `nix flake check`.
