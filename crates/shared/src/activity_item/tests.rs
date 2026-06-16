use chrono::Utc;

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
