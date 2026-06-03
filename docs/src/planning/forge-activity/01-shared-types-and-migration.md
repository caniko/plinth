# Phase 01 — Shared types and the activity migration

> **Recommended Codex model: GPT 5.5 medium**
>
> This is a moderate but load-bearing foundation phase: it defines the WASM-safe wire
> contract (six enums/structs) and the canonical Postgres schema that all seven downstream
> phases consume. The work is mostly pattern-matching against the existing portfolio/blog
> bricks, so it does not need a high tier — but it is unforgiving of two specific mistakes a
> too-small model makes: (1) leaking a non-WASM-safe dependency (`reqwest`, `sqlx`,
> `pgvector`) into `plinth-shared`, which silently breaks the `wasm32-unknown-unknown` client
> build that every page compiles into; and (2) getting the schema's natural key, `CHECK`, or
> `vector(384)`/HNSW shape wrong, which forces a painful migration rewrite once Phases 02–07
> have built on top. Medium gives enough care to copy the idioms exactly while keeping the
> blast radius understood.

## Working tree

- `cwd = /data/nvme0/can/Projects/solo/plinth` (the plinth repo).
- This is **Wave 0** — nothing else runs concurrently, so there are no pull/rebase
  serialization concerns for this phase. You start from a clean tree.
- You touch **only** `crates/shared/` and `crates/server/migrations/`. You do **not** touch
  any other crate, any server handler, the forge crate, the CLI, or the frontend — those are
  later phases that depend on the contract you produce here.
- Feature wiring: this phase introduces the `brick-activity` feature in `plinth-shared`'s
  `Cargo.toml` (a leaf `[]` marker) and adds it to that crate's `default`. Do **not** add
  `brick-activity` to the server/client/cli/workspace manifests yet — those crates do not
  reference the shared activity types until Phases 03/05/06, and adding the feature there now
  with no consumer would be dead wiring. (Downstream phases add their own feature chains.)

## Goal

This phase succeeds when `plinth-shared` exposes a complete, WASM-safe data contract for the
new **activity** brick — the `Forge`, `ActivityKind`, `ActivityState`, and `RankingStrategy`
enums; the `FetchedActivity` DTO (what the forge crate will return); the `ActivityItem` full
row type; the `ActivityListItem` ranked-list type carrying a computed `score: f64`; and the
`PublishActivityRequest` admin payload — plus a validator that enforces `impact ∈ 1..=10`,
non-empty `repo_owner`/`repo_name`, and `number > 0`, all covered by inline `#[cfg(test)]`
unit tests. In parallel, `crates/server/migrations/0006_activity.sql` creates the
`activity_items` table with the exact columns, the `url` UNIQUE constraint, the
`(forge, repo_owner, repo_name, kind, number)` natural-key UNIQUE constraint, the
`impact BETWEEN 1 AND 10` CHECK, an `embedding vector(384)` column with an HNSW
(`vector_cosine_ops`) index, and the `schema_migrations` ledger insert. `cargo build -p
plinth-shared` and `cargo build -p plinth-shared --target wasm32-unknown-unknown` both succeed
with no new dependencies leaking in; `cargo test -p plinth-shared` runs the new validation
tests green.

## Why this matters now

This is the trunk of the whole feature tree. Every sibling phase imports from here:

- **Phase 02 (forge crate)** returns `FetchedActivity` and switches on `Forge`/`ActivityKind`.
- **Phase 03 (server brick)** persists `ActivityItem`, accepts `PublishActivityRequest`, and
  serves `ActivityListItem` (with `score`); it also runs the migration you write.
- **Phase 05 (CLI)** builds a `PublishActivityRequest` and POSTs it.
- **Phase 06 (frontend)** renders `ActivityItem`/`ActivityListItem` in Leptos pages compiled
  to WASM — which is exactly why the types must stay WASM-safe.
- **Phase 07 (feed + search)** reads the `embedding` column and the same row type.

If the schema's natural key or the `vector(384)`/HNSW index is wrong, or a type drags in
`reqwest`/`sqlx`, the breakage surfaces three phases later in a far more expensive place (a
failed WASM build, or a migration that must be edited after data exists). Getting it right
once, here, is the cheapest possible point to be correct.

## Out of scope

- **No server code.** Do not create `crates/server/src/bricks/activity/` or any handler,
  cache actor, or `services/db.rs` upsert. That is Phase 03.
- **No forge crate.** Do not create `crates/forge/`. `FetchedActivity` is a *pure data* DTO
  here; the `ForgeClient` trait and reqwest clients are Phase 02.
- **No CLI / frontend.** No `crates/cli/`, no `crates/client/` edits, no routes, no server
  functions, no Cargo wiring in those crates.
