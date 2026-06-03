# Phase 02 — The plinth-forge crate (GitHub + Forgejo clients)

> **Recommended Codex model: GPT 5.5 high**
>
> This phase is the single most error-prone in the whole feature because it reconciles two
> divergent external API shapes (GitHub REST vs. Forgejo/Gitea) into one normalized DTO, and a
> subtle mis-normalization ships wrong data sitewide — a PR shown as "open" when it merged, a
> missing `merged_at` that silently demotes a high-impact contribution in the ranking SQL, or a
> swallowed 404 that leaves deleted upstream items lingering. The GitHub `/issues/{n}` vs
> `/pulls/{n}` field-presence quirk (no `merged`/`additions`/`deletions` on the issues endpoint),
> the two different stars field names (`stargazers_count` vs `stars_count`), the merge-detection
> rule (`state == "closed" && merged_at != null`), and the rate-limit divergence (GitHub exposes
> `x-ratelimit-*` headers + 60/hr unauth; Codeberg exposes none, only reactive 429) all require
> careful, simultaneous handling of two protocols plus typed error modeling. A smaller model would
> conflate the two endpoint shapes, miss the merged-via-issues-endpoint derivation, or write tests
> that hit the real network (which the Nix sandbox forbids). High tier buys the cross-API attention
> and the discipline to mock every request.

## Working tree

cwd = `/data/nvme0/can/Projects/solo/plinth` (the plinth repo, branch `trunk`).

This is a **new, self-contained crate** at `crates/forge/`. It touches almost no existing files:
only the workspace `Cargo.toml` (add the member) and `flake.nix` (add the source-filter line).
Both edits are tiny and additive.

Serialization note: Phase 02 runs in **Wave 1 alongside Phase 03** (server brick core). The two
are disjoint by design — Phase 02 owns `crates/forge/**`, Phase 03 owns
`crates/server/src/bricks/activity/**`. The only shared file is the workspace root `Cargo.toml`
(Phase 02 adds `"crates/forge"` to `members`; Phase 03 does not edit `members`). If you discover a
merge conflict in `Cargo.toml`, it is a trivial members-list addition — re-add your line and move
on. **Phase 02 depends on Phase 01** (`plinth-shared` must already export `FetchedActivity`,
`Forge`, `ActivityKind`, `ActivityState` behind the `brick-activity` feature). If those types are
not yet present, do not invent them here — pull/rebase until Phase 01 has landed, because this
crate normalizes *into* those exact types and the contract must match.

## Goal

This phase succeeds when a new library crate `plinth-forge` (at `crates/forge/`) is a workspace
member that builds clean under `cargo build -p plinth-forge` and `cargo clippy -p plinth-forge
--all-targets -- --deny warnings`, and exposes a `ForgeClient` async trait with two impls —
`GitHubClient` (against `https://api.github.com`) and `CodebergClient` (against
`https://codeberg.org/api/v1`, Forgejo) — that each fetch a single PR or single issue (plus the
repo for star count) and **normalize both forges' divergent payloads into the shared
`plinth_shared::FetchedActivity` DTO**, with typed errors distinguishing not-found (404/410),
rate-limited (429/403-with-exhausted-quota, honoring `Retry-After` / `x-ratelimit-reset`), and
network/decode failures. The crate is depended on by `plinth-server` and `plinth-cli` later; it
**must not** be reachable from `plinth-client` (keep `reqwest` out of the WASM build). It ships
unit + integration tests using **wiremock** (transport-level mocking — no real network) covering,
by name: GitHub merged PR, GitHub issue, Codeberg PR, a 404, and a rate-limited response.

## Why this matters now

