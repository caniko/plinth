# Phase 05 — CLI: plinth activity add/remove/update/list

> **Recommended Codex model: GPT 5.5 medium**
>
> This phase is mechanical pattern-replication with two genuinely fiddly spots: (1) plumbing a
> NEW crate dependency (`plinth-forge`) and a NEW Cargo feature (`brick-activity`) through the CLI
> manifest and the feature chain, and (2) gluing three independent subsystems together in one
> command — the forge fetch (async reqwest via `plinth-forge`), the fastembed embedding
> (CPU-blocking, off-runtime), and the Bearer-authed `POST` to the admin API. None of these are
> individually hard, but the wiring touches `main.rs` (clap enum + dispatch + `#[cfg]` import),
> `commands/mod.rs`, `api_client.rs`, and `Cargo.toml` in lockstep, and the `#[cfg(feature)]`
> gating must be consistent across all of them or the crate won't compile. A too-small model
> tends to: forget one of the four feature-chain edits, drop the `#[cfg]` on a new module/import,
> mis-handle the `--pr` XOR `--issue` validation (clap groups vs. manual check), or block the
> async runtime by calling fastembed inline instead of in `spawn_blocking`. Medium reasoning is
> enough to follow the existing `portfolio`/`todo`/`publish` idioms precisely without
> over-engineering.

## Working tree

cwd = `/data/nvme0/can/Projects/solo/plinth` (the plinth repo).

This phase is confined to `crates/cli/` (and consumes types already defined in `crates/shared`
by Phase 01 and the client crate in `crates/forge` by Phase 02). There is **no file-level
serialization conflict** with the other Wave-2 phases (04 server cache/refresh, 06 client, 07
server search/feed) — they touch `crates/server` and `crates/client`, not `crates/cli`. The only
shared edit risk is `Cargo.lock` and the root `Cargo.toml` `[workspace.dependencies]` table if
Phase 02 has not yet landed `plinth-forge` there. **Before starting:** confirm Phase 01
(`plinth-shared` activity types) and Phase 02 (`plinth-forge` crate) are merged into your base;
this phase will not compile without both. If they are not yet on `trunk`, rebase onto the branch
that contains them, or pull first. If you must add `plinth-forge` to
`[workspace.dependencies]` yourself because Phase 02 raced, do it, but expect a trivial
`Cargo.lock` merge later.

## Goal

This phase succeeds when a maintainer can run, from a built `plinth` binary:

```
plinth activity add --forge github --repo cli/cli --pr 9000 --impact 7 --featured
plinth activity list
plinth activity update <id> --impact 9 --featured false
plinth activity remove <id>
```

…and `plinth activity add` (a) fetches the PR/issue metadata from the forge via `plinth-forge`,
(b) generates a 384-dim fastembed embedding of `title + "\n\n" + body` locally (the server never
runs fastembed), (c) builds a `PublishActivityRequest` validated to `impact ∈ 1..=10` with
exactly one of `--pr`/`--issue`, and (d) `POST`s it to `/api/admin/activity` with a Bearer token
via `ApiClient`; `remove <id>`/`update <id>` take a numeric `i64` id and hit `DELETE`/`PATCH`
on `/api/admin/activity/{id}` (Bearer), and `list` hits the public `GET /api/activity` (no auth).
The build compiles with and without `--no-default-features`, `cargo clippy
--all-targets -- --deny warnings` is clean for the CLI crate, and a named unit test proves
`plinth activity add` builds the correct request from mocked forge data.

## Why this matters now

The `activity` brick's data does not seed itself: the owner curates each external contribution by
hand, and the CLI is the **only** ingestion path (the server refresh actor in Phase 04 only
*re-pulls* metadata for rows that already exist; it never creates rows and never embeds — see the
embedding pitfall below). Without this phase, the server brick (Phase 03), the frontend surfaces
(Phase 06), the feed and the search union (Phase 07) all have an empty table and nothing to show.
Critically, **embeddings only ever enter the system through this command** (Phase 04's refresh
deliberately does not re-embed because title/body rarely change), so semantic search over
contributions (Phase 07) is entirely dependent on `activity add` writing a non-null `embedding`.
This phase is the producer that every downstream consumer reads from.

## Out of scope

- **The server admin endpoints** (`POST/DELETE/PATCH /api/admin/activity`, `GET /api/activity`):
  owned by Phase 03. This phase assumes the contract below and talks to it over HTTP; do not add
  or edit anything under `crates/server/`.
- **The Kameo refresh / stale-while-revalidate actor**: Phase 04. Do not touch `cache.rs`/
  `refresh.rs`.
- **Frontend `/activity` pages, home strip, routes**: Phase 06 (`crates/client/`).
- **RSS feed and the pgvector search UNION**: Phase 07.
- **Defining `plinth-forge` itself** (the `ForgeClient` trait, `GitHubClient`, `CodebergClient`):
  Phase 02. This phase only *calls* its public API.
- **Defining the shared DTOs** (`PublishActivityRequest`, `ActivityListItem`, `Forge`,
  `ActivityKind`, `ActivityState`, `FetchedActivity`): Phase 01. Consume them; do not redefine.
