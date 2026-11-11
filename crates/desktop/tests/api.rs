use std::{
    fs,
    net::TcpListener,
    sync::{Arc, Mutex},
};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
};
use plinth_desktop::{
    ApiKey, Draft, PublishError, Receipt, Replaces, Site, Target, list_posts, publish,
};
use serde_json::{Value, json};
use tempfile::TempDir;

// ── Mock personal API (mirrors the real admin/public surface) ────────────

#[derive(Clone)]
struct MockApi {
    key: String,
    posts: Arc<Mutex<Vec<Value>>>,
}

fn with_published(api: &MockApi, published: bool) -> Value {
    let posts = api.posts.lock().unwrap();
    let wanted = |post: &&Value| post["published"].as_bool().unwrap_or(true) == published;
    Value::Array(posts.iter().filter(wanted).cloned().collect())
}

fn authorized(api: &MockApi, headers: &HeaderMap) -> bool {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        == Some(api.key.as_str())
}

async fn mock_list(State(api): State<MockApi>) -> Json<Value> {
    Json(with_published(&api, true))
}

async fn mock_drafts(State(api): State<MockApi>, headers: HeaderMap) -> impl IntoResponse {
    if !authorized(&api, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "unauthorized" })),
        );
    }
    (StatusCode::OK, Json(with_published(&api, false)))
}

async fn mock_replace(
    State(api): State<MockApi>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    store(&api, &headers, &body, Some(id))
}

async fn mock_publish(
    State(api): State<MockApi>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    store(&api, &headers, &body, None)
}

/// Mirrors the server's transaction: every check happens before anything
/// changes, under one lock; a replaced draft gets a new id.
fn store(
    api: &MockApi,
    headers: &HeaderMap,
    body: &Value,
    replacing: Option<i64>,
) -> (StatusCode, Json<Value>) {
    if !authorized(api, headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "unauthorized" })),
        );
    }
    let slug = body["slug"].as_str().unwrap_or_default().to_string();
    let mut posts = api.posts.lock().unwrap();
    let replaced = match replacing {
        Some(id) => match posts
            .iter()
            .position(|post| post["id"] == id && post["published"] == false)
        {
            Some(at) => Some(at),
            None => {
                return (
                    StatusCode::GONE,
                    Json(json!({ "error": "Gone: no draft to replace" })),
                );
            }
        },
        None => None,
    };
    let taken = posts
        .iter()
        .enumerate()
        .any(|(at, post)| Some(at) != replaced && post["slug"] == slug.as_str());
    if taken {
        return (
            StatusCode::CONFLICT,
            Json(
                json!({ "error": "Conflict: a post with this slug already exists", "details": null }),
            ),
        );
    }
    // Taken before the removal, so a replaced draft never keeps its id.
    let id = posts
        .iter()
        .filter_map(|post| post["id"].as_i64())
        .max()
        .unwrap_or(0)
        + 1;
    if let Some(at) = replaced {
        posts.remove(at);
    }
    posts.push(json!({
        "id": id,
        "slug": slug,
        "title": body["title"],
        "description": body["description"].as_str().unwrap_or_default(),
        "tags": body["tags"].clone(),
        "content": body["content"].clone(),
        "content_format": "markdown",
        "published": body["published"].clone(),
        "published_at": chrono::Utc::now().to_rfc3339(),
        "author": "test",
    }));
    (
        StatusCode::OK,
        Json(
            json!({ "success": true, "slug": slug, "id": format!("blog_posts:{id}"), "message": "ok" }),
        ),
    )
}

fn spawn_mock(key: &str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let router = Router::new()
        .route("/api/posts", get(mock_list))
        .route("/api/admin/articles", post(mock_publish))
        .route("/api/admin/drafts", get(mock_drafts))
        .route("/api/admin/drafts/{id}", put(mock_replace))
        .with_state(MockApi {
            key: key.to_string(),
            posts: Arc::new(Mutex::new(vec![
                json!({"id": 1, "slug": "hello-world", "title": "Hello, world", "published_at": "2026-11-08T09:00:00Z"}),
                json!({"id": 2, "slug": "release-notes", "title": "Release notes", "published_at": "2026-09-01T09:00:00Z"}),
                json!({"id": 4, "slug": "idea", "title": "Idea", "description": "", "tags": ["rust"], "content": "First pass.", "content_format": "markdown", "published": false, "published_at": "2026-10-01T09:00:00Z"}),
                json!({"id": 3, "slug": "typeset", "title": "Typeset", "description": "", "tags": [], "content": "= Hi", "content_format": "typst", "published": false, "published_at": "2026-08-01T09:00:00Z"}),
            ])),
        });
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async move {
            let listener = tokio::net::TcpListener::from_std(listener).unwrap();
            axum::serve(listener, router).await.unwrap();
        });
    });
    format!("http://{address}")
}