- **No config sections.** `[ranking]` / `[forge]` config structs in `toml_config.rs` belong to
  Phases 03/04/07. You only define the `RankingStrategy` *enum* (a wire/value type), not the
  config plumbing.
- **No SQL ranking expression.** The read-time score SQL (`power(0.5, …)` etc.) is Phase 03.
  Here, `ActivityListItem.score` is just a plain `f64` field that Phase 03 will populate.
- **No new dependencies.** `plinth-shared` must keep its current deps (`serde`, `chrono`,
  optional `toml`). Adding `sqlx`, `reqwest`, `pgvector`, or `uuid` here is a hard failure.

## Plan

All file paths are absolute under `/data/nvme0/can/Projects/solo/plinth`.

### 1. Add the `brick-activity` leaf feature to `plinth-shared`

Edit `crates/shared/Cargo.toml`. The current `[features]` block is:

```toml
[features]
default = ["brick-blog", "brick-portfolio", "brick-todo"]
brick-blog = []
brick-portfolio = []
brick-todo = []
config-toml = ["dep:toml"]
```

Change it to add `brick-activity` as a leaf marker and to `default` (mirroring the other
bricks exactly — shared brick features are empty `[]` cfg markers, see `crates/shared/src/lib.rs`):

```toml
[features]
default = ["brick-blog", "brick-portfolio", "brick-todo", "brick-activity"]
brick-blog = []
brick-portfolio = []
brick-todo = []
brick-activity = []
config-toml = ["dep:toml"]
```

Do **not** add any new entries under `[dependencies]`.

### 2. Create the shared module `crates/shared/src/activity_item.rs`

This is the only new source file. It mirrors the conventions in
`crates/shared/src/portfolio_item.rs` (the canonical template): `use chrono::{DateTime, Utc};`,
`use serde::{Deserialize, Serialize};`, `#[serde(skip_serializing_if = "Option::is_none")]`
on optionals, and an inline `#[cfg(test)] mod tests`. Everything is `Serialize + Deserialize`
so it round-trips over the admin API and the Leptos server-function boundary.

**One deliberate divergence from blog/portfolio:** activity has NO slug — it is identified and
routed entirely by its numeric `BIGSERIAL` primary key. So the `id` on `ActivityItem` /
`ActivityListItem` is a plain `i64`, **not** the `Option<String>` + `deserialize_flexible_id`
wrapper the slug-bearing bricks use. Do **not** import `deserialize_flexible_id` here.

Each of `Forge`, `ActivityKind`, and `ActivityState` provides both `as_str(self) -> &'static str`
and an `impl std::str::FromStr` (with `Err = ParseEnumError`), so the Phase 03 server row decoder
can call `row.try_get::<String, _>(col)?.parse()?` on the `forge`/`kind`/`state` TEXT columns.
Inline round-trip tests pair `as_str()` against `parse()` for each.

Write the file with exactly this content:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Error returned when a DB/wire token cannot be parsed back into an enum.
///
/// Used by the `FromStr` impls so the server row decoder (Phase 03) can call
/// `s.parse()` on the `forge`/`kind`/`state` TEXT columns and surface a typed error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseEnumError {
    /// The enum being parsed (e.g. `"Forge"`), for a clear message.
    pub kind: &'static str,
    /// The unrecognized token that was provided.
    pub value: String,
}

impl core::fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "invalid {} token: {:?}", self.kind, self.value)
    }
}

impl std::error::Error for ParseEnumError {}

/// The code forge a contribution lives on.
///
/// Serialized lowercase to match the `forge TEXT` column values
/// (`'github'` | `'codeberg'`) used by the `activity_items` table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Forge {
    GitHub,
    Codeberg,
}

impl Forge {
    /// Lowercase wire/DB token (`"github"` | `"codeberg"`).
    pub fn as_str(self) -> &'static str {
        match self {
            Forge::GitHub => "github",
            Forge::Codeberg => "codeberg",
        }
    }
}

impl std::str::FromStr for Forge {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "github" => Ok(Forge::GitHub),
            "codeberg" => Ok(Forge::Codeberg),
            other => Err(ParseEnumError {
                kind: "Forge",
                value: other.to_string(),
            }),
        }
    }
}

/// Whether a contribution is a pull request or an issue.
///
/// Serialized as the short DB tokens `'pr'` | `'issue'`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivityKind {
    #[serde(rename = "pr")]
    PullRequest,
    Issue,
}

impl ActivityKind {
    /// DB token (`"pr"` | `"issue"`).
    pub fn as_str(self) -> &'static str {
        match self {
            ActivityKind::PullRequest => "pr",
            ActivityKind::Issue => "issue",
        }
    }
}