- Do **not** add an interactive (`-i`) flow or a TOML-manifest flow for activity — `add` is
  flag-driven only (unlike `publish`/`portfolio`).

## Plan

All paths below are repo-relative to `/data/nvme0/can/Projects/solo/plinth` unless absolute.

### Step 1 — Cargo: add the `plinth-forge` dependency and the `brick-activity` feature

Edit `crates/cli/Cargo.toml`.

Add `plinth-forge` to `[dependencies]` (it lives at `crates/forge/`, declared as a workspace dep
by Phase 02; if `[workspace.dependencies]` lacks it, add
`plinth-forge = { path = "crates/forge" }` to the root `Cargo.toml` first):

```toml
# Workspace dependencies
plinth-shared = { workspace = true, default-features = false, features = ["config-toml"] }
plinth-forge = { workspace = true }
toml = { workspace = true }
```

Extend the `[features]` table — mirror the existing `brick-portfolio = ["plinth-shared/brick-portfolio"]`
line exactly, and append the new feature to `default`:

```toml
[features]
default = ["brick-blog", "brick-portfolio", "brick-todo", "brick-activity"]
brick-blog = ["plinth-shared/brick-blog"]
brick-portfolio = ["plinth-shared/brick-portfolio"]
brick-todo = ["plinth-shared/brick-todo"]
brick-activity = ["plinth-shared/brick-activity"]
```

Note: `plinth-forge` does **not** need to be feature-gated as an optional dep here — the CLI
always links it (it is small and reqwest-based; the CLI already pulls reqwest). The
`brick-activity` feature only gates the *shared types* and the *activity subcommand module*. (If
Phase 02 put `plinth-forge`'s reqwest usage behind a feature, enable that default feature; by the
brief, the crate is plain reqwest with no required feature.)

### Step 2 — Register the command module

Edit `crates/cli/src/commands/mod.rs` and add, mirroring the portfolio line (`mod.rs:5-6`):

```rust
#[cfg(feature = "brick-activity")]
pub mod activity;
```

### Step 3 — clap: subcommand enum + dispatch in `main.rs`

Edit `crates/cli/src/main.rs`.

**(3a)** Add the top-of-file gated import, mirroring `portfolio` (`main.rs:15-16`):

```rust
#[cfg(feature = "brick-activity")]
use commands::activity;
```

**(3b)** Add the variant inside `enum Commands` (after the `Portfolio` variant at `main.rs:83-86`):

```rust
    /// External activity (PRs/issues) management
    #[cfg(feature = "brick-activity")]
    #[command(subcommand)]
    Activity(ActivityCommands),
```

**(3c)** Define the `ActivityCommands` enum near the other `*Commands` enums (after
`PortfolioCommands` at `main.rs:109-117`). Use a clap `ArgGroup` to enforce the `--pr` XOR
`--issue` invariant at parse time AND keep the impact range check declarative via
`value_parser`:

```rust
#[cfg(feature = "brick-activity")]
#[derive(clap::Subcommand)]
enum ActivityCommands {
    /// Fetch a PR/issue from a forge, embed it, and publish it
    #[command(group(
        clap::ArgGroup::new("ref_kind")
            .required(true)
            .args(["pr", "issue"]),
    ))]
    Add {
        /// Forge to fetch from
        #[arg(long, value_enum)]
        forge: ForgeArg,
        /// Repository in owner/name form (e.g. cli/cli)
        #[arg(long)]
        repo: String,
        /// Pull-request number (mutually exclusive with --issue)
        #[arg(long, group = "ref_kind")]
        pr: Option<u32>,
        /// Issue number (mutually exclusive with --pr)
        #[arg(long, group = "ref_kind")]
        issue: Option<u32>,
        /// Curated impact score, 1..=10
        #[arg(long, value_parser = clap::value_parser!(u8).range(1..=10))]
        impact: u8,
        /// Mark as a featured (home-strip) entry
        #[arg(long)]
        featured: bool,
    },
    /// Remove an activity by numeric id
    Remove {
        /// Numeric id (e.g. 42)
        id: i64,
    },
    /// Update an existing activity's impact and/or featured flag
    Update {
        /// Numeric id (e.g. 42)
        id: i64,
        /// New impact score, 1..=10
        #[arg(long, value_parser = clap::value_parser!(u8).range(1..=10))]
        impact: Option<u8>,
        /// New featured flag (true/false)
        #[arg(long)]
        featured: Option<bool>,
    },
    /// List all activities
    List,
}

/// CLI-facing forge selector. Maps to `plinth_shared::Forge`.
#[cfg(feature = "brick-activity")]
#[derive(Clone, Copy, clap::ValueEnum)]
enum ForgeArg {
    Github,
    Codeberg,
}

#[cfg(feature = "brick-activity")]
impl From<ForgeArg> for plinth_shared::Forge {
    fn from(a: ForgeArg) -> Self {
        match a {
            ForgeArg::Github => plinth_shared::Forge::GitHub,
            ForgeArg::Codeberg => plinth_shared::Forge::Codeberg,
        }
    }
}
```