fn personal_site(api_url: &str, key: &str) -> Site {
    Site {
        id: "personal".to_string(),
        name: "personal.test".to_string(),
        target: Target::Personal {
            api_url: api_url.to_string(),
            api_key: ApiKey::new(key),
            public_url: None,
        },
    }
}

fn draft(title: &str, slug: &str, published: bool) -> Draft {
    Draft {
        title: title.to_string(),
        slug: slug.to_string(),
        description: "One-line summary".to_string(),
        tags: vec![
            "Rust".to_string(),
            "release".to_string(),
            "rust".to_string(),
        ],
        body: "Hello *world*.\n".to_string(),
        published,
        replaces: None,
    }
}

// ── Personal site ───────────────────────────────────────────────────────

#[tokio::test]
async fn personal_publish_succeeds_and_lists_the_new_post() {
    let api_url = spawn_mock("test-key");
    let site = personal_site(&api_url, "test-key");

    let before = list_posts(&site).await.unwrap();
    assert_eq!(before.len(), 4);
    assert_eq!(before[0].slug, "hello-world");
    assert!(before[0].published);
    assert!(before[0].draft.is_none());
    assert_eq!(before[0].date.as_deref(), Some("2026-11-08"));

    let receipt = publish(&site, &draft("New post", "new-post", true))
        .await
        .unwrap();
    assert_eq!(
        receipt,
        Receipt::Personal {
            slug: "new-post".to_string(),
            url: format!("{api_url}/posts/new-post"),
        }
    );

    let after = list_posts(&site).await.unwrap();
    assert_eq!(after.len(), 5);
    assert!(after.iter().any(|post| post.slug == "new-post"));
}

#[tokio::test]
async fn personal_drafts_are_listed_and_an_opened_draft_is_promoted_in_place() {
    let api_url = spawn_mock("test-key");
    let site = personal_site(&api_url, "test-key");

    let listed = list_posts(&site).await.unwrap();
    let idea = listed.iter().find(|post| post.slug == "idea").unwrap();
    assert!(!idea.published);
    let opened = idea.draft.clone().expect("a markdown draft can be opened");
    assert_eq!(opened.body, "First pass.");
    assert_eq!(opened.tags, vec!["rust"]);
    let replaces = opened.replaces.clone().unwrap();
    assert_eq!(
        (replaces.slug.as_str(), replaces.version.as_str()),
        ("idea", "4")
    );
    let typeset = listed.iter().find(|post| post.slug == "typeset").unwrap();
    assert!(
        typeset.draft.is_none(),
        "typst drafts are listed, not opened"
    );

    // Without `replaces` the taken slug is refused, never overwritten.
    let fresh = Draft {
        replaces: None,
        published: true,
        ..opened.clone()
    };
    assert_eq!(
        publish(&site, &fresh).await.unwrap_err(),
        PublishError::SlugTaken
    );

    // A rename onto a taken slug is refused and keeps the draft.
    let onto = Draft {
        slug: "hello-world".to_string(),
        ..opened.clone()
    };
    assert_eq!(
        publish(&site, &onto).await.unwrap_err(),
        PublishError::SlugTaken
    );
    assert!(
        list_posts(&site)
            .await
            .unwrap()
            .iter()
            .any(|post| post.slug == "idea")
    );

    // Saving it unpublished gives it a new id: the copy opened before (the
    // highest id) is stale and cannot overwrite the newer save.
    let saved = Draft {
        body: "Second pass.".to_string(),
        ..opened.clone()
    };
    publish(&site, &saved).await.unwrap();
    match publish(&site, &opened).await.unwrap_err() {
        PublishError::DraftChanged => {}
        other => panic!("expected DraftChanged for a stale copy, got {other:?}"),
    }
    let reopened = list_posts(&site)
        .await
        .unwrap()
        .into_iter()
        .find(|post| post.slug == "idea")
        .and_then(|post| post.draft)
        .unwrap();
    assert_eq!(reopened.body, "Second pass.");

    let promoted = Draft {
        published: true,
        ..reopened
    };
    publish(&site, &promoted).await.unwrap();
    let after = list_posts(&site).await.unwrap();
    let idea = after.iter().find(|post| post.slug == "idea").unwrap();
    assert!(idea.published);
    assert_eq!(after.len(), listed.len());

    // The draft is gone now: a stale editor cannot replace the public post.
    match publish(&site, &promoted).await.unwrap_err() {
        PublishError::DraftChanged => {}
        other => panic!("expected DraftChanged, got {other:?}"),
    }
}