impl std::str::FromStr for ActivityKind {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pr" => Ok(ActivityKind::PullRequest),
            "issue" => Ok(ActivityKind::Issue),
            other => Err(ParseEnumError {
                kind: "ActivityKind",
                value: other.to_string(),
            }),
        }
    }
}

/// Lifecycle state of a contribution.
///
/// `Merged` is a derived state (a PR that is closed with a merge timestamp);
/// neither forge reports `"merged"` directly — Phase 02 derives it.
/// Serialized as `'open'` | `'closed'` | `'merged'`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivityState {
    Open,
    Closed,
    Merged,
}

impl ActivityState {
    /// DB token (`"open"` | `"closed"` | `"merged"`).
    pub fn as_str(self) -> &'static str {
        match self {
            ActivityState::Open => "open",
            ActivityState::Closed => "closed",
            ActivityState::Merged => "merged",
        }
    }
}

impl std::str::FromStr for ActivityState {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open" => Ok(ActivityState::Open),
            "closed" => Ok(ActivityState::Closed),
            "merged" => Ok(ActivityState::Merged),
            other => Err(ParseEnumError {
                kind: "ActivityState",
                value: other.to_string(),
            }),
        }
    }
}

/// Ranking strategy selecting how `score` is computed at read time (Phase 03).
///
/// `Exponential` is the default. The score SQL is NOT defined here — this enum
/// only names the strategy; Phase 03 threads it into the read query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RankingStrategy {
    /// impact * power(0.5, age_days / half_life_days)
    #[default]
    Exponential,
    /// impact * greatest(0, 1 - age_days / window_days)
    Linear,
    /// impact (recency only as a tiebreaker)
    Pure,
}

impl RankingStrategy {
    /// Lowercase config/wire token.
    pub fn as_str(self) -> &'static str {
        match self {
            RankingStrategy::Exponential => "exponential",
            RankingStrategy::Linear => "linear",
            RankingStrategy::Pure => "pure",
        }
    }
}

/// Pure, WASM-safe DTO describing a single contribution as fetched from a forge.
///
/// This is what the `plinth-forge` crate (Phase 02) returns and what the CLI
/// (Phase 05) / server refresh (Phase 04) normalize GitHub and Forgejo payloads
/// into. It carries NO transport state (no reqwest, no headers) — it is just data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FetchedActivity {
    pub forge: Forge,
    pub repo_owner: String,
    pub repo_name: String,
    pub kind: ActivityKind,
    pub number: i32,
    pub url: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub state: ActivityState,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additions: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletions: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comments_count: Option<i32>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_stars: Option<i32>,
}

/// A fully persisted activity row (mirrors the `activity_items` table).
///
/// NOTE: unlike the blog/portfolio bricks, activity has NO slug and routes by its
/// numeric primary key, so `id` is a plain `i64` (the `BIGSERIAL` PK) — there is no
/// `deserialize_flexible_id`/`Option<String>` wrapper here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityItem {
    /// Database record ID (the `BIGSERIAL` primary key).
    pub id: i64,

    pub forge: Forge,
    pub repo_owner: String,
    pub repo_name: String,
    pub kind: ActivityKind,
    pub number: i32,
    pub url: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub state: ActivityState,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_at: Option<DateTime<Utc>>,

    /// Curated impact weight, 1..=10 (mirrors the SMALLINT CHECK).
    pub impact: i16,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additions: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletions: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comments_count: Option<i32>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_stars: Option<i32>,

    /// Snapshot/refresh time; drives the stale-while-revalidate TTL (Phase 04).
    pub fetched_at: DateTime<Utc>,

    #[serde(default)]
    pub featured: bool,
    #[serde(default = "default_true")]
    pub published: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
}

/// A list/grid projection plus the read-time computed ranking score.
///
/// `score` is computed in SQL (Phase 03) and never stored; here it is a plain
/// `f64` field the server populates from the ranking query. Like `ActivityItem`,
/// `id` is a plain `i64` (numeric routing; no slug, no flexible-string wrapper).
/// There is NO stored `reference_date` column/field — it is ALWAYS derived from
/// `created_at`/`closed_at`/`merged_at` via the `reference_date()` helper below.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityListItem {
    /// Database record ID (the `BIGSERIAL` primary key).
    pub id: i64,
    pub forge: Forge,
    pub repo_owner: String,
    pub repo_name: String,
    pub kind: ActivityKind,
    pub number: i32,
    pub url: String,
    pub title: String,
    pub state: ActivityState,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_at: Option<DateTime<Utc>>,
    pub impact: i16,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub featured: bool,
    /// Read-time computed ranking score (impact x recency). Never stored.
    #[serde(default)]
    pub score: f64,
}