`clap::value_parser!(u8).range(1..=10)` makes `--impact 0` / `--impact 11` fail at parse time
with a clear clap message (`error: invalid value '11' for '--impact <IMPACT>': 11 is not in
1..=10`). The `ArgGroup` with `required(true)` makes "neither `--pr` nor `--issue`" and "both"
each a clap error. (Re-validate impact in the command fn too, see Step 4, so the invariant is
also enforced when the request is built from a non-CLI caller in tests.)

**(3d)** Add the dispatch arm inside the big `match &cli.command` (after the `Portfolio` arm at
`main.rs:387-392`):

```rust
        #[cfg(feature = "brick-activity")]
        Commands::Activity(activity_cmd) => match activity_cmd {
            ActivityCommands::Add {
                forge,
                repo,
                pr,
                issue,
                impact,
                featured,
            } => {
                activity::add(
                    (*forge).into(),
                    repo,
                    *pr,
                    *issue,
                    *impact,
                    *featured,
                    &api_client,
                )
                .await?;
            }
            ActivityCommands::Remove { id } => {
                activity::remove(*id, &api_client).await?;
            }
            ActivityCommands::Update {
                id,
                impact,
                featured,
            } => {
                activity::update(*id, *impact, *featured, &api_client).await?;
            }
            ActivityCommands::List => {
                activity::list(&api_client).await?;
            }
        },
```

### Step 4 — `commands/activity.rs` (the command implementations)

Create `crates/cli/src/commands/activity.rs`. This mirrors `commands/todo.rs` +
`commands/publish.rs` (the embedding helper) + `commands/portfolio.rs` (validation idioms).

Key facts to encode (from Phase 01 shared types and the brief):

- `PublishActivityRequest` (defined in `crates/shared/src/activity_item.rs` by Phase 01) carries
  EXACTLY: `forge: Forge`, `repo_owner: String`, `repo_name: String`, `kind: ActivityKind`,
  `number: i32`, `url`, `title`, `body: Option<String>`, `state: ActivityState`,
  `created_at`, `closed_at: Option<_>`, `merged_at: Option<_>`, `impact: i16`, `additions`,
  `deletions`, `comments_count`, `labels: Vec<String>`, `repo_stars`, `embedding: Option<Vec<f32>>`,
  `published: bool`, `content_hash: Option<String>`, `featured: bool`. There is **no `fetched_at`
  field** on the request — the **server** stamps `fetched_at = chrono::Utc::now()` on insert. The
  CLI sends `published: true` and `content_hash: None`. The struct literal in `build_request` must
  be exhaustive against this exact field set.
- The forge fetch goes through `plinth_forge::ForgeClient` (per Phase 02), whose ONLY fetch
  entrypoint is `async fn fetch(&self, r: &ActivityRef) -> ForgeResult<FetchedActivity>` (there is
  **no** `fetch_pull_request` / `fetch_issue`; PR-vs-issue routing is internal to the client, keyed
  off `r.kind`). Build an `ActivityRef { forge, owner, repo, kind, number }` and call
  `client.fetch(&r)`. Construct the client by `--forge`: `GitHubClient::new(token)` /
  `CodebergClient::new(token)`, where `token` comes from the optional `GITHUB_TOKEN` /
  `CODEBERG_TOKEN` env var (or wire a `ForgeRouter` and let it dispatch). `FetchedActivity` is the
  DTO in `plinth-shared` with the normalized fields (forge/repo_owner/repo_name/kind/number/url/
  title/body/state/dates/counts/labels/stars).
- The fastembed call is **identical** to `publish.rs:267-292` `generate_embedding` (model
  `AllMiniLML6V2`, 384-dim, `tokio::task::spawn_blocking`, truncate to 5000 bytes on a char
  boundary). Embed `title + "\n\n" + body.unwrap_or_default()`.

Module body:

```rust
use anyhow::{Context, Result};

use crate::api_client::ApiClient;
use crate::ui;
use plinth_shared::{
    ActivityKind, ActivityState, Forge, FetchedActivity, PublishActivityRequest,
};
use plinth_forge::ActivityRef;

/// `plinth activity add` — fetch a PR/issue from the forge, embed it locally, publish it.
#[allow(clippy::too_many_arguments)]
pub async fn add(
    forge: Forge,
    repo: &str,
    pr: Option<u32>,
    issue: Option<u32>,
    impact: u8,
    featured: bool,
    api_client: &ApiClient,
) -> Result<()> {
    // Defense-in-depth: clap already enforces these, but keep the invariants here so
    // the function is correct when called directly (e.g. from tests).
    if !(1..=10).contains(&impact) {
        anyhow::bail!("--impact must be between 1 and 10 (got {impact})");
    }
    let (owner, name) = split_repo(repo)?;
    let (kind, number) = match (pr, issue) {
        (Some(n), None) => (ActivityKind::PullRequest, n),
        (None, Some(n)) => (ActivityKind::Issue, n),
        (Some(_), Some(_)) => anyhow::bail!("pass exactly one of --pr or --issue, not both"),
        (None, None) => anyhow::bail!("pass exactly one of --pr or --issue"),
    };
    if number == 0 {
        anyhow::bail!("PR/issue number must be greater than 0");
    }

    // 1) Fetch normalized metadata from the forge.
    let sp = ui::spinner(&format!("Fetching {repo} #{number} from {forge:?}..."));
    let fetched: FetchedActivity = fetch(forge, &owner, &name, kind, number)
        .await
        .context("Failed to fetch activity from forge")?;
    sp.finish_and_clear();

    // 2) Embed title + body locally (server never runs fastembed).
    let embed_text = match &fetched.body {
        Some(body) if !body.trim().is_empty() => format!("{}\n\n{}", fetched.title, body),
        _ => fetched.title.clone(),
    };
    let sp = ui::spinner("Generating embedding...");
    let embedding = generate_embedding(&embed_text).await?;
    sp.finish_and_clear();
    ui::status("Embedded", &format!("{} dimensions", embedding.len()));

    // 3) Build the request from fetched data + CLI flags.
    let request = build_request(
        forge,
        &owner,
        &name,
        kind,
        number,
        impact,
        featured,
        embedding,
        &fetched,
    );

    // 4) POST to the admin API (Bearer auth handled by ApiClient).
    let sp = ui::spinner("Publishing activity...");
    let resp = api_client.publish_activity(request).await?;
    sp.finish_and_clear();
    ui::success(&format!("Activity published: {}", resp.url));
    Ok(())
}

/// Pure builder, factored out so it is unit-testable without network or the runtime.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_request(
    forge: Forge,
    owner: &str,
    name: &str,
    kind: ActivityKind,
    number: u32,
    impact: u8,
    featured: bool,
    embedding: Vec<f32>,
    fetched: &FetchedActivity,
) -> PublishActivityRequest {
    PublishActivityRequest {
        forge,
        repo_owner: owner.to_string(),
        repo_name: name.to_string(),
        kind,
        number: number as i32,
        url: fetched.url.clone(),
        title: fetched.title.clone(),
        body: fetched.body.clone(),
        state: fetched.state,
        created_at: fetched.created_at,
        closed_at: fetched.closed_at,
        merged_at: fetched.merged_at,
        impact: impact as i16,
        additions: fetched.additions,
        deletions: fetched.deletions,
        comments_count: fetched.comments_count,
        labels: fetched.labels.clone(),
        repo_stars: fetched.repo_stars,
        embedding: Some(embedding),
        published: true,
        content_hash: None,
        featured,
    }
}

/// `plinth activity remove <id>` — DELETE by numeric id.
pub async fn remove(id: i64, api_client: &ApiClient) -> Result<()> {
    let sp = ui::spinner(&format!("Removing activity {id}..."));
    api_client.delete_activity(id).await?;
    sp.finish_and_clear();
    ui::success(&format!("Activity removed: {id}"));
    Ok(())
}

/// `plinth activity update <id> [--impact N] [--featured B]` — PATCH by numeric id.
pub async fn update(
    id: i64,
    impact: Option<u8>,
    featured: Option<bool>,
    api_client: &ApiClient,
) -> Result<()> {
    if impact.is_none() && featured.is_none() {
        anyhow::bail!("nothing to update: pass --impact and/or --featured");
    }
    if let Some(i) = impact {
        if !(1..=10).contains(&i) {
            anyhow::bail!("--impact must be between 1 and 10 (got {i})");
        }
    }
    let sp = ui::spinner(&format!("Updating activity {id}..."));
    api_client
        .patch_activity(id, impact.map(|i| i as i16), featured)
        .await?;
    sp.finish_and_clear();
    ui::success(&format!("Activity updated: {id}"));
    Ok(())
}

/// `plinth activity list` — GET.
pub async fn list(api_client: &ApiClient) -> Result<()> {
    let items = api_client.list_activities().await?;
    if items.is_empty() {
        ui::status("Info", "No activities found.");
        return Ok(());
    }
    ui::status("Found", &format!("{} activity item(s)", items.len()));
    println!();
    for item in items {
        // item is plinth_shared::ActivityListItem (incl. the computed `score`).
        println!(
            "  {} {} #{}  impact={}  score={:.3}",
            ui::bold_style().apply_to(&item.title),
            format!("{}/{}", item.repo_owner, item.repo_name),
            item.number,
            item.impact,
            item.score,
        );
    }
    Ok(())
}

/// "owner/name" -> (owner, name); both must be non-empty.
fn split_repo(repo: &str) -> Result<(String, String)> {
    let mut parts = repo.splitn(2, '/');
    let owner = parts.next().unwrap_or("").trim();
    let name = parts.next().unwrap_or("").trim();
    if owner.is_empty() || name.is_empty() {
        anyhow::bail!("--repo must be in owner/name form (got '{repo}')");
    }
    Ok((owner.to_string(), name.to_string()))
}

/// Build the right plinth-forge client by `--forge`, then call its single `fetch` entrypoint.
/// Tokens come from env. PR-vs-issue routing is internal to the client (keyed off `r.kind`).
async fn fetch(
    forge: Forge,
    owner: &str,
    name: &str,
    kind: ActivityKind,
    number: u32,
) -> Result<FetchedActivity> {
    use plinth_forge::{CodebergClient, ForgeClient, GitHubClient};
    let client: Box<dyn ForgeClient> = match forge {
        Forge::GitHub => Box::new(GitHubClient::new(std::env::var("GITHUB_TOKEN").ok())),
        Forge::Codeberg => Box::new(CodebergClient::new(std::env::var("CODEBERG_TOKEN").ok())),
    };
    let r = ActivityRef {
        forge,
        owner: owner.to_string(),
        repo: name.to_string(),
        kind,
        number: number as i32,
    };
    let fetched = client.fetch(&r).await?;
    Ok(fetched)
}

/// Generate a 384-dim embedding with fastembed (copied from commands/publish.rs).
async fn generate_embedding(content: &str) -> Result<Vec<f32>> {
    use fastembed::{EmbeddingModel, TextEmbedding};
    let mut end = content.len().min(5000);
    while end > 0 && !content.is_char_boundary(end) {
        end -= 1;
    }
    let truncated = content[..end].to_string();
    tokio::task::spawn_blocking(move || -> Result<Vec<f32>> {
        let mut init_options = fastembed::TextInitOptions::default();
        init_options.model_name = EmbeddingModel::AllMiniLML6V2;
        init_options.show_download_progress = false;
        let mut model = TextEmbedding::try_new(init_options)?;
        let embeddings = model.embed(vec![truncated], None)?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Failed to generate embedding"))
    })
    .await?
}
```