#[tokio::test]
async fn only_the_api_gone_answer_means_the_draft_changed() {
    let api_url = spawn_mock("test-key");
    let site = personal_site(&api_url, "test-key");
    let opened = list_posts(&site)
        .await
        .unwrap()
        .into_iter()
        .find_map(|post| post.draft)
        .unwrap();
    // A wrong base path answers 404 for the route: that is a rejection, not
    // a changed draft, so the user is not steered into creating a duplicate.
    let elsewhere = personal_site(&format!("{api_url}/old"), "test-key");
    match publish(&elsewhere, &opened).await.unwrap_err() {
        PublishError::Rejected { status: 404, .. } => {}
        other => panic!("expected a 404 rejection, got {other:?}"),
    }
}

#[tokio::test]
async fn personal_drafts_need_the_key() {
    let api_url = spawn_mock("test-key");
    let site = personal_site(&api_url, "wrong-key");
    assert_eq!(
        list_posts(&site).await.unwrap_err(),
        PublishError::Unauthorized
    );
}

#[tokio::test]
async fn personal_publish_maps_unauthorized() {
    let api_url = spawn_mock("test-key");
    let site = personal_site(&api_url, "wrong-key");

    let error = publish(&site, &draft("New post", "new-post", true))
        .await
        .unwrap_err();
    assert_eq!(error, PublishError::Unauthorized);
}

#[tokio::test]
async fn personal_publish_maps_duplicate_slug_to_slug_taken() {
    let api_url = spawn_mock("test-key");
    let site = personal_site(&api_url, "test-key");

    let error = publish(&site, &draft("Hello again", "hello-world", true))
        .await
        .unwrap_err();
    assert_eq!(error, PublishError::SlugTaken);
}

#[tokio::test]
async fn personal_publish_rejects_an_invalid_draft_before_any_request() {
    let api_url = spawn_mock("test-key");
    let site = personal_site(&api_url, "test-key");

    let error = publish(&site, &draft("  ", "Not A Slug", true))
        .await
        .unwrap_err();
    match error {
        PublishError::Invalid(errors) => assert_eq!(errors.len(), 2),
        other => panic!("expected Invalid, got {other:?}"),
    }
}

#[tokio::test]
async fn personal_publish_to_a_closed_port_is_unreachable() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let site = personal_site(&format!("http://{address}"), "test-key");

    let error = publish(&site, &draft("New post", "new-post", true))
        .await
        .unwrap_err();
    assert!(
        matches!(error, PublishError::Unreachable(_)),
        "expected Unreachable, got {error:?}"
    );
}

// ── Project site ────────────────────────────────────────────────────────

fn project_site() -> (TempDir, Site) {
    let directory = TempDir::new().unwrap();
    fs::write(
        directory.path().join("plinth-project.toml"),
        "[site]\ntitle = \"Test site\"\ndescription = \"fixture\"\n\n[blog]\n",
    )
    .unwrap();
    let site = Site {
        id: "project".to_string(),
        name: "Test project".to_string(),
        target: Target::Project {
            root: directory.path().to_path_buf(),
            base_url: None,
        },
    };
    (directory, site)
}

#[tokio::test]
async fn project_publish_writes_the_fixed_file_contract() {
    let (directory, site) = project_site();
    let mut draft = draft("My \"quoted\" post", "my-post", true);
    draft.body = "# Hello\n\nFirst post.\n\n".to_string();

    let receipt = publish(&site, &draft).await.unwrap();
    let path = directory.path().join("posts/my-post.md");
    assert_eq!(
        receipt,
        Receipt::Project {
            slug: "my-post".to_string(),
            path: path.clone(),
        }
    );

    let today = chrono::Local::now().date_naive().format("%Y-%m-%d");
    let expected = format!(
        concat!(
            "---\n",
            "title: \"My \\\"quoted\\\" post\"\n",
            "description: \"One-line summary\"\n",
            "date: \"{}\"\n",
            "tags: [\"rust\",\"release\"]\n",
            "published: true\n",
            "---\n",
            "# Hello\n\nFirst post.\n",
        ),
        today
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), expected);

    let posts = list_posts(&site).await.unwrap();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].slug, "my-post");
    assert_eq!(posts[0].title, "My \"quoted\" post");
    assert_eq!(posts[0].date, Some(today.to_string()));
    assert!(posts[0].published);
}