impl ActivityListItem {
    /// The date used for recency in the ranking: prefer `merged_at`, then
    /// `closed_at`, falling back to `created_at`. There is no stored
    /// `reference_date` — it is ALWAYS derived here.
    pub fn reference_date(&self) -> DateTime<Utc> {
        self.merged_at.or(self.closed_at).unwrap_or(self.created_at)
    }
}

/// Admin upsert payload (Bearer auth). Upserted by the UNIQUE natural key
/// `(forge, repo_owner, repo_name, kind, number)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishActivityRequest {
    pub forge: Forge,
    pub repo_owner: String,
    pub repo_name: String,
    pub kind: ActivityKind,
    pub number: i32,
    pub url: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    pub state: ActivityState,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged_at: Option<DateTime<Utc>>,

    /// Curated impact weight, 1..=10. Defaults to 1.
    #[serde(default = "default_impact")]
    pub impact: i16,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additions: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletions: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comments_count: Option<i32>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_stars: Option<i32>,

    /// 384-dim embedding generated by the CLI (Phase 05). Server does NOT embed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,

    #[serde(default)]
    pub featured: bool,
    #[serde(default = "default_true")]
    pub published: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_impact() -> i16 {
    1
}

/// Validation error for an activity payload (WASM-safe, no external error crate).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityValidationError {
    ImpactOutOfRange(i16),
    EmptyRepoOwner,
    EmptyRepoName,
    NonPositiveNumber(i32),
}

impl core::fmt::Display for ActivityValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ActivityValidationError::ImpactOutOfRange(v) => {
                write!(f, "impact must be between 1 and 10, got {v}")
            }
            ActivityValidationError::EmptyRepoOwner => write!(f, "repo_owner must not be empty"),
            ActivityValidationError::EmptyRepoName => write!(f, "repo_name must not be empty"),
            ActivityValidationError::NonPositiveNumber(v) => {
                write!(f, "number must be greater than 0, got {v}")
            }
        }
    }
}

impl PublishActivityRequest {
    /// Validate the curated fields: impact 1..=10, non-empty owner/name, number > 0.
    pub fn validate(&self) -> Result<(), ActivityValidationError> {
        validate_activity_fields(self.impact, &self.repo_owner, &self.repo_name, self.number)
    }
}