Forge fetching is needed in **two** places: the server's lazy refresh actor (Phase 04) re-pulls
forge metadata on stale reads, and the CLI `activity add` command (Phase 05) fetches a PR/issue at
add-time before embedding and POSTing it. Putting this logic in a shared library crate now is what
prevents the duplication the design brief explicitly forbids ("it lives in a shared crate, NOT
duplicated"). Deferring it would force Phase 04 and Phase 05 to each grow their own ad-hoc reqwest
calls, which would then drift in their normalization rules — the exact failure this crate exists to
prevent. Because both the refresh actor and the CLI feed the same `activity_items` table and the
same ranking SQL, the normalization must be *one* canonical implementation. This is Wave-1 work so
that Phase 04 (`depends on: 02, 03`) and Phase 05 (`depends on: 01, 02, 03`) can both build on it.

## Out of scope

- **No persistence.** This crate must not depend on `sqlx`, `pgvector`, or touch any database. It
  returns DTOs; the server/CLI persist them.
- **No brick.** Do not create or edit `crates/server/src/bricks/activity/**` — that is Phase 03.
- **No CLI command.** Do not add an `activity` subcommand or edit `crates/cli/**` — Phase 05.
- **No server wiring** beyond declaring the crate. Do not add `plinth-forge` to
  `crates/server/Cargo.toml` dependencies (Phase 04 does that), do not edit `main.rs`/`lib.rs`.
- **No embeddings / fastembed.** This crate does not embed anything; the CLI (Phase 05) embeds.
- **No config struct edits.** `ForgeConfig` / `[forge]` / token env-override wiring lives in
  `crates/shared/src/toml_config.rs` and is Phase 04/05 concern. This crate reads tokens **only**
  from explicit constructor arguments or `std::env` at call sites it owns (see Plan step 6) — it
  defines no global config type.
- **Do not edit `plinth-client`'s Cargo.toml or add any `brick-activity` chaining to it.** The
  whole point is that `reqwest` never enters the WASM build.
- **No `bin-features`/`lib-features` edits** in the root `Cargo.toml` — this crate is a plain lib
  consumed by server/CLI, not compiled by cargo-leptos.

## Plan

### 1. Add the crate to the workspace

Edit `/data/nvme0/can/Projects/solo/plinth/Cargo.toml` line 3 — add `"crates/forge"`:

```toml
members = ["crates/shared", "crates/client", "crates/server", "crates/cli", "crates/forge"]
```

### 2. Create `crates/forge/Cargo.toml`

The workspace pins (root `Cargo.toml`): `reqwest = "0.13"` (lockfile resolves 0.12.28),
`tokio = "1"`, `serde = "1"` (derive), `serde_json = "1"`, `chrono = "0.4"` (serde),
`thiserror = "2"`, `plinth-shared = { path = "crates/shared", default-features = false }`.
edition is `2024` (workspace). Use `async-trait` for the object-safe `ForgeClient` (add it as a
direct dep — it is not yet in the workspace deps; pin `async-trait = "0.1"` locally, it is tiny and
WASM-irrelevant since this crate is never in the WASM build).

```toml
[package]
name = "plinth-forge"
version.workspace = true
edition.workspace = true

[dependencies]
plinth-shared = { workspace = true, features = ["brick-activity"] }
reqwest = { workspace = true, features = ["json"] }
tokio = { workspace = true, features = ["time"] }
serde = { workspace = true }
serde_json = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
async-trait = "0.1"
tracing = { workspace = true }

[dev-dependencies]
wiremock = "0.6"
tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }
serde_json = { workspace = true }

[features]
default = []
```

Rationale per the brief and research:
- `reqwest` with `json` feature for `.json::<T>()` decoding.
- `wiremock = "0.6"` is the R5-recommended mock dep: transport-level, reqwest-version-agnostic
  (works with the locked reqwest 0.12.28), binds loopback inside the Nix sandbox (permitted), and
  needs **zero** production-code change because the client targets a configurable base URL.
- `plinth-shared` is brought in with **only** `brick-activity` so the DTO compiles; do **not**
  pull `default-features` (avoids dragging blog/portfolio/todo type modules into this crate). The
  `brick-activity` feature on `plinth-shared` is created by Phase 01.

### 3. Crate module layout

Create these files under `crates/forge/src/`:

```
crates/forge/src/
  lib.rs        -- re-exports + the ActivityRef + the ForgeClient trait
  error.rs      -- ForgeError (thiserror) typed errors
  github.rs     -- GitHubClient + GitHub-specific payload structs + normalization
  codeberg.rs   -- CodebergClient + Forgejo payload structs + normalization
  router.rs     -- ForgeRouter { github, codeberg } impl ForgeClient; dispatches by r.forge
```

And the test files:

```
crates/forge/tests/
  github.rs     -- wiremock integration tests for GitHubClient
  codeberg.rs   -- wiremock integration tests for CodebergClient
```

### 4. `error.rs` — typed errors

The brief requires typed errors for 404/410 (deleted upstream), rate-limit
(429 / 403-with-exhausted-quota / `Retry-After` / `x-ratelimit-*`), and network/decode. Model it
with `thiserror` (workspace `thiserror = "2"`). **Every variant is a struct or single-field tuple
variant exactly as the canonical contract fixes it, and the `forge` field is the shared `Forge`
enum (Phase 01), NOT a `&'static str`** — callers in Phase 04/05 match `Err(ForgeError::NotFound {
.. })` / `Err(ForgeError::RateLimited { .. })` and read `forge: Forge`:

```rust
use std::time::Duration;

use plinth_shared::Forge;

/// Errors returned by forge clients. Callers (the refresh actor in Phase 04, the CLI in Phase 05)
/// match on these to decide: drop a deleted item (NotFound), back off and keep stale data
/// (RateLimited), or surface a transient failure (Network/Decode).
#[derive(Debug, thiserror::Error)]
pub enum ForgeError {
    /// 404 or 410 — the PR/issue/repo no longer exists upstream (deleted/transferred away).
    #[error("forge resource not found ({forge}: {url}, http {status})")]
    NotFound { forge: Forge, url: String, status: u16 },

    /// 429, or 403 with an exhausted quota. `retry_after` is the best-known wait:
    /// from `Retry-After` (both forges) or derived from GitHub `x-ratelimit-reset`.
    #[error("forge rate limited ({forge}; retry after {retry_after:?})")]
    RateLimited {
        forge: Forge,
        retry_after: Option<Duration>,
    },

    /// Any other non-success HTTP status (5xx, unexpected 4xx); `body` is the captured response text.
    #[error("forge http error ({forge}: http {status})")]
    Http { forge: Forge, status: u16, body: String },

    /// Transport failure (DNS, connection, timeout) — the formatted message from the source error.
    #[error("forge network error: {0}")]
    Network(String),

    /// Body decode / JSON shape mismatch — the formatted message from the source error.
    #[error("forge decode error: {0}")]
    Decode(String),
}

pub type ForgeResult<T> = Result<T, ForgeError>;
```

`Forge` derives `Display` in Phase 01 (or use `forge.as_str()` in the messages if it does not — the
`{forge}` format expects `Display`; substitute `{}` with `forge.as_str()` if needed). Build
`Network`/`Decode` from a `reqwest::Error` with `ForgeError::Network(e.to_string())` /
`ForgeError::Decode(e.to_string())` at the call sites in `github.rs` / `codeberg.rs`.

### 5. `lib.rs` — the `ForgeClient` trait

Mirror the design brief's "`ForgeClient` trait with `GitHubClient` and `CodebergClient` impls,
normalizing both into a single `FetchedActivity` DTO". The shared DTO and enums come from Phase 01
(`plinth_shared::{FetchedActivity, Forge, ActivityKind, ActivityState}`). **Do not redefine them.**
For reference, Phase 01's `FetchedActivity` is the WASM-safe, reqwest-free struct carrying:
`forge: Forge`, `repo_owner: String`, `repo_name: String`, `kind: ActivityKind`, `number: i32`,
`url: String`, `title: String`, `body: Option<String>`, `state: ActivityState`,
`created_at: DateTime<Utc>`, `closed_at: Option<DateTime<Utc>>`, `merged_at: Option<DateTime<Utc>>`,
`additions: Option<i32>`, `deletions: Option<i32>`, `comments_count: Option<i32>`,
`labels: Vec<String>`, `repo_stars: Option<i32>`. `ActivityKind` is `{ PullRequest, Issue }`;
`ActivityState` is `{ Open, Closed, Merged }`; `Forge` is `{ GitHub, Codeberg }`. (Inline this
contract here so the agent need not open Phase 01; if the actual Phase-01 fields differ, **the
Phase-01 definition wins** — adjust the normalization, never redefine the type.)

```rust
//! plinth-forge — fetch a single PR/issue from GitHub or Forgejo and normalize into
//! `plinth_shared::FetchedActivity`. reqwest-based; MUST NOT be depended on by plinth-client
//! (keep reqwest out of the WASM build).

mod codeberg;
mod error;
mod github;
mod router;

pub use codeberg::CodebergClient;
pub use error::{ForgeError, ForgeResult};
pub use github::GitHubClient;
pub use router::ForgeRouter;

use async_trait::async_trait;
use plinth_shared::{ActivityKind, FetchedActivity, Forge};

/// Identifies one PR/issue on one repo. `forge` selects the backend (so a `ForgeRouter` can
/// dispatch on it); `number` is the PR/issue number (GitHub) or index (Forgejo), a positive
/// integer. This is the single canonical fetch input — Phase 04 (refresh) and Phase 05 (CLI add)
/// each build an `ActivityRef` and call `client.fetch(&r)`.
#[derive(Debug, Clone)]
pub struct ActivityRef {
    pub forge: Forge,
    pub owner: String,
    pub repo: String,
    pub kind: ActivityKind,
    pub number: i32,
}

#[async_trait]
pub trait ForgeClient: Send + Sync {
    /// Fetch and normalize a single PR or issue (and the repo's star count) into a
    /// `FetchedActivity`. PR-vs-issue routing is internal to the impl (keyed off `r.kind`); there is
    /// deliberately NO `fetch_one` / `fetch_pull_request` / `fetch_issue` on this trait — `fetch` is
    /// the sole entrypoint. Stamping `fetched_at` is NOT this crate's job — the caller stamps it
    /// (refresh time is a server/CLI concern); this DTO carries only forge-sourced fields.
    async fn fetch(&self, r: &ActivityRef) -> ForgeResult<FetchedActivity>;
}
```

`GitHubClient` and `CodebergClient` each `impl ForgeClient` directly (their internal `fetch` routes
PR vs issue by `r.kind`). The production object — wired by the server refresh actor (Phase 04) and
the CLI (Phase 05) as `Arc<dyn ForgeClient>` — is the `ForgeRouter`, which holds one client per
forge and dispatches on `r.forge`. Add `crates/forge/src/router.rs`:

```rust
use async_trait::async_trait;
use plinth_shared::{FetchedActivity, Forge};

use crate::{ActivityRef, CodebergClient, ForgeClient, ForgeResult, GitHubClient};

/// Holds one client per forge and dispatches `fetch` by `r.forge`. This is what production wires as
/// `Arc<dyn ForgeClient>`; tests inject a mock `Arc<dyn ForgeClient>` instead.
pub struct ForgeRouter {
    pub github: GitHubClient,
    pub codeberg: CodebergClient,
}

impl ForgeRouter {
    /// Build a router from optional tokens, using the default forge base URLs.
    pub fn new(github_token: Option<String>, codeberg_token: Option<String>) -> Self {
        Self {
            github: GitHubClient::new(github_token),
            codeberg: CodebergClient::new(codeberg_token),
        }
    }
}

#[async_trait]
impl ForgeClient for ForgeRouter {
    async fn fetch(&self, r: &ActivityRef) -> ForgeResult<FetchedActivity> {
        match r.forge {
            Forge::GitHub => self.github.fetch(r).await,
            Forge::Codeberg => self.codeberg.fetch(r).await,
        }
    }
}
```

Phase 04 builds the router with `with_base_url` (from `ForgeConfig` base URLs + env tokens) so the
base is overridable in production; the convenience `ForgeRouter::new` above uses the defaults.

Note: `FetchedActivity` carries no `fetched_at` (per the brief, `fetched_at` is the DB
snapshot/refresh time, stamped by the server/CLI, not by the forge). If Phase 01 chose to include
`fetched_at` on `FetchedActivity`, leave it `Default`/`None`-equivalent here and let the caller set
it — do not invent a clock dependency in this crate.

Constructors take an **optional token** and an **overridable base URL** (the base URL override is
mandatory — it is the wiremock injection point):

```rust
impl GitHubClient {
    /// `base_url` defaults to "https://api.github.com" via `GitHubClient::default()` or
    /// `GitHubClient::new(token)`. Tests pass the wiremock server URI.
    pub fn new(token: Option<String>) -> Self { Self::with_base_url("https://api.github.com".into(), token) }
    pub fn with_base_url(base_url: String, token: Option<String>) -> Self { /* build reqwest::Client */ }
}
```

Build the inner `reqwest::Client` with `Client::builder().build()` (NOT `Client::new()`), mirroring
the CLI's `ApiClient` rationale at `crates/cli/src/api_client.rs:53-62` ("avoid a panic when CA
certs are missing"). Set a `User-Agent` header on the builder — **GitHub rejects requests without a
`User-Agent`** with 403. Use e.g. `.user_agent("plinth-forge")`.

### 6. `github.rs` — GitHubClient + normalization (the load-bearing part)

Endpoints (R6, verified live against `api.github.com`):

| Purpose | GitHub path |
|---|---|
| Single PR | `GET /repos/{owner}/{repo}/pulls/{number}` |
| Single issue | `GET /repos/{owner}/{repo}/issues/{number}` |
| Repo (for stars) | `GET /repos/{owner}/{repo}` → `stargazers_count` |

Required headers on every GitHub request:
`Accept: application/vnd.github+json`, `X-GitHub-Api-Version: 2022-11-28`, and (if a token is set)
`Authorization: Bearer <TOKEN>`. The `User-Agent` is set on the client builder (step 5).

**THE CRITICAL QUIRK (R6, verified live):** on GitHub a PR is also an issue, but the two endpoints
return different fields. `/issues/{n}` does **not** carry `merged`, `additions`, or `deletions`. It
only has a `pull_request` sub-object whose `merged_at` you can read. **Therefore: if
`ActivityRef.kind == ActivityKind::PullRequest`, always call `/pulls/{n}`** (gives `merged`,
`additions`, `deletions`). Only call `/issues/{n}` for `ActivityKind::Issue`.

Define serde structs for exactly the fields you read (use `serde_json` via `.json::<T>()`):

```rust
#[derive(serde::Deserialize)]
struct GhPull {
    title: String,
    body: Option<String>,
    state: String,                 // "open" | "closed"
    merged: Option<bool>,
    merged_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    closed_at: Option<chrono::DateTime<chrono::Utc>>,
    additions: Option<i32>,
    deletions: Option<i32>,
    comments: Option<i32>,
    labels: Vec<GhLabel>,
}
#[derive(serde::Deserialize)]
struct GhIssue {
    title: String,
    body: Option<String>,
    state: String,
    created_at: chrono::DateTime<chrono::Utc>,
    closed_at: Option<chrono::DateTime<chrono::Utc>>,
    comments: Option<i32>,
    labels: Vec<GhLabel>,
    pull_request: Option<GhIssuePrMeta>,   // present only if this issue IS a PR
}
#[derive(serde::Deserialize)]
struct GhIssuePrMeta { merged_at: Option<chrono::DateTime<chrono::Utc>> }
#[derive(serde::Deserialize)]
struct GhLabel { name: String }
#[derive(serde::Deserialize)]
struct GhRepo { stargazers_count: Option<i32> }
```

dates: GitHub emits RFC3339 with `Z`; `chrono::DateTime<Utc>` with serde parses it directly.

**Merge / state normalization rule (both forges):**
`state == "closed" && merged_at.is_some()` ⇒ `ActivityState::Merged`;
`state == "closed" && merged_at.is_none()` ⇒ `ActivityState::Closed`;
otherwise ⇒ `ActivityState::Open`. For PRs, also prefer the explicit `merged: Some(true)` boolean
as a corroborating signal, but `merged_at` is the canonical input the ranking SQL keys off (the
reference date is `coalesce(merged_at, closed_at, created_at)`), so a merged PR **must** carry
`merged_at`.

`fetch()` flow for GitHub:
1. If `kind == PullRequest`: `GET /repos/{o}/{r}/pulls/{n}` → `GhPull`. Else
   `GET /repos/{o}/{r}/issues/{n}` → `GhIssue` (read `merged_at` from `pull_request.merged_at`).
2. `GET /repos/{o}/{r}` → `GhRepo` for `stargazers_count`. (One extra call. If this call alone
   fails with a non-fatal error you may set `repo_stars = None` rather than failing the whole
   fetch — but a 404 on the *primary* PR/issue call IS fatal and must surface as `NotFound`.)
3. Normalize into `FetchedActivity` (forge = `Forge::GitHub`, url = the upstream html url; build it
   as `https://github.com/{owner}/{repo}/pull/{n}` for PRs or `.../issues/{n}` for issues — the
   API also returns `html_url`, you may deserialize and use that instead, which is more robust).

**Status → error mapping** (apply this to every request before decoding). `forge` is the shared
`Forge` enum value (here `Forge::GitHub`); the variants are the canonical struct shapes — `NotFound`
keeps `{ forge, url, status }`, `RateLimited` is `{ forge, retry_after }` (no `url`/`status`), and
`Http` is `{ forge, status, body }` (no `url`). Because `Http` needs the response body, status
mapping reads the body before constructing the error, so this helper is `async` (or pass the
already-read body text in):

```rust
async fn map_status(forge: Forge, resp: reqwest::Response) -> Result<reqwest::Response, ForgeError> {
    let status = resp.status();
    if status.is_success() { return Ok(resp); }
    let code = status.as_u16();
    Err(match code {
        404 | 410 => ForgeError::NotFound { forge, url: resp.url().to_string(), status: code },
        429 => ForgeError::RateLimited { forge, retry_after: retry_after_from(&resp) },
        403 if rate_limit_exhausted(&resp) =>
            ForgeError::RateLimited { forge, retry_after: retry_after_from(&resp) },
        _ => {
            let body = resp.text().await.unwrap_or_default();
            ForgeError::Http { forge, status: code, body }
        }
    })
}
```

GitHub rate-limit signals (R6, verified live):
- `403` or `429` with `x-ratelimit-remaining: 0` ⇒ rate limited. `rate_limit_exhausted(resp)`
  reads the `x-ratelimit-remaining` header and treats `"0"` as exhausted.
- `retry_after_from(resp)`: prefer the `Retry-After` header (seconds, integer) →
  `Duration::from_secs`. If absent on GitHub, derive from `x-ratelimit-reset` (UTC epoch seconds):
  `Duration::from_secs((reset - now).max(0))`. Implement both reads; return `None` if neither
  present.

Wrap reqwest send errors as `ForgeError::Network(e.to_string())`, and `.json()` decode errors as
`ForgeError::Decode(e.to_string())` (these variants carry only the formatted message — no `forge`
/ `url` fields, per the canonical contract).

### 7. `codeberg.rs` — CodebergClient + normalization

Base URL `https://codeberg.org/api/v1` (any Forgejo instance: `<base>/api/v1`).

| Purpose | Forgejo path |
|---|---|
| Single PR | `GET /repos/{owner}/{repo}/pulls/{index}` |
| Single issue | `GET /repos/{owner}/{repo}/issues/{index}` |
| Repo (for stars) | `GET /repos/{owner}/{repo}` → `stars_count` |

Auth header (R6): Forgejo requires the literal word `token` + space:
`Authorization: token <TOKEN>` (OAuth2 tokens also accept `Bearer`, but use `token` for PATs).
`Accept: application/json`. No `X-GitHub-Api-Version` header. No special `User-Agent` requirement,
but set one anyway (harmless, consistent).

Forgejo serde structs — note the field-name differences from GitHub (`stars_count`, and the issue
endpoint's `pull_request` carries `merged`/`merged_at`):

```rust
#[derive(serde::Deserialize)]
struct FjPull {
    title: String,
    body: Option<String>,
    state: String,                 // "open" | "closed"
    merged: Option<bool>,
    merged_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    closed_at: Option<chrono::DateTime<chrono::Utc>>,
    additions: Option<i32>,        // i64 in spec; deserialize as i32 is fine for realistic diffs,
    deletions: Option<i32>,        // but use i64 -> cast if you want to be strict
    comments: Option<i32>,
    labels: Vec<FjLabel>,
}
#[derive(serde::Deserialize)]
struct FjIssue {
    title: String,
    body: Option<String>,          // Forgejo issue body is a string (may be empty)
    state: String,
    created_at: chrono::DateTime<chrono::Utc>,
    closed_at: Option<chrono::DateTime<chrono::Utc>>,
    comments: Option<i32>,
    labels: Vec<FjLabel>,
    pull_request: Option<FjPrMeta>,
}
#[derive(serde::Deserialize)]
struct FjPrMeta { merged: Option<bool>, merged_at: Option<chrono::DateTime<chrono::Utc>> }
#[derive(serde::Deserialize)]
struct FjLabel { name: String }
#[derive(serde::Deserialize)]
struct FjRepo { stars_count: Option<i32> }
```

dates: Forgejo emits RFC3339 with offsets (e.g. `+02:00`); `chrono::DateTime<Utc>` serde parses
RFC3339 with offset and normalizes to UTC — no special handling needed. (Confirm the field is not
`null`-typed in a way that breaks `Option`; map empty strings on `body` to `Some("")` which is
fine, or normalize empty → `None` if you prefer — be consistent with GitHub, which sends `null`.)

Same `fetch()` flow as GitHub (PR → `/pulls`, issue → `/issues` reading `pull_request.merged_at`,
plus the repo call for `stars_count`), same merge/state rule, normalizing into
`FetchedActivity { forge: Forge::Codeberg, .. }`. URL: build
`https://codeberg.org/{owner}/{repo}/pulls/{n}` (PR) or `.../issues/{n}` (issue), or use the
`html_url` field the API returns.

**Rate-limit divergence (R6, verified live):** Codeberg returns **no** `X-RateLimit-*` headers
(HAProxy-enforced ~2000 req/300s, IP-based). So `rate_limit_exhausted()` can never key off
remaining-quota headers for Codeberg — treat **429 reactively**: a 429 ⇒
`ForgeError::RateLimited { forge: Forge::Codeberg, retry_after }` with `retry_after` from
`Retry-After` if HAProxy set it, else `None` (caller applies fixed backoff). Do **not** look for
`x-ratelimit-reset` on Codeberg; it's absent. 404 (and any 410-equivalent, though Forgejo uses 404
for deleted) ⇒ `ForgeError::NotFound { forge: Forge::Codeberg, url, status }`. Reuse the same
`map_status` shape as GitHub, passing `Forge::Codeberg`.

### 8. Tests — `crates/forge/tests/github.rs` and `crates/forge/tests/codeberg.rs`

Use **wiremock** (no real network — the Nix sandbox `plinth-test` check has no internet; wiremock
binds loopback, which is permitted). Pattern (R5): start a `MockServer`, mount a `Mock` matching
method+path returning a canned JSON body, construct the client with `with_base_url(server.uri(),
None)`, call `fetch`, assert the normalized DTO.

Each test must be **named exactly** as below (the acceptance criteria name them):

`crates/forge/tests/github.rs`:
- `github_pr_merged` — mount `GET /repos/octocat/hello/pulls/1` → `200` with a PR body where
  `state="closed"`, `merged=true`, `merged_at` set, `additions/deletions` set; mount
  `GET /repos/octocat/hello` → `200` `{"stargazers_count": 42}`. Assert the result has
  `state == ActivityState::Merged`, `merged_at.is_some()`, `additions == Some(...)`,
  `repo_stars == Some(42)`, `kind == ActivityKind::PullRequest`, `forge == Forge::GitHub`.
- `github_issue` — mount `GET /repos/octocat/hello/issues/7` → `200` issue body with
  `state="open"`, no `pull_request`; mount the repo call. Assert `state == ActivityState::Open`,
  `kind == ActivityKind::Issue`, `merged_at.is_none()`, `additions.is_none()`.
- `github_404` — mount `GET /repos/octocat/ghost/pulls/999` → `404`. Assert
  `matches!(err, ForgeError::NotFound { .. })`.
- `github_rate_limited` — mount `GET /repos/octocat/hello/pulls/1` → `403` with header
  `x-ratelimit-remaining: 0` and `x-ratelimit-reset: <future epoch>`. Assert
  `matches!(err, ForgeError::RateLimited { retry_after: Some(_), .. })`.

`crates/forge/tests/codeberg.rs`:
- `codeberg_pr` — mount `GET /api/v1/repos/forgejo/forgejo/pulls/8326` → `200` Forgejo PR body
  (`state="closed"`, `merged=true`, `merged_at` set, `additions`/`deletions`); mount
  `GET /api/v1/repos/forgejo/forgejo` → `200` `{"stars_count": 100}`. Assert
  `state == ActivityState::Merged`, `repo_stars == Some(100)`, `forge == Forge::Codeberg`.
- (Optionally add `codeberg_404` and `codeberg_rate_limited` mirroring GitHub — the brief requires
  at minimum: GitHub PR merged, GitHub issue, Codeberg PR, a 404, a rate-limited response. The four
  GitHub tests plus `codeberg_pr` already satisfy the five required cases; add the Codeberg
  variants if cheap.)

wiremock test skeleton:

```rust
use plinth_forge::{ActivityRef, CodebergClient, ForgeClient, ForgeError, GitHubClient};
use plinth_shared::{ActivityKind, ActivityState, Forge};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn github_pr_merged() {
    let server = MockServer::start().await;
    Mock::given(method("GET")).and(path("/repos/octocat/hello/pulls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "title": "Fix the thing", "body": "details", "state": "closed",
            "merged": true, "merged_at": "2026-01-02T03:04:05Z",
            "created_at": "2026-01-01T00:00:00Z", "closed_at": "2026-01-02T03:04:05Z",
            "additions": 10, "deletions": 2, "comments": 3,
            "labels": [{"name":"bug"}], "html_url": "https://github.com/octocat/hello/pull/1"
        })))
        .mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/octocat/hello"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "stargazers_count": 42
        })))
        .mount(&server).await;

    let client = GitHubClient::with_base_url(server.uri(), None);
    let r = ActivityRef { forge: Forge::GitHub, owner: "octocat".into(), repo: "hello".into(),
                          kind: ActivityKind::PullRequest, number: 1 };
    let got = client.fetch(&r).await.expect("fetch ok");
    assert_eq!(got.forge, Forge::GitHub);
    assert_eq!(got.state, ActivityState::Merged);
    assert!(got.merged_at.is_some());
    assert_eq!(got.additions, Some(10));
    assert_eq!(got.repo_stars, Some(42));
}
```

(For `set_body_json` to work, the dev-dep `serde_json` must be present — it is, in step 2.
`ResponseTemplate::new(403).insert_header("x-ratelimit-remaining", "0")` sets headers for the
rate-limit test.)

### 9. Add the crate to the Nix source filter

Edit `/data/nvme0/can/Projects/solo/plinth/flake.nix` — inside the `src = lib.fileset.toSource {
... fileset = lib.fileset.unions [ ... ] }` block (around lines 211–237), add, mirroring the four
existing `maybeMissing` lines:

```nix
(lib.fileset.maybeMissing ./crates/forge)
```

Without this, the new crate's sources are filtered out of the Nix sandbox and `nix flake check`
fails to find the crate. (`craneLib.fileset.commonCargoSources` catches `Cargo.toml` + `.rs`, but
the explicit line is the established convention and is required for any non-Rust files; mirror the
existing entries.)

### 10. Build and verify

```bash
cargo build -p plinth-forge
cargo clippy -p plinth-forge --all-targets -- --deny warnings
cargo test -p plinth-forge
# WASM-isolation check: client must NOT pull plinth-forge (and thus reqwest):
cargo tree -p plinth-client -e features 2>/dev/null | grep -i plinth-forge && echo "LEAK" || echo "OK: no forge in client"
```

## Acceptance criteria

- [ ] `cargo build -p plinth-forge` succeeds with no errors.
- [ ] `cargo clippy -p plinth-forge --all-targets -- --deny warnings` succeeds with **zero
      warnings** (warnings are hard errors in `flake.nix`'s `plinth-clippy` check).
- [ ] `cargo test -p plinth-forge` passes, and the run includes these named tests, all green:
      `github_pr_merged`, `github_issue`, `github_404`, `github_rate_limited` (in
      `crates/forge/tests/github.rs`) and `codeberg_pr` (in `crates/forge/tests/codeberg.rs`).
- [ ] `github_pr_merged` asserts the normalized DTO has `state == ActivityState::Merged`,
      `merged_at.is_some()`, `additions == Some(10)`, `repo_stars == Some(42)`,
      `kind == ActivityKind::PullRequest`, `forge == Forge::GitHub`.
- [ ] `github_issue` asserts `state == ActivityState::Open`, `kind == ActivityKind::Issue`,
      `merged_at.is_none()`, `additions.is_none()` (proving the issue endpoint is read correctly
      and diff stats are absent, not defaulted to 0).
- [ ] `github_404` asserts the error matches `ForgeError::NotFound { .. }`.
- [ ] `github_rate_limited` asserts the error matches
      `ForgeError::RateLimited { retry_after: Some(_), .. }` for a `403` carrying
      `x-ratelimit-remaining: 0` and a future `x-ratelimit-reset`.
- [ ] `codeberg_pr` asserts the Forgejo `stars_count` maps to `repo_stars == Some(100)`,
      `state == ActivityState::Merged`, and `forge == Forge::Codeberg` (proving the field-name
      divergence from GitHub's `stargazers_count` is handled).
- [ ] No test makes a real network request (all use a `wiremock::MockServer`); the suite passes
      offline / inside the Nix sandbox.
- [ ] `"crates/forge"` is in the workspace `members` list in
      `/data/nvme0/can/Projects/solo/plinth/Cargo.toml`.
- [ ] `flake.nix` `src` fileset includes `(lib.fileset.maybeMissing ./crates/forge)`.
- [ ] `cargo tree -p plinth-client -e features | grep -i plinth-forge` returns **nothing** (the
      client does NOT depend on `plinth-forge`, so `reqwest` stays out of the WASM build).
- [ ] `crates/forge/Cargo.toml` does NOT list `sqlx`, `pgvector`, or `fastembed` as dependencies
      (no persistence/embedding leakage into this crate).
- [ ] `ActivityRef` carries `forge: Forge` (plus `owner`, `repo`, `kind`, `number`), the
      `ForgeClient` trait has the single method `fetch(&self, &ActivityRef)` (no
      `fetch_one`/`fetch_pull_request`/`fetch_issue`), and `ForgeRouter { github, codeberg }`
      `impl ForgeClient` dispatches by `r.forge` — so production can wire it as
      `Arc<dyn ForgeClient>`.
- [ ] `ForgeError`'s variants are exactly `NotFound { forge: Forge, url, status }`,
      `RateLimited { forge: Forge, retry_after }`, `Http { forge: Forge, status, body }`,
      `Network(String)`, `Decode(String)` — `forge` is the shared `Forge` enum, not `&'static str`.

## Files likely touched

New:
- `crates/forge/Cargo.toml`
- `crates/forge/src/lib.rs`
- `crates/forge/src/error.rs`
- `crates/forge/src/github.rs`
- `crates/forge/src/codeberg.rs`
- `crates/forge/src/router.rs`
- `crates/forge/tests/github.rs`
- `crates/forge/tests/codeberg.rs`

Edited (tiny, additive):
- `/data/nvme0/can/Projects/solo/plinth/Cargo.toml` (add `"crates/forge"` to `members`)
- `/data/nvme0/can/Projects/solo/plinth/flake.nix` (add `(lib.fileset.maybeMissing ./crates/forge)`
  to the `src` fileset unions)

Consumed (read-only, must already exist from Phase 01):
- `/data/nvme0/can/Projects/solo/plinth/crates/shared/src/` — `FetchedActivity`, `Forge`,
  `ActivityKind`, `ActivityState` behind the `brick-activity` feature.

## Pitfalls

- **GitHub `/issues/{n}` has no `merged`/`additions`/`deletions`.** Symptom: PRs show as
  `Open`/`Closed` never `Merged`, and diff stats are always `None`. Cause: fetching a PR through
  the issues endpoint. Recovery: route by `ActivityRef.kind` — PRs go to `/pulls/{n}`. Only true
  issues use `/issues/{n}`, where `merged_at` (if it's a PR-flavored issue) lives under
  `pull_request.merged_at`.
- **Two different stars field names.** Symptom: `repo_stars` is always `None` for one forge. Cause:
  GitHub uses `stargazers_count`, Forgejo uses `stars_count`. Recovery: separate serde structs per
  forge (`GhRepo` vs `FjRepo`). Do not share one struct.
- **GitHub 403 vs 429.** Symptom: rate-limit errors surface as generic `Http` instead of
  `RateLimited`. Cause: GitHub returns `403` (not always `429`) when the hourly quota (60/hr
  unauthenticated) is exhausted. Recovery: treat `403` with `x-ratelimit-remaining: 0` as
  `RateLimited`; treat plain `429` as `RateLimited` unconditionally.
- **Codeberg has no rate-limit headers.** Symptom: code waits on a header that never arrives.
  Cause: Codeberg/HAProxy exposes no `X-RateLimit-*` and not always a `Retry-After`. Recovery: on
  Codeberg, key only off the `429` status; `retry_after` may be `None` (caller does fixed backoff).
- **Missing `User-Agent` on GitHub.** Symptom: every GitHub request 403s even unauthenticated.
  Cause: GitHub rejects requests without a `User-Agent`. Recovery: `.user_agent("plinth-forge")` on
  the client builder.
- **Pulling `plinth-shared` default features.** Symptom: `plinth-forge` compiles blog/portfolio/todo
  modules (and may fail if those need deps this crate lacks). Cause: forgetting
  `default-features = false` is the workspace default for `plinth-shared`, but you still must
  request only `brick-activity`. Recovery: `plinth-shared = { workspace = true, features =
  ["brick-activity"] }` — the workspace dep already sets `default-features = false`.
- **`plinth-forge` leaking into the client.** Symptom: WASM build pulls `reqwest`, bloats or fails.
  Cause: someone (a later phase) adds `plinth-forge` to `crates/client/Cargo.toml`. Recovery: never
  add it there; the `cargo tree` acceptance check guards this.
- **Real network in tests.** Symptom: tests hang or fail under `nix flake check`. Cause: hitting
  `api.github.com` directly. Recovery: every test uses `MockServer` + `with_base_url(server.uri(),
  ..)`; never construct a client with the real base URL in a test.
- **Date parse failures on Forgejo offsets.** Symptom: `Decode` error on Codeberg responses. Cause:
  Forgejo emits `+02:00`-style offsets, not `Z`. Recovery: `chrono::DateTime<Utc>` with serde
  parses RFC3339-with-offset and normalizes to UTC — no custom deserializer needed; just ensure the
  field type is `DateTime<Utc>` (not `NaiveDateTime`).

## Risk profile

The blast radius of a bug here is the entire feature: every surface (the `/activity` page, the
home strip, the feed, semantic search results) reads `activity_items` whose forge-sourced fields
originate **only** from this crate. A wrong `merged_at` silently corrupts the ranking
(`reference_date = coalesce(merged_at, closed_at, created_at)` and the exponential/linear decay all
key off it). A swallowed 404 leaves deleted-upstream items live. A mis-classified rate-limit error
turns the refresh actor's backoff into a hot retry loop (Phase 04). The risk is concentrated in
correctness of normalization and error typing — not in volume of code. Likelihood of a subtle bug
is high (two divergent APIs, one of which — GitHub's issue/PR split — actively misleads); detection
is good *if* the named tests assert the specific normalized fields (not just "fetch returns Ok").

## Strategy

1. Land Phase 01's shared types first (or confirm they exist) — `FetchedActivity`, `Forge`,
   `ActivityKind`, `ActivityState`. Read them; do not redefine. The DTO contract is the spec.
2. Build error + trait + GitHub path first; write `github_pr_merged` and `github_issue`
   immediately and iterate normalization against those mocks (test-first on the trickiest forge).
3. Add the GitHub error paths (`github_404`, `github_rate_limited`) before touching Codeberg — get
   the status→error mapping right once, then reuse the helper.
4. Port to Codeberg, copying the structure; the *only* deltas are: base URL, `token `-prefix auth
   header, `stars_count` field name, no rate-limit headers, no version header. Write `codeberg_pr`.
5. Run clippy with `--deny warnings` early and often — the Nix check is unforgiving; a single
   `unused import` blocks the whole workspace `nix flake check`.
6. Last: wire the workspace member + flake fileset line, then `cargo build -p plinth-forge` and the
   `cargo tree` isolation check.

Keep the two clients structurally parallel (same private helper names: `map_status`,
`retry_after_from`, `normalize_*`) so a reviewer can diff GitHub vs Codeberg and see exactly the
intended deltas — divergence beyond those five points is a smell.

## Rollback drill

This phase is purely additive — nothing existing depends on `plinth-forge` yet (Phase 04/05 add
the dependencies later). To roll back cleanly:

1. `git rm -r crates/forge` (removes the whole new crate).
2. Revert the one-line `members` addition in `/data/nvme0/can/Projects/solo/plinth/Cargo.toml`.
3. Revert the one-line `(lib.fileset.maybeMissing ./crates/forge)` addition in `flake.nix`.
4. `cargo build` (workspace) and `cargo clippy --workspace --all-targets -- --deny warnings` to
   confirm the workspace is green without the crate. Because no other crate imports `plinth-forge`,
   removal cannot break server/client/cli/shared compilation. If a later phase has already added a
   `plinth-forge` dependency, that phase must be rolled back first — but within Wave 1 this crate
   stands alone.

There is no DB migration, no running service, and no persisted state to undo — rollback is a pure
source revert with zero data risk.

## Failure modes and recoveries

- **F1 — Phase 01 types absent or differently shaped.** Symptom: `plinth-forge` won't compile —
  `unresolved import plinth_shared::FetchedActivity` or field-name/type mismatches in
  normalization. Cause: Phase 01 not landed, or its DTO fields differ from the contract inlined in
  step 5. Recovery: do not redefine the types in this crate. Read the actual definitions in
  `crates/shared/src/` and adapt the normalization. If Phase 01 is missing entirely, stop and
  rebase onto it — this crate's reason for existing is to populate *those* types.
- **F2 — clippy fails the workspace check on a warning.** Symptom: `nix flake check` /
  `plinth-clippy` red; `cargo build` was green. Cause: `--deny warnings` promotes unused
  imports/`Result`-must-use/`needless_return` to errors. Recovery: run `cargo clippy -p
  plinth-forge --all-targets -- --deny warnings` locally and fix each; common offenders are unused
  serde fields (prefix with `#[allow(dead_code)]` only if genuinely unused, else read them),
  unused `serde_json` import in a test, and `clippy::large_enum_variant` on `ForgeError` (box a
  large variant if flagged).
- **F3 — wiremock not matching, test sees a 404 from the mock server.** Symptom: a `200` test
  unexpectedly returns `NotFound`. Cause: the mounted `path(...)` doesn't match the client's actual
  request path (e.g. you mounted `/pulls/1` but the client built `/repos/o/r/pulls/1`, or the repo
  call path is wrong). Recovery: mock the **full** path the client requests
  (`/repos/{owner}/{repo}/pulls/{n}` and the repo `/repos/{owner}/{repo}`); for Codeberg include
  the `/api/v1` prefix. Temporarily log `server.received_requests().await` to see the exact path
  the client sent.
- **F4 — real network leak in a test.** Symptom: test hangs ~30s then fails with a connection
  error under `nix flake check` (no internet). Cause: a client constructed with the real base URL
  (`GitHubClient::new(None)` instead of `with_base_url(server.uri(), None)`). Recovery: grep tests
  for `::new(` and replace with `with_base_url(server.uri(), ..)`; the default base URL must never
  appear in a test.
- **F5 — `reqwest` reaches the WASM client.** Symptom: WASM build (`cargo leptos build`) fails or
  bloats; `cargo tree -p plinth-client` shows `plinth-forge`/`reqwest`. Cause: a dependency edge
  from `plinth-client` to `plinth-forge`. Recovery: there must be none — `plinth-forge` is consumed
  only by `plinth-server` and `plinth-cli` (in later phases). Remove any such edge; the acceptance
  `cargo tree` check is the gate.
- **F6 — `additions`/`deletions` defaulted to 0 instead of `None`.** Symptom: issues report
  `additions = Some(0)`; ranking/UI can't distinguish "no diff stats" from "zero-line change".
  Cause: declaring the serde field as `i32` (non-optional) with `#[serde(default)]`. Recovery: keep
  these `Option<i32>` with no `default` so an absent field deserializes to `None`; `github_issue`
  asserts `additions.is_none()` precisely to catch this.

## Reference

Sequencing only (do not pull execution content from these — everything needed is inlined above):

- **Phase 01** (`./01-shared-types-and-migration.md`) must land first — it defines
  `plinth_shared::{FetchedActivity, Forge, ActivityKind, ActivityState}` behind `brick-activity`,
  which this crate normalizes into. If absent, rebase onto it.
- **Phase 03** (`./03-server-brick-core.md`) runs in parallel (Wave 1), disjoint files
  (`crates/server/src/bricks/activity/**`). No file overlap except the workspace `Cargo.toml`
  `members` list (trivial merge).
- **Phase 04** (`./04-lazy-refresh-actor.md`) consumes this crate's `ForgeClient` in the refresh
  actor; it adds the `plinth-forge` dependency to `crates/server/Cargo.toml` — not this phase.
- **Phase 05** (`./05-cli-commands.md`) consumes this crate in `plinth activity add`; it adds the
  dependency to `crates/cli/Cargo.toml` — not this phase.
- Design brief: the "Forge Activity" feature spec (forge API facts: R6; mocking dep + sandbox
  constraints: R5; CLI `ApiClient` builder idiom: R2 / `crates/cli/src/api_client.rs:53-62`).
