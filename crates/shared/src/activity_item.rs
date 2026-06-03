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
/// flexible string-ID wrapper here.
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

/// Hook the server provides into the Leptos SSR context so the page read path can
/// drive freshness on a visit (decision: refresh lazily when someone views the
/// page if data is stale).
///
/// The SSR `#[server]` functions for the activity pages read the database
/// directly (the established plinth pattern), so they never touch the
/// `ActivityCache` actor that owns the stale-while-revalidate refresh. To still
/// let a page visit trigger that refresh without the WASM client depending on
/// `plinth-server` (which would create a dependency cycle), the server installs a
/// type-erased implementation of this trait into the request context. The client
/// retrieves it with `use_context::<std::sync::Arc<dyn ActivityRefreshHook>>()`
/// and calls [`poke`](ActivityRefreshHook::poke) after serving the page.
pub trait ActivityRefreshHook: Send + Sync {
    /// Ask the activity cache to consider a stale-while-revalidate refresh.
    ///
    /// Fire-and-forget: this never blocks the response and is a no-op when the
    /// cached data is still fresh, a refresh is already in flight, or the actor
    /// is in a backoff window.
    fn poke(&self);
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
        assert_eq!(
            r.validate(),
            Err(ActivityValidationError::ImpactOutOfRange(0))
        );
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
        for v in [
            ActivityState::Open,
            ActivityState::Closed,
            ActivityState::Merged,
        ] {
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
