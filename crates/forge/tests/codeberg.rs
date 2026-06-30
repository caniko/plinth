use plinth_forge::{ActivityRef, CodebergClient, ForgeClient, ForgeError};
use plinth_shared::{ActivityKind, ActivityState, Forge};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn codeberg_pr() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/repos/forgejo/forgejo/pulls/8326"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "title": "Improve federation",
            "body": "details",
            "state": "closed",
            "merged": true,
            "merged_at": "2026-01-02T03:04:05+02:00",
            "created_at": "2026-01-01T00:00:00+02:00",
            "closed_at": "2026-01-02T03:04:05+02:00",
            "additions": 12,
            "deletions": 4,
            "comments": 9,
            "labels": [{"name": "feature"}],
            "html_url": "https://codeberg.org/forgejo/forgejo/pulls/8326"
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/v1/repos/forgejo/forgejo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "stars_count": 100
        })))
        .mount(&server)
        .await;

    let client = CodebergClient::with_base_url(server.uri(), None).unwrap();
    let got = client
        .fetch(&activity_ref(ActivityKind::PullRequest, 8326))
        .await;
    let got = got.expect("fetch ok");

    assert_eq!(got.forge, Forge::Codeberg);
    assert_eq!(got.kind, ActivityKind::PullRequest);
    assert_eq!(got.state, ActivityState::Merged);
    assert!(got.merged_at.is_some());
    assert_eq!(got.additions, Some(12));
    assert_eq!(got.deletions, Some(4));
    assert_eq!(got.comments_count, Some(9));
    assert_eq!(got.repo_stars, Some(100));
    assert_eq!(got.labels, vec!["feature"]);
}

#[tokio::test]
async fn codeberg_404() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/repos/forgejo/forgejo/pulls/999"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = CodebergClient::with_base_url(server.uri(), None).unwrap();
    let err = client
        .fetch(&activity_ref(ActivityKind::PullRequest, 999))
        .await
        .expect_err("expected not found");

    assert!(matches!(err, ForgeError::NotFound { .. }));
}

#[tokio::test]
async fn codeberg_rate_limited() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/repos/forgejo/forgejo/pulls/8326"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "15"))
        .mount(&server)
        .await;

    let client = CodebergClient::with_base_url(server.uri(), None).unwrap();
    let err = client
        .fetch(&activity_ref(ActivityKind::PullRequest, 8326))
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
        forge: Forge::Codeberg,
        owner: "forgejo".into(),
        repo: "forgejo".into(),
        kind,
        number,
    }
}