Notes:
- The `plinth-forge` API is fixed by the canonical contract: `ForgeClient::fetch(&self, &ActivityRef)
  -> ForgeResult<FetchedActivity>` is the **only** fetch entrypoint (no `fetch_pull_request` /
  `fetch_issue`), with `GitHubClient::new(token: Option<String>)` / `CodebergClient::new(token:
  Option<String>)` constructors. PR-vs-issue routing is internal to the client (keyed off
  `r.kind`). Build an `ActivityRef`, call `client.fetch(&r)`. Error matches use struct patterns,
  e.g. `Err(ForgeError::NotFound { .. })`.
- The `PublishActivityRequest` / `FetchedActivity` / `ActivityListItem` field sets are the canonical
  ones above (the `activity_items` columns). The request literal must be **exhaustive** and include
  `published: true`, `content_hash: None`, `embedding: Some(..)`, `featured` — and must **not** carry
  `fetched_at` (the server stamps it). Do not invent or rename fields.
- Render owner/name in `list` via `format!("{}/{}", item.repo_owner, item.repo_name)` — there is no
  `repo_owner_name()` method on `ActivityListItem`.

### Step 5 — `api_client.rs`: four new gated methods

Edit `crates/cli/src/api_client.rs`. Add the imports (gated) at the top, mirroring the
portfolio import (`api_client.rs:2-3`):

```rust
#[cfg(feature = "brick-activity")]
use plinth_shared::{ActivityListItem, PublishActivityRequest};
```

Add a response struct next to `PublishPortfolioResponse` (`api_client.rs:22-31`):

```rust
/// Response from the publish activity endpoint
#[cfg(feature = "brick-activity")]
#[derive(Debug, serde::Deserialize)]
pub struct PublishActivityResponse {
    #[allow(dead_code)]
    pub success: bool,
    pub url: String,
    pub id: Option<i64>,
    #[allow(dead_code)]
    pub message: String,
}
```

(Match the JSON shape Phase 03's `POST /api/admin/activity` returns: `{ success, url, id, message }`.
The activity id is numeric (`i64`); the natural key is the forge URL, so `url` is the human-facing
identifier echoed back.)

Add four methods inside `impl ApiClient` (mirror `publish_portfolio` at `api_client.rs:128-174`
for the POST, `delete_todo`/`delete_article` for DELETE, `update_todo` for PATCH, and
`list_todos` for GET — all already in this file):

```rust
    /// Publish (upsert) an activity item.
    #[cfg(feature = "brick-activity")]
    pub async fn publish_activity(
        &self,
        request: PublishActivityRequest,
    ) -> Result<PublishActivityResponse> {
        let url = format!("{}/api/admin/activity", self.base_url);
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send activity publish request to API")?;
        let status = response.status();
        if status.is_success() {
            response
                .json()
                .await
                .context("Failed to parse activity publish response")
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                anyhow::bail!("API error: {} {}", err.error, err.details.unwrap_or_default());
            }
            anyhow::bail!("Activity publish failed (HTTP {status}): {error_text}");
        }
    }

    /// Delete an activity by numeric id.
    #[cfg(feature = "brick-activity")]
    pub async fn delete_activity(&self, id: i64) -> Result<()> {
        let url = format!("{}/api/admin/activity/{id}", self.base_url);
        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("Failed to send delete activity request")?;
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Delete activity {id} failed (HTTP {status}): {error_text}");
        }
        Ok(())
    }

    /// Patch impact and/or featured for an activity by numeric id.
    #[cfg(feature = "brick-activity")]
    pub async fn patch_activity(
        &self,
        id: i64,
        impact: Option<i16>,
        featured: Option<bool>,
    ) -> Result<()> {
        let url = format!("{}/api/admin/activity/{id}", self.base_url);
        let body = serde_json::json!({ "impact": impact, "featured": featured });
        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send patch activity request")?;
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Update activity {id} failed (HTTP {status}): {error_text}");
        }
        Ok(())
    }

    /// List all activity items (ranked, server-side). PUBLIC endpoint — no auth header.
    #[cfg(feature = "brick-activity")]
    pub async fn list_activities(&self) -> Result<Vec<ActivityListItem>> {
        let url = format!("{}/api/activity", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send list activities request")?;
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("List activities failed (HTTP {status}): {error_text}");
        }
        response
            .json()
            .await
            .context("Failed to parse activities list")
    }
```

