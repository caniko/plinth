use super::*;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

fn target(url: String, kind: SiteCheckKind) -> SiteCheckTarget {
    SiteCheckTarget {
        id: "test".to_string(),
        title: "Test".to_string(),
        url,
        kind,
        routes: Vec::new(),
        markers: Vec::new(),
        expected_status: 200,
        follow_redirects: true,
    }
}

#[test]
fn parses_defaults() {
    let config: SiteCheckConfig = toml::from_str(
        r#"
        [[targets]]
        id = "site"
        title = "Site"
        url = "https://example.com"
        kind = "static"
        "#,
    )
    .unwrap();

    let target = &config.targets[0];
    assert_eq!(target.expected_status, 200);
    assert!(target.follow_redirects);
}

#[test]
fn expands_plinth_default_routes() {
    let routes = routes_for_target(&target(
        "https://example.com".to_string(),
        SiteCheckKind::Plinth,
    ));
    assert_eq!(
        routes,
        vec![
            "/",
            "/about",
            "/posts",
            "/projects",
            "/feeds/blog.xml",
            "/feeds/projects.xml"
        ]
    );
}

#[test]
fn expands_static_root_once() {
    let mut target = target("https://example.com".to_string(), SiteCheckKind::Static);
    target.routes = vec!["/".to_string(), "/docs".to_string()];
    assert_eq!(routes_for_target(&target), vec!["/", "/docs"]);
}

#[tokio::test]
async fn static_target_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string("hello marker"))
        .mount(&server)
        .await;

    let mut target = target(server.uri(), SiteCheckKind::Static);
    target.markers = vec!["marker".to_string()];

    let report = check_target(target).await;
    assert!(report.ok);
}

#[tokio::test]
async fn static_target_missing_marker_fails() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_string("hello"))
        .mount(&server)
        .await;

    let mut target = target(server.uri(), SiteCheckKind::Static);
    target.markers = vec!["marker".to_string()];

    let report = check_target(target).await;
    assert!(!report.ok);
    assert!(report.probes[0].message.contains("missing marker"));
}

#[tokio::test]
async fn plinth_bad_health_json_fails() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/health"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&server)
        .await;
    for route in routes_for_target(&target(server.uri(), SiteCheckKind::Plinth)) {
        Mock::given(method("GET"))
            .and(path(route))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&server)
            .await;
    }

    let report = check_target(target(server.uri(), SiteCheckKind::Plinth)).await;
    assert!(!report.ok);
    assert!(
        report.probes[0]
            .message
            .contains("failed to parse health JSON")
    );
}

#[tokio::test]
async fn redirect_handling_can_be_disabled() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(302).insert_header("location", "/final"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/final"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;

    let mut follows = target(server.uri(), SiteCheckKind::Static);
    assert!(check_target(follows.clone()).await.ok);

    follows.follow_redirects = false;
    let report = check_target(follows).await;
    assert!(!report.ok);
    assert_eq!(report.probes[0].status, Some(302));
}