#[tokio::test]
async fn project_publish_refuses_to_overwrite() {
    let (directory, site) = project_site();
    publish(&site, &draft("First", "same-slug", true))
        .await
        .unwrap();
    let path = directory.path().join("posts/same-slug.md");
    let before = fs::read_to_string(&path).unwrap();

    let error = publish(&site, &draft("Second", "same-slug", false))
        .await
        .unwrap_err();
    assert_eq!(error, PublishError::SlugTaken);
    assert_eq!(fs::read_to_string(&path).unwrap(), before);
}

#[tokio::test]
async fn project_publish_requires_a_plinth_project_root() {
    let directory = TempDir::new().unwrap();
    let site = Site {
        id: "project".to_string(),
        name: "Not a project".to_string(),
        target: Target::Project {
            root: directory.path().to_path_buf(),
            base_url: None,
        },
    };

    let error = publish(&site, &draft("Post", "post", true))
        .await
        .unwrap_err();
    match error {
        PublishError::Io(message) => assert!(message.contains("plinth-project.toml")),
        other => panic!("expected Io, got {other:?}"),
    }
    assert!(!directory.path().join("posts").exists());
}

#[tokio::test]
async fn project_list_tolerates_unreadable_posts_and_reports_drafts() {
    let (directory, site) = project_site();
    let posts = directory.path().join("posts");
    fs::create_dir_all(&posts).unwrap();
    fs::write(
        posts.join("draft.md"),
        "---\ntitle: \"A draft\"\ndate: \"2026-11-02\"\npublished: false\n---\nbody\n",
    )
    .unwrap();
    fs::write(posts.join("broken.md"), "no frontmatter here").unwrap();
    fs::write(posts.join("notes.txt"), "not a post").unwrap();

    let listed = list_posts(&site).await.unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].slug, "draft");
    assert_eq!(listed[0].title, "A draft");
    assert!(!listed[0].published);
    let opened = listed[0].draft.clone().expect("drafts can be opened");
    assert_eq!(opened.body, "body");
    assert_eq!(opened.replaces.unwrap().slug, "draft");
}

#[tokio::test]
async fn project_opened_drafts_are_replaced_renamed_and_published_but_posts_are_not() {
    let (directory, site) = project_site();
    let posts = directory.path().join("posts");
    publish(&site, &draft("Idea", "idea", false)).await.unwrap();
    publish(&site, &draft("Live", "live", true)).await.unwrap();

    let opened = list_posts(&site)
        .await
        .unwrap()
        .into_iter()
        .find_map(|post| post.draft)
        .unwrap();
    assert_eq!(opened.slug, "idea");
    assert_eq!(opened.tags, vec!["rust", "release"]);

    // Same slug: replaced in place.
    let edited = Draft {
        body: "Second pass.\n".to_string(),
        ..opened.clone()
    };
    publish(&site, &edited).await.unwrap();
    assert!(
        fs::read_to_string(posts.join("idea.md"))
            .unwrap()
            .ends_with("---\nSecond pass.\n")
    );

    // The first copy is stale now: sending it again changes nothing.
    let current = fs::read_to_string(posts.join("idea.md")).unwrap();
    assert!(matches!(
        publish(&site, &opened).await.unwrap_err(),
        PublishError::DraftChanged
    ));
    assert_eq!(fs::read_to_string(posts.join("idea.md")).unwrap(), current);

    // Reopened, renamed and published: written under the new name, old
    // draft removed, nothing left aside.
    let reopened = list_posts(&site)
        .await
        .unwrap()
        .into_iter()
        .find_map(|post| post.draft)
        .unwrap();
    let promoted = Draft {
        slug: "final".to_string(),
        published: true,
        ..reopened
    };
    publish(&site, &promoted).await.unwrap();
    assert!(!posts.join("idea.md").exists());
    assert!(
        fs::read_to_string(posts.join("final.md"))
            .unwrap()
            .contains("published: true")
    );
    let names: Vec<String> = fs::read_dir(&posts)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(names.len(), 2, "only final.md and live.md: {names:?}");

    // A published file is never replaced, even when the version matches.
    let before = fs::read_to_string(posts.join("live.md")).unwrap();
    let forged = Draft {
        slug: "live".to_string(),
        replaces: Some(Replaces {
            slug: "live".to_string(),
            version: before.clone(),
        }),
        ..promoted
    };
    assert!(matches!(
        publish(&site, &forged).await.unwrap_err(),
        PublishError::DraftChanged
    ));
    assert_eq!(fs::read_to_string(posts.join("live.md")).unwrap(), before);
}