No URL-encoding is needed: `remove`/`update` take a **numeric `i64` id** (the clap arg type is
`i64`), so the path segment is always a plain integer interpolated as `…/api/admin/activity/{id}`.
There is no id-or-url form and no `urlencoding` dependency — `DELETE`/`PATCH` map straight onto
`admin::delete_activity_handler` / `admin::patch_activity_handler`, which take `Path<i64>`.

`GET /api/activity` is the **public** list endpoint (`api::list_activity_items`, no auth), so
`list_activities` sends **no** `Authorization` header. The admin-authenticated calls
(`publish_activity`, `delete_activity`, `patch_activity`) send `Bearer {api_key}`.

### Step 6 — Tests

Add a `#[cfg(test)] mod tests` at the bottom of `crates/cli/src/commands/activity.rs`. The
load-bearing acceptance test is **`build_request` from mocked forge data** — it needs no network
and no runtime (the embedding is passed in as a fixture vector):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn sample_fetched() -> FetchedActivity {
        FetchedActivity {
            forge: Forge::GitHub,
            repo_owner: "cli".to_string(),
            repo_name: "cli".to_string(),
            kind: ActivityKind::PullRequest,
            number: 9000,
            url: "https://github.com/cli/cli/pull/9000".to_string(),
            title: "Fix the thing".to_string(),
            body: Some("Body text".to_string()),
            state: ActivityState::Merged,
            created_at: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            closed_at: Some(Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap()),
            merged_at: Some(Utc.with_ymd_and_hms(2025, 1, 2, 0, 0, 0).unwrap()),
            additions: Some(10),
            deletions: Some(3),
            comments_count: Some(2),
            labels: vec!["bug".to_string()],
            repo_stars: Some(1234),
        }
    }

    #[test]
    fn build_request_maps_fetched_and_flags() {
        let req = build_request(
            Forge::GitHub,
            "cli",
            "cli",
            ActivityKind::PullRequest,
            9000,
            7,
            true,
            vec![0.1_f32; 384],
            &sample_fetched(),
        );
        assert_eq!(req.forge, Forge::GitHub);
        assert_eq!(req.repo_owner, "cli");
        assert_eq!(req.repo_name, "cli");
        assert_eq!(req.kind, ActivityKind::PullRequest);
        assert_eq!(req.number, 9000);
        assert_eq!(req.impact, 7);
        assert!(req.featured);
        assert_eq!(req.url, "https://github.com/cli/cli/pull/9000");
        assert_eq!(req.state, ActivityState::Merged);
        assert_eq!(req.merged_at, sample_fetched().merged_at);
        assert_eq!(req.labels, vec!["bug".to_string()]);
        assert_eq!(req.embedding.as_ref().map(|e| e.len()), Some(384));
        assert!(req.published);
        assert_eq!(req.content_hash, None);
    }

    #[test]
    fn split_repo_rejects_bad_input() {
        assert!(split_repo("cli/cli").is_ok());
        assert!(split_repo("noslash").is_err());
        assert!(split_repo("/name").is_err());
        assert!(split_repo("owner/").is_err());
    }
}
```

The `sample_fetched()` literal sets **all** canonical `FetchedActivity` fields (`forge`,
`repo_owner`, `repo_name`, `kind`, `number`, `url`, `title`, `body`, `state`, `created_at`,
`closed_at`, `merged_at`, `additions`, `deletions`, `comments_count`, `labels`, `repo_stars`), and
`build_request` sets `published: true` + `content_hash: None`, so
`build_request_maps_fetched_and_flags` compiles against Phase 01's exact shape. (`build_request`
and `split_repo` are `pub(crate)`/module-private so the test in the same file can call them.)

Optionally add a clap smoke test in the same module asserting the command tree parses (this is
cheap and proves help/subcommands exist):

```rust
    #[test]
    fn impact_out_of_range_is_rejected() {
        use clap::Parser;
        let err = crate::Cli::try_parse_from([
            "plinth", "activity", "add", "--forge", "github",
            "--repo", "cli/cli", "--pr", "9000", "--impact", "11",
        ])
        .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("11") && msg.contains("1..=10"), "got: {msg}");
    }