/// Shared validation used by both the CLI add path and the server admin handler.
pub fn validate_activity_fields(
    impact: i16,
    repo_owner: &str,
    repo_name: &str,
    number: i32,
) -> Result<(), ActivityValidationError> {
    if !(1..=10).contains(&impact) {
        return Err(ActivityValidationError::ImpactOutOfRange(impact));
    }
    if repo_owner.trim().is_empty() {
        return Err(ActivityValidationError::EmptyRepoOwner);
    }
    if repo_name.trim().is_empty() {
        return Err(ActivityValidationError::EmptyRepoName);
    }
    if number <= 0 {
        return Err(ActivityValidationError::NonPositiveNumber(number));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request() -> PublishActivityRequest {
        PublishActivityRequest {
            forge: Forge::GitHub,
            repo_owner: "cli".to_string(),
            repo_name: "cli".to_string(),
            kind: ActivityKind::PullRequest,
            number: 9000,
            url: "https://github.com/cli/cli/pull/9000".to_string(),
            title: "Fix a thing".to_string(),
            body: Some("body".to_string()),
            state: ActivityState::Merged,
            created_at: Utc::now(),
            closed_at: None,
            merged_at: Some(Utc::now()),
            impact: 5,
            additions: Some(10),
            deletions: Some(2),
            comments_count: Some(3),
            labels: vec!["bug".to_string()],
            repo_stars: Some(1234),
            embedding: None,
            featured: false,
            published: true,
            content_hash: None,
        }
    }

    #[test]
    fn test_validate_accepts_valid_request() {
        assert_eq!(valid_request().validate(), Ok(()));
    }

    #[test]
    fn test_validate_rejects_impact_below_range() {
        let mut r = valid_request();
        r.impact = 0;
        assert_eq!(r.validate(), Err(ActivityValidationError::ImpactOutOfRange(0)));
    }

    #[test]
    fn test_validate_rejects_impact_above_range() {
        let mut r = valid_request();
        r.impact = 11;
        assert_eq!(
            r.validate(),
            Err(ActivityValidationError::ImpactOutOfRange(11))
        );
    }

    #[test]
    fn test_validate_accepts_impact_boundaries() {
        assert_eq!(validate_activity_fields(1, "o", "n", 1), Ok(()));
        assert_eq!(validate_activity_fields(10, "o", "n", 1), Ok(()));
    }

    #[test]
    fn test_validate_rejects_empty_repo_owner() {
        let mut r = valid_request();
        r.repo_owner = "   ".to_string();
        assert_eq!(r.validate(), Err(ActivityValidationError::EmptyRepoOwner));
    }

    #[test]
    fn test_validate_rejects_empty_repo_name() {
        let mut r = valid_request();
        r.repo_name = String::new();
        assert_eq!(r.validate(), Err(ActivityValidationError::EmptyRepoName));
    }

    #[test]
    fn test_validate_rejects_non_positive_number() {
        let mut r = valid_request();
        r.number = 0;
        assert_eq!(
            r.validate(),
            Err(ActivityValidationError::NonPositiveNumber(0))
        );
        r.number = -3;
        assert_eq!(
            r.validate(),
            Err(ActivityValidationError::NonPositiveNumber(-3))
        );
    }

    #[test]
    fn test_default_impact_is_one() {
        assert_eq!(default_impact(), 1);
    }

    #[test]
    fn test_ranking_strategy_default_is_exponential() {
        assert_eq!(RankingStrategy::default(), RankingStrategy::Exponential);
    }

    #[test]
    fn test_enum_wire_tokens() {
        assert_eq!(Forge::GitHub.as_str(), "github");
        assert_eq!(Forge::Codeberg.as_str(), "codeberg");
        assert_eq!(ActivityKind::PullRequest.as_str(), "pr");
        assert_eq!(ActivityKind::Issue.as_str(), "issue");
        assert_eq!(ActivityState::Merged.as_str(), "merged");
        assert_eq!(RankingStrategy::Linear.as_str(), "linear");
    }

    #[test]
    fn test_forge_str_round_trip() {
        for v in [Forge::GitHub, Forge::Codeberg] {
            assert_eq!(v.as_str().parse::<Forge>(), Ok(v));
        }
        assert!("gitlab".parse::<Forge>().is_err());
    }

    #[test]
    fn test_activity_kind_str_round_trip() {
        for v in [ActivityKind::PullRequest, ActivityKind::Issue] {
            assert_eq!(v.as_str().parse::<ActivityKind>(), Ok(v));
        }
        assert!("commit".parse::<ActivityKind>().is_err());
    }

    #[test]
    fn test_activity_state_str_round_trip() {
        for v in [ActivityState::Open, ActivityState::Closed, ActivityState::Merged] {
            assert_eq!(v.as_str().parse::<ActivityState>(), Ok(v));
        }
        assert!("draft".parse::<ActivityState>().is_err());
    }

    #[test]
    fn test_kind_serde_uses_short_tokens() {
        // PullRequest serializes to "pr", not "pullrequest".
        assert_eq!(
            serde_json::to_string(&ActivityKind::PullRequest).unwrap(),
            "\"pr\""
        );
        assert_eq!(
            serde_json::from_str::<ActivityKind>("\"issue\"").unwrap(),
            ActivityKind::Issue
        );
    }
}
```

Notes for the executing agent:
- `serde_json` is already a `dev-dependency` of `plinth-shared` (see `crates/shared/Cargo.toml`),
  so the `test_kind_serde_uses_short_tokens` test compiles without adding a dependency.
- Integer widths are chosen to match the SQL columns and the future sqlx row decoder
  (`i16` ⇔ `SMALLINT`, `i32` ⇔ `INTEGER`); do not use `u*` types — Postgres has no unsigned
  integers and sqlx will not bind them.

### 3. Register the module in `crates/shared/src/lib.rs`

Add the gated module declaration and re-export, mirroring the `portfolio_item` lines exactly.

Add to the module-declaration block (next to `#[cfg(feature = "brick-portfolio")] pub mod portfolio_item;`):

```rust
#[cfg(feature = "brick-activity")]
pub mod activity_item;
```

Add to the re-export block (next to the portfolio `pub use`):

```rust
#[cfg(feature = "brick-activity")]
pub use activity_item::{
    ActivityItem, ActivityKind, ActivityListItem, ActivityState, ActivityValidationError,
    FetchedActivity, Forge, ParseEnumError, PublishActivityRequest, RankingStrategy,
    validate_activity_fields,
};
```

### 4. Write the migration `crates/server/migrations/0006_activity.sql`

Match the exact style of `crates/server/migrations/0004_portfolio.sql` and the
`embedding`/HNSW lines from `0003_blog.sql`. The `vector` extension is already created in
`0001_init.sql`, so do not re-create it. End the file with the `schema_migrations` ledger
insert (the migration runner uses a parallel `schema_migrations` ledger for status reporting,
distinct from sqlx's `_sqlx_migrations`). Write exactly:

```sql
-- Activity brick: curated external contributions (PRs/issues) across forges,
-- ranked by impact x recency at read time. The `vector` extension is created
-- in 0001_init.sql. EMBEDDING_DIM = 384; must match fastembed::AllMiniLML6V2
-- and blog_posts.embedding.

CREATE TABLE activity_items (
    id BIGSERIAL PRIMARY KEY,
    forge TEXT NOT NULL,                         -- 'github' | 'codeberg'
    repo_owner TEXT NOT NULL,
    repo_name TEXT NOT NULL,
    kind TEXT NOT NULL,                          -- 'pr' | 'issue'
    number INTEGER NOT NULL,
    url TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    body TEXT,
    state TEXT NOT NULL,                         -- 'open' | 'closed' | 'merged'
    created_at TIMESTAMPTZ NOT NULL,
    closed_at TIMESTAMPTZ,
    merged_at TIMESTAMPTZ,
    impact SMALLINT NOT NULL DEFAULT 1 CHECK (impact BETWEEN 1 AND 10),
    additions INTEGER,
    deletions INTEGER,
    comments_count INTEGER,
    labels TEXT[] DEFAULT '{}',
    repo_stars INTEGER,
    embedding vector(384),
    fetched_at TIMESTAMPTZ NOT NULL,             -- snapshot/refresh time; drives the TTL
    featured BOOLEAN NOT NULL DEFAULT false,
    published BOOLEAN NOT NULL DEFAULT true,
    content_hash TEXT,
    CONSTRAINT activity_items_natural_key UNIQUE (forge, repo_owner, repo_name, kind, number)
);

CREATE INDEX activity_items_state_idx ON activity_items (state);
CREATE INDEX activity_items_featured_idx ON activity_items (featured);
CREATE INDEX activity_items_published_idx ON activity_items (published);
CREATE INDEX activity_items_labels_idx ON activity_items USING gin (labels);
CREATE INDEX activity_items_embedding_hnsw_idx
    ON activity_items USING hnsw (embedding vector_cosine_ops);

INSERT INTO schema_migrations (brick, version, name)
VALUES ('activity', 1, 'initial_activity_schema');
```

Notes:
- `url TEXT NOT NULL UNIQUE` provides the inline unique constraint required by the brief;
  the named composite constraint `activity_items_natural_key` is the upsert target Phase 03
  uses (`ON CONFLICT (forge, repo_owner, repo_name, kind, number)`).
- `fetched_at` is **server-stamped**, not client-supplied. `PublishActivityRequest` carries
  **no** `fetched_at` field; the Phase 03 admin upsert handler sets
  `fetched_at = chrono::Utc::now()` on insert, and Phase 04 re-stamps it on each refresh. The
  column is `NOT NULL` because the server always provides a value — the request never does.
- The HNSW index uses `vector_cosine_ops`, exactly matching `blog_posts_embedding_hnsw_idx`
  in `0003_blog.sql`, so the cosine `<=>` operator Phase 07's search union relies on uses the
  index automatically.
- Do **not** add a stored `score` column — ranking is computed at read time (Phase 03).

### 5. Build and test

Run, from the repo root, with the binary-target restriction so you only build the library
crate under test (the workspace's default build pulls in `cargo-leptos`/WASM tooling not
needed here):

```bash
cargo build -p plinth-shared
cargo build -p plinth-shared --target wasm32-unknown-unknown
cargo test -p plinth-shared
```

If `wasm32-unknown-unknown` is not installed in the toolchain, add it with
`rustup target add wasm32-unknown-unknown` (the project already targets it for the client —
see `flake.nix` toolchain targets). The WASM build is the load-bearing check that no
non-WASM-safe dependency leaked in.

## Acceptance criteria

- [ ] `crates/shared/src/activity_item.rs` exists and defines: `Forge`, `ActivityKind`,
      `ActivityState`, `RankingStrategy`, `FetchedActivity`, `ActivityItem`, `ActivityListItem`,
      `PublishActivityRequest`, `ActivityValidationError`, `ParseEnumError`, and
      `validate_activity_fields`.
- [ ] `ActivityListItem` has a `pub score: f64` field, and `ActivityItem` has a `pub impact: i16`
      field — verifiable with
      `grep -n "pub score: f64" crates/shared/src/activity_item.rs` and
      `grep -n "pub impact: i16" crates/shared/src/activity_item.rs`.
- [ ] `ActivityItem.id` and `ActivityListItem.id` are both `pub id: i64` (numeric routing; no
      slug, no `deserialize_flexible_id`) —
      `grep -n "pub id: i64" crates/shared/src/activity_item.rs` shows two occurrences and
      `grep -n "deserialize_flexible_id" crates/shared/src/activity_item.rs` returns nothing.
- [ ] `ActivityListItem` has NO stored `reference_date` field; it exposes a
      `pub fn reference_date(&self) -> DateTime<Utc>` helper and a `pub labels: Vec<String>`
      field — `grep -n "fn reference_date" crates/shared/src/activity_item.rs` shows the method
      and `grep -n "pub reference_date" crates/shared/src/activity_item.rs` returns nothing.
- [ ] `Forge`, `ActivityKind`, and `ActivityState` each `impl std::str::FromStr` with
      `type Err = ParseEnumError` —
      `grep -n "impl std::str::FromStr" crates/shared/src/activity_item.rs` shows three impls.
- [ ] `PublishActivityRequest` has NO `fetched_at` field —
      `grep -n "fetched_at" crates/shared/src/activity_item.rs` shows it only on `ActivityItem`,
      not on `PublishActivityRequest`.
- [ ] `crates/shared/src/lib.rs` declares `#[cfg(feature = "brick-activity")] pub mod activity_item;`
      and re-exports the types — `grep -n "activity_item" crates/shared/src/lib.rs` shows both lines.
- [ ] `crates/shared/Cargo.toml` lists `brick-activity = []` and includes `brick-activity` in
      `default` — `grep -n "brick-activity" crates/shared/Cargo.toml` shows two occurrences.
- [ ] `cargo build -p plinth-shared` succeeds with **0 warnings**.
- [ ] `cargo build -p plinth-shared --target wasm32-unknown-unknown` succeeds (proves WASM-safety;
      no `reqwest`/`sqlx`/`pgvector` leaked in). Confirm shared still has no such deps:
      `grep -E "reqwest|sqlx|pgvector" crates/shared/Cargo.toml` returns nothing.
- [ ] `cargo test -p plinth-shared` passes, and these named test functions run green:
      `test_validate_accepts_valid_request`, `test_validate_rejects_impact_below_range`,
      `test_validate_rejects_impact_above_range`, `test_validate_accepts_impact_boundaries`,
      `test_validate_rejects_empty_repo_owner`, `test_validate_rejects_empty_repo_name`,
      `test_validate_rejects_non_positive_number`, `test_default_impact_is_one`,
      `test_ranking_strategy_default_is_exponential`, `test_enum_wire_tokens`,
      `test_forge_str_round_trip`, `test_activity_kind_str_round_trip`,
      `test_activity_state_str_round_trip`, `test_kind_serde_uses_short_tokens`.
- [ ] `crates/server/migrations/0006_activity.sql` exists and contains: `CREATE TABLE activity_items`,
      `impact SMALLINT NOT NULL DEFAULT 1 CHECK (impact BETWEEN 1 AND 10)`,
      `embedding vector(384)`, `CONSTRAINT activity_items_natural_key UNIQUE (forge, repo_owner, repo_name, kind, number)`,
      `url TEXT NOT NULL UNIQUE`, an HNSW index using `vector_cosine_ops`, and the
      `INSERT INTO schema_migrations ... ('activity', 1, 'initial_activity_schema')` line.
      Verify with `grep -n "activity_items_natural_key\|vector(384)\|BETWEEN 1 AND 10\|hnsw" crates/server/migrations/0006_activity.sql`.
- [ ] The migration applies cleanly against Postgres 16 + pgvector — e.g. running the server's
      migration runner (or `psql -f`) against a fresh DB with the `vector` extension produces
      a table where `\d activity_items` shows the `CHECK`, the two UNIQUE constraints, and the
      `activity_items_embedding_hnsw_idx` HNSW index. (Phase 03 exercises this via its
      `#[sqlx::test]` suite; this phase only requires the SQL to parse and apply.)
- [ ] No files outside `crates/shared/` and `crates/server/migrations/` are modified
      (`git status` shows only those paths plus this doc).

## Files likely touched

Shared crate (the contract):
- `crates/shared/src/activity_item.rs` — **new**: all enums, DTOs, request type, validator, tests.
- `crates/shared/src/lib.rs` — gated `pub mod activity_item;` + re-export block.
- `crates/shared/Cargo.toml` — add `brick-activity = []` leaf feature + add to `default`.

Server migrations (the schema):
- `crates/server/migrations/0006_activity.sql` — **new**: `activity_items` table + indexes +
  HNSW + `schema_migrations` insert.

## Pitfalls

- **Leaking a non-WASM-safe dependency.** *Symptom:* `cargo build -p plinth-shared --target
  wasm32-unknown-unknown` fails (often a `mio`/`socket2`/`tokio` net or `getrandom` error).
  *Cause:* adding `sqlx`, `reqwest`, `pgvector`, or `uuid` to `plinth-shared`, or importing
  one transitively. *Recovery:* keep `plinth-shared` deps exactly as they are (`serde`,
  `chrono`, optional `toml`); represent the embedding as `Option<Vec<f32>>` (not
  `pgvector::Vector`), the `id` as a plain `i64` (not a DB type, and — unlike the slug bricks —
  not a flexible `Option<String>`), and the state/kind as the plain enums above. The forge
  crate (Phase 02) owns reqwest; shared owns only data.

- **Wrong integer widths.** *Symptom:* Phase 03's sqlx row decoder fails to compile or panics
  binding `impact`. *Cause:* using `u8`/`u16`/`u32` — Postgres has no unsigned integers.
  *Recovery:* `SMALLINT` ⇒ `i16`, `INTEGER` ⇒ `i32`. The types above already do this.

- **Forgetting the natural-key constraint name / shape.** *Symptom:* Phase 03's
  `ON CONFLICT (forge, repo_owner, repo_name, kind, number)` upsert errors with "there is no
  unique or exclusion constraint matching the ON CONFLICT specification". *Cause:* omitting
  the composite UNIQUE, or listing columns in a different order/spelling. *Recovery:* keep
  `CONSTRAINT activity_items_natural_key UNIQUE (forge, repo_owner, repo_name, kind, number)`
  verbatim.

- **Re-creating the `vector` extension.** *Symptom:* migration ordering noise or a duplicate
  `CREATE EXTENSION`. *Cause:* copying the extension line from `0001_init.sql`. *Recovery:*
  `0001_init.sql` already runs `CREATE EXTENSION IF NOT EXISTS vector;` — `0006` must not.

- **Adding a stored `score` column.** *Symptom:* stale rankings / a redundant refresh job
  later. *Cause:* over-eager schema. *Recovery:* there is no `score` column; `score` exists
  only as the `f64` field on `ActivityListItem`, populated by Phase 03's read-time SQL.

- **Adding `brick-activity` to the wrong manifests.** *Symptom:* `cargo build` failures in
  server/client/cli for an unresolved feature or unused-feature churn. *Cause:* chaining
  `brick-activity` into other crates before they have any consumer. *Recovery:* in this phase,
  add `brick-activity` **only** to `crates/shared/Cargo.toml`. Phases 03/05/06 add their own
  feature chains (`brick-activity = ["plinth-shared/brick-activity", ...]`) when they
  introduce real consumers.

- **`enum` serde tokens not matching the DB.** *Symptom:* round-trip mismatches (`"pr"` vs
  `"pullrequest"`, `"github"` vs `"GitHub"`). *Cause:* missing `#[serde(rename_all =
  "lowercase")]` / the `#[serde(rename = "pr")]` on `PullRequest`. *Recovery:* keep the serde
  attributes above; the `test_kind_serde_uses_short_tokens` and `test_enum_wire_tokens` tests
  guard this.

## Reference

- **Design brief:** the "Forge Activity" brick — see the feature spec for the locked schema,
  endpoints, ranking strategies, and surfaces. This phase implements only the
  `01 shared-types-and-migration` slice of the locked phase ordering.
- **Sequencing (context only — do not pull content from siblings):**
  - Phase 02 (`./02-forge-crate.md`) consumes `FetchedActivity` + `Forge`/`ActivityKind`/
    `ActivityState` and derives `Merged` from forge payloads. Depends on this phase.
  - Phase 03 (`./03-server-brick-core.md`) runs `0006_activity.sql`, upserts via the
    natural-key constraint, populates `ActivityListItem.score` with the read-time ranking SQL,
    and adds the `services/rows.rs` decoder + `services/db.rs` upsert. Depends on this phase.
  - Phase 05 (`./05-cli-commands.md`) builds `PublishActivityRequest`, calls `validate()`,
    embeds title+body, and sets `embedding: Some(_)`. Depends on this phase.
  - Phase 06 (`./06-frontend-surfaces.md`) renders `ActivityItem`/`ActivityListItem` in WASM —
    the reason these types must stay WASM-safe. Depends (via Phase 03) on this phase.
- **In-repo patterns followed here (read these for the exact idioms):**
  - `crates/shared/src/portfolio_item.rs` — struct/serde/inline-test template that
    `activity_item.rs` mirrors. NOTE: portfolio uses a slug + `deserialize_flexible_id`
    `Option<String>` id; activity deliberately does NOT — it uses a plain `i64` numeric id.
  - `crates/shared/src/lib.rs` — gated `pub mod` + re-export convention.
  - `crates/shared/src/serde_helpers.rs` — `deserialize_flexible_id` definition (used by the
    slug bricks; activity does not import it).
  - `crates/server/migrations/0004_portfolio.sql` — table + index + `schema_migrations` insert
    style.
  - `crates/server/migrations/0003_blog.sql` — `embedding vector(384)` column and the
    `hnsw (embedding vector_cosine_ops)` index this migration copies.
  - `crates/server/migrations/0001_init.sql` — where `CREATE EXTENSION vector` and the
    `schema_migrations` table already live (do not duplicate).
