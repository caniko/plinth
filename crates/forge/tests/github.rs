use plinth_forge::{ActivityRef, ForgeClient, ForgeError, GitHubClient};
use plinth_shared::{ActivityKind, ActivityState, Forge};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn github_pr_merged() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/octocat/hello/pulls/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "title": "Fix the thing",
            "body": "details",
            "state": "closed",
            "merged": true,
            "merged_at": "2026-01-02T03:04:05Z",
            "created_at": "2026-01-01T00:00:00Z",
            "closed_at": "2026-01-02T03:04:05Z",
            "additions": 10,
            "deletions": 2,
            "comments": 3,
            "labels": [{"name": "bug"}],
            "html_url": "https://github.com/octocat/hello/pull/1"
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/repos/octocat/hello"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "stargazers_count": 42
        })))
        .mount(&server)
        .await;

    let client = GitHubClient::with_base_url(server.uri(), None).unwrap();
    let got = client
        .fetch(&activity_ref(ActivityKind::PullRequest, 1))
        .await;
    let got = got.expect("fetch ok");

    assert_eq!(got.forge, Forge::GitHub);
    assert_eq!(got.kind, ActivityKind::PullRequest);
    assert_eq!(got.state, ActivityState::Merged);
    assert!(got.merged_at.is_some());
    assert_eq!(got.additions, Some(10));
    assert_eq!(got.deletions, Some(2));
    assert_eq!(got.comments_count, Some(3));
    assert_eq!(got.repo_stars, Some(42));
    assert_eq!(got.labels, vec!["bug"]);
}

#[tokio::test]
async fn github_issue() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/octocat/hello/issues/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "title": "Track the thing",
            "body": null,
            "state": "open",
            "created_at": "2026-01-01T00:00:00Z",
            "closed_at": null,
            "comments": 5,
            "labels": [{"name": "enhancement"}],
            "html_url": "https://github.com/octocat/hello/issues/7"
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/repos/octocat/hello"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "stargazers_count": 42
        })))
        .mount(&server)
        .await;

    let client = GitHubClient::with_base_url(server.uri(), None).unwrap();
    let got = client.fetch(&activity_ref(ActivityKind::Issue, 7)).await;
    let got = got.expect("fetch ok");

    assert_eq!(got.forge, Forge::GitHub);
    assert_eq!(got.kind, ActivityKind::Issue);
    assert_eq!(got.state, ActivityState::Open);
    assert!(got.merged_at.is_none());
    assert!(got.additions.is_none());
    assert!(got.deletions.is_none());
    assert_eq!(got.comments_count, Some(5));
    assert_eq!(got.repo_stars, Some(42));
}

#[tokio::test]
async fn github_404() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/octocat/hello/pulls/999"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = GitHubClient::with_base_url(server.uri(), None).unwrap();
    let err = client
        .fetch(&activity_ref(ActivityKind::PullRequest, 999))
        .await
        .expect_err("expected not found");

    assert!(matches!(err, ForgeError::NotFound { .. }));
}

#[tokio::test]
async fn github_rate_limited() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/octocat/hello/pulls/1"))
        .respond_with(
            ResponseTemplate::new(403)
                .insert_header("x-ratelimit-remaining", "0")
                .insert_header("x-ratelimit-reset", "4102444800"),
        )
        .mount(&server)
        .await;

    let client = GitHubClient::with_base_url(server.uri(), None).unwrap();
    let err = client
        .fetch(&activity_ref(ActivityKind::PullRequest, 1))
        .await
        .expect_err("expected rate limit");

    assert!(matches!(
        err,
        ForgeError::RateLimited {
            retry_after: Some(_),
            ..
        }
    ));
}

fn activity_ref(kind: ActivityKind, number: i32) -> ActivityRef {
    ActivityRef {
        forge: Forge::GitHub,
        owner: "octocat".into(),
        repo: "hello".into(),
        kind,
        number,
    }
}