```

For this to compile from a test module inside `main.rs`'s binary crate, `Cli` and
`ActivityCommands` must be reachable; if the binary's items are not visible from
`commands/activity.rs`, put the clap parse test in `crates/cli/tests/activity_cli.rs` instead
(an integration test sees the binary only via `assert_cmd`, which is not a dep). Simpler: keep the
clap-range proof as the **documented manual smoke** below and rely on the `build_request` unit
test as the named automated acceptance test. Either satisfies the acceptance criteria; prefer
the unit test for CI.

### Step 7 — Build and verify

```
cargo build -p plinth-cli
cargo build -p plinth-cli --no-default-features
cargo clippy -p plinth-cli --all-targets -- --deny warnings
cargo test -p plinth-cli
cargo run -p plinth-cli -- activity --help
cargo run -p plinth-cli -- activity add --help
cargo run -p plinth-cli -- activity add --forge github --repo cli/cli --impact 11
```

The last command must exit non-zero with a message naming `11` and the `1..=10` range (no
network call should happen — clap rejects before dispatch).

## Acceptance criteria

- [ ] `cargo test -p plinth-cli` passes, and includes a test named
  `commands::activity::tests::build_request_maps_fetched_and_flags` that constructs a
  `PublishActivityRequest` from a mocked `FetchedActivity` and asserts: `forge == Forge::GitHub`,
  `repo_owner == "cli"`, `repo_name == "cli"`, `kind == ActivityKind::PullRequest`,
  `number == 9000`, `impact == 7`, `featured == true`, `state == ActivityState::Merged`,
  `embedding.len() == 384`.
- [ ] `cargo test -p plinth-cli` includes `commands::activity::tests::split_repo_rejects_bad_input`
  proving `"cli/cli"` parses and `"noslash"`, `"/name"`, `"owner/"` each error.
- [ ] `cargo run -p plinth-cli -- activity --help` lists exactly the four subcommands
  `add`, `remove`, `update`, `list` (and `activity add --help` shows `--forge`, `--repo`,
  `--pr`, `--issue`, `--impact`, `--featured`).
- [ ] `cargo run -p plinth-cli -- activity add --forge github --repo cli/cli --pr 1 --impact 11`
  exits non-zero and prints an error mentioning `11` and the `1..=10` range, **without** making a
  network request (verifiable by it failing instantly even with no `PLINTH_API_KEY` set — clap
  rejects before credential resolution).
- [ ] `cargo run -p plinth-cli -- activity add --forge github --repo cli/cli --impact 5`
  (neither `--pr` nor `--issue`) exits non-zero with a clap error stating one of `--pr`/`--issue`
  is required; and supplying **both** also errors.
- [ ] `cargo run -p plinth-cli -- activity add --forge github --repo bogus --pr 1 --impact 5`
  exits non-zero with an error mentioning `owner/name` form (the `--repo` validator), when an API
  key is present so dispatch is reached.
- [ ] `cargo build -p plinth-cli --no-default-features` compiles (the `activity` module,
  `ActivityCommands`, `ForgeArg`, and the four `ApiClient` methods are all behind
  `#[cfg(feature = "brick-activity")]`, so no `brick-activity` items leak into the
  no-features build).
- [ ] `cargo clippy -p plinth-cli --all-targets -- --deny warnings` reports **0 warnings**.
- [ ] **Documented manual smoke** (for the live POST, no automated network test): with a server
  from Phase 03 running and `PLINTH_API_KEY` set,
  `plinth activity add --forge github --repo cli/cli --pr 9000 --impact 7 --featured`
  prints `Embedded 384 dimensions`, then `Activity published: https://github.com/cli/cli/pull/9000`,
  and a subsequent `plinth activity list` shows that row with `impact=7`.

## Files likely touched

- `crates/cli/Cargo.toml` — add `plinth-forge` dep; add `brick-activity` feature + to `default`.
- `crates/cli/src/commands/mod.rs` — `#[cfg] pub mod activity;`.
- `crates/cli/src/commands/activity.rs` — **new**: `add`/`remove`/`update`/`list`, `build_request`,
  `split_repo`, `fetch`, `generate_embedding`, `#[cfg(test)] mod tests`.
- `crates/cli/src/main.rs` — gated `use commands::activity;`; `Activity(ActivityCommands)` variant;
  `ActivityCommands` + `ForgeArg` enums + `From<ForgeArg> for Forge`; dispatch arm.
- `crates/cli/src/api_client.rs` — gated imports; `PublishActivityResponse`; `publish_activity`,
  `delete_activity`, `patch_activity`, `list_activities`.
- (possibly) root `Cargo.toml` `[workspace.dependencies]` — add `plinth-forge` if Phase 02 didn't.

## Pitfalls

- **Embedding blocks the runtime.** Symptom: the CLI hangs or the spinner freezes during embed.
  Cause: calling fastembed (`TextEmbedding::try_new` / `model.embed`) directly in the async fn.
  Recovery: it MUST run inside `tokio::task::spawn_blocking`, exactly as `publish.rs:279-291`.
- **Forgetting one feature-chain edit.** Symptom: `error[E0432]: unresolved import
  plinth_shared::PublishActivityRequest` or `cannot find type Forge`. Cause: `brick-activity`
  not chained to `plinth-shared/brick-activity` in `crates/cli/Cargo.toml`, OR Phase 01 hasn't
  landed the shared types yet. Recovery: verify the `brick-activity = ["plinth-shared/brick-activity"]`
  line and that `default` includes `brick-activity`; confirm Phase 01 is in your base.
- **`#[cfg]` leaks break `--no-default-features`.** Symptom: `--no-default-features` build fails
  with "cannot find ... ActivityCommands". Cause: a reference to a gated item from an ungated
  site (e.g. the dispatch arm or the import missing its `#[cfg(feature = "brick-activity")]`).
  Recovery: every activity-specific item (module, enum, variant, import, dispatch arm, ApiClient
  method, response struct) carries the gate. Build with `--no-default-features` to confirm.
- **`--pr`/`--issue` both-or-neither.** Symptom: panics or silently picks one. Cause: relying on
  manual matching without the clap `ArgGroup`. Recovery: the `#[command(group(ArgGroup::new(
  "ref_kind").required(true).args(["pr","issue"])))]` makes clap enforce exactly-one at parse
  time; keep the redundant `match (pr, issue)` in `add()` for non-CLI callers.
- **Using a forge URL (or string) as the remove/update key.** Symptom: `remove`/`update` 404, or
  a clap type error / a path that needs escaping. Cause: treating the arg as an id-or-url string.
  Recovery: the `remove`/`update` clap arg is a **numeric `i64` id**, and `delete_activity(id: i64)`
  / `patch_activity(id: i64, ..)` interpolate it straight into `…/api/admin/activity/{id}` (which
  maps to `delete_activity_handler` / `patch_activity_handler`, both `Path<i64>`). No URL form, no
  percent-encoding, no `urlencoding` dep.
- **Refresh does NOT re-embed (system-level pitfall to remember, not to fix here).** The Phase 04
  server refresh re-pulls forge metadata but deliberately never regenerates the embedding (title/
  body rarely change). Therefore **this command is the only writer of `embedding`**. If you skip
  embedding here (e.g. send `embedding: None`), the row will never become searchable in Phase 07.
  Always send `Some(embedding)`.
- **Reaching for a per-kind fetch method.** Symptom: `no method named fetch_pull_request` /
  `fetch_issue`. Cause: assuming two entrypoints. The canonical `ForgeClient` has exactly ONE:
  `fetch(&self, &ActivityRef) -> ForgeResult<FetchedActivity>`; PR-vs-issue routing is internal to
  the client (keyed off `r.kind`). Recovery: build `ActivityRef { forge, owner, repo, kind, number }`
  and call `client.fetch(&r)`; match errors with struct patterns (`Err(ForgeError::NotFound { .. })`).
- **`FetchedActivity`/`PublishActivityRequest` field-name drift.** Symptom: struct-literal
  field errors in `build_request`. Cause: Phase 01 named a column differently (e.g. `comments`
  vs `comments_count`). Recovery: read `crates/shared/src/activity_item.rs` and use Phase 01's
  exact field names; do not add or rename fields from this phase.

## Reference

- **Design brief**: the "Forge Activity" brick spec. The `activity_items` schema columns,
  `POST/DELETE/PATCH /api/admin/activity` + `GET /api/activity` endpoints, the `Forge` /
  `ActivityKind` / `ActivityState` enums, and the "CLI embeds, server does not" rule are all
  defined there and inlined above — no need to open it to execute this phase.
- **Phase 01 — shared-types-and-migration** (`./01-shared-types-and-migration.md`): must land
  first; defines `PublishActivityRequest`, `ActivityListItem`, `FetchedActivity`, `Forge`,
  `ActivityKind`, `ActivityState` in `crates/shared`. Sequencing only.
- **Phase 02 — forge-crate** (`./02-forge-crate.md`): must land first; defines the
  `plinth-forge` `ForgeClient` trait (single `fetch(&ActivityRef)` entrypoint), `ActivityRef`,
  `GitHubClient`/`CodebergClient`, and `ForgeError`/`ForgeResult`. Sequencing only — the canonical
  signatures used by `fetch()` above are authoritative.
- **Phase 03 — server-brick-core** (`./03-server-brick-core.md`): must land first (or the live
  POST has no endpoint); owns the admin/public handlers this CLI calls. Sequencing only — the
  endpoint contract is inlined above.
- **Existing CLI patterns to copy** (real files in this repo):
  `crates/cli/src/main.rs` (clap enum + dispatch, lines 49-396),
  `crates/cli/src/commands/todo.rs` (fullest command-group example),
  `crates/cli/src/commands/publish.rs` (the `generate_embedding` helper, lines 267-292),
  `crates/cli/src/commands/portfolio.rs` (validation idioms),
  `crates/cli/src/api_client.rs` (Bearer POST/DELETE/PATCH/GET methods).
