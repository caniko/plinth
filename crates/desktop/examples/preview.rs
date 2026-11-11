//! Standalone preview of the Plinth workspace module.
//!
//! ```bash
//! cargo run --features desktop --example preview -p plinth-desktop
//! PREVIEW_SIZE=760x900 PREVIEW_FIXTURE=offline cargo run --features desktop --example preview
//! ```
//!
//! The harness starts an in-process mock of the personal admin API, writes a
//! throwaway `plinth-project` checkout with two posts, and renders the module
//! inside a minimal stand-in for the Fleetix Hub shell.
//!
//! Fixtures: default (API + project site), `PREVIEW_FIXTURE=empty` (no sites),
//! `PREVIEW_FIXTURE=offline` (personal site on a closed port),
//! `PREVIEW_FIXTURE=badkey` (personal site rejects the key),
//! `PREVIEW_FIXTURE=readonly` (project `posts/` is not writable).
//! `PREVIEW_FIXTURE=fresh` (project site without posts),
//! `PREVIEW_FIXTURE=dense` (seven sites, 43 posts on the personal one).
//! `PREVIEW_DELAY_MS=3000` makes the mock API hold each publish that long,
//! `PREVIEW_API_LATE_MS=8000` starts the mock API that long after launch, and
//! `PREVIEW_DRAFTS_DIR=path` keeps drafts across launches.

use std::{
    net::TcpListener,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
};
use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use plinth_desktop::{
    ApiKey, BlogWorkspace, STYLE, Site, Target, WORKSPACE_HINT, WORKSPACE_ICON_SVG, WORKSPACE_LABEL,
};
use serde_json::{Value, json};
use tempfile::TempDir;

// ── Fixtures ────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Fixture {
    sites: Vec<Site>,
    drafts_dir: Option<PathBuf>,
    // Keeps the fixture directories alive for the life of the window.
    _dirs: Arc<Vec<TempDir>>,
}

impl Fixture {
    fn default_fixture() -> Self {
        let api_url = spawn_mock_api("preview-key");
        let project = TempDir::new().expect("project fixture directory");
        let website = project.path().join("website");
        std::fs::create_dir_all(website.join("posts")).expect("posts directory");
        std::fs::write(
            website.join("plinth-project.toml"),
            "[site]\ntitle = \"Pink Raven\"\ndescription = \"Notes and releases\"\n\n[blog]\n",
        )
        .expect("project config");
        std::fs::write(
            website.join("posts/first-light.md"),
            "---\ntitle: \"First light\"\ndescription: \"One-line summary\"\ndate: \"2026-10-02\"\ntags: [\"release\"]\npublished: true\n---\nThe first post.\n",
        )
        .expect("published post");
        std::fs::write(
            website.join("posts/next-up.md"),
            "---\ntitle: \"Next up\"\ndescription: \"Still cooking\"\ndate: \"2026-11-01\"\ntags: []\npublished: false\n---\nNot out yet.\n",
        )
        .expect("draft post");
        let drafts = TempDir::new().expect("drafts directory");
        let drafts_dir = std::env::var_os("PREVIEW_DRAFTS_DIR")
            .map_or_else(|| drafts.path().to_path_buf(), PathBuf::from);

        Self {
            sites: vec![
                Site {
                    id: "personal".to_string(),
                    name: "tartanoglu.com".to_string(),
                    target: Target::Personal {
                        api_url,
                        api_key: ApiKey::new("preview-key"),
                        // Links show the public host; posting uses the mock API.
                        public_url: Some("https://tartanoglu.com".to_string()),
                    },
                },
                Site {
                    id: "pink-raven".to_string(),
                    name: "Pink Raven".to_string(),
                    target: Target::Project {
                        root: website,
                        base_url: Some("https://pinkraven.example".to_string()),
                    },
                },
            ],
            drafts_dir: Some(drafts_dir),
            _dirs: Arc::new(vec![project, drafts]),
        }
    }

    fn empty() -> Self {
        Self {
            sites: Vec::new(),
            drafts_dir: None,
            _dirs: Arc::new(Vec::new()),
        }
    }

    fn offline() -> Self {
        let mut fixture = Self::default_fixture();
        fixture.sites[0].target = Target::Personal {
            // Port 1 has no listener: publish and list both fail to connect.
            api_url: "http://127.0.0.1:1".to_string(),
            api_key: ApiKey::new("preview-key"),
            public_url: Some("https://tartanoglu.com".to_string()),
        };
        fixture
    }

    fn badkey() -> Self {
        let mut fixture = Self::default_fixture();
        if let Target::Personal { api_key, .. } = &mut fixture.sites[0].target {
            *api_key = ApiKey::new("wrong-key");
        }
        fixture
    }

    fn fresh() -> Self {
        let fixture = Self::default_fixture();
        if let Target::Project { root, .. } = &fixture.sites[1].target {
            for name in ["first-light.md", "next-up.md"] {
                std::fs::remove_file(root.join("posts").join(name)).expect("remove fixture post");
            }
        }
        fixture
    }

    fn dense() -> Self {
        // Read by the mock API when it starts, on this thread.
        unsafe { std::env::set_var("PREVIEW_POSTS", "40") };
        let mut fixture = Self::default_fixture();
        let project = fixture.sites[1].clone();
        for name in [
            "Fleetix documentation and release notes",
            "Canix",
            "Harbor",
            "Simit continuous integration",
            "Tartan UI",
        ] {
            let mut site = project.clone();
            site.id = slugify_id(name);
            site.name = name.to_string();
            fixture.sites.push(site);
        }
        fixture
    }

    fn readonly() -> Self {
        use std::os::unix::fs::PermissionsExt;
        let fixture = Self::default_fixture();
        if let Target::Project { root, .. } = &fixture.sites[1].target {
            std::fs::set_permissions(root.join("posts"), std::fs::Permissions::from_mode(0o555))
                .expect("read-only posts directory");
        }
        fixture
    }
}

fn slugify_id(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

// ── Mock personal API ───────────────────────────────────────────────────

#[derive(Clone)]
struct MockApi {
    key: String,
    posts: Arc<Mutex<Vec<Value>>>,
}

async fn list_posts(State(api): State<MockApi>) -> Json<Value> {
    // Like the real server, the public list carries published posts only.
    let posts = api.posts.lock().expect("mock post list");
    Json(
        posts
            .iter()
            .filter(|post| post["published"] != false)
            .cloned()
            .collect(),
    )
}

fn authorized(api: &MockApi, headers: &HeaderMap) -> bool {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        == Some(api.key.as_str())
}

async fn list_drafts(State(api): State<MockApi>, headers: HeaderMap) -> impl IntoResponse {
    if !authorized(&api, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "unauthorized" })),
        );
    }
    let posts = api.posts.lock().expect("mock post list");
    let drafts: Vec<Value> = posts
        .iter()
        .filter(|post| post["published"] == false)
        .cloned()
        .collect();
    (StatusCode::OK, Json(Value::Array(drafts)))
}

async fn publish_post(
    State(api): State<MockApi>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    store(api, headers, body, None).await
}

async fn replace_draft(
    State(api): State<MockApi>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    store(api, headers, body, Some(id)).await
}

/// Like the server: create refuses any taken slug; replace swaps out the
/// draft with id `replacing` (404 if gone). Checks run before any change, and
/// a replaced draft gets a new id.
async fn store(
    api: MockApi,
    headers: HeaderMap,
    body: Value,
    replacing: Option<i64>,
) -> (StatusCode, Json<Value>) {
    if let Some(ms) = std::env::var("PREVIEW_DELAY_MS")
        .ok()
        .and_then(|raw| raw.parse().ok())
    {
        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
    }
    if !authorized(&api, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "unauthorized" })),
        );
    }

    let slug = body["slug"].as_str().unwrap_or_default().to_string();
    let mut posts = api.posts.lock().expect("mock post list");
    let replaced = match replacing {
        Some(id) => match posts
            .iter()
            .position(|post| post["id"] == id && post["published"] == false)
        {
            Some(at) => Some(at),
            None => {
                return (
                    StatusCode::GONE,
                    Json(json!({ "error": format!(
                        "Gone: draft {id} is gone: it was published, replaced or deleted since it was opened"
                    ) })),
                );
            }
        },
        None => None,
    };
    if posts
        .iter()
        .enumerate()
        .any(|(at, post)| Some(at) != replaced && post["slug"] == slug.as_str())
    {
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
        "published_at": chrono::Utc::now().to_rfc3339(),
        "author": "preview",
        "published": body["published"].as_bool().unwrap_or(true),
    }));
    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "slug": slug,
            "id": null,
            "message": "Article published successfully",
        })),
    )
}

fn spawn_mock_api(key: &str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("mock API port");
    let address = listener.local_addr().expect("mock API address");
    listener
        .set_nonblocking(true)
        .expect("non-blocking mock listener");

    let api = MockApi {
        key: key.to_string(),
        posts: Arc::new(Mutex::new(vec![
            json!({"id": 1, "slug": "hello-world", "title": "Hello, world", "description": "Start here", "published_at": "2026-10-01T09:00:00Z", "author": "can"}),
            json!({"id": 2, "slug": "release-notes", "title": "Release notes", "description": "What changed", "published_at": "2026-10-20T09:00:00Z", "author": "can"}),
            json!({"id": 3, "slug": "on-writing", "title": "On writing", "description": "Notes on notes", "published_at": "2026-11-05T09:00:00Z", "author": "can"}),
            json!({"id": 4, "slug": "field-notes", "title": "Field notes", "description": "Unfinished", "tags": ["notes"], "content": "Started on the train.\n\n## To add\n\n- the photos\n", "content_format": "markdown", "published_at": "2026-11-09T18:00:00Z", "author": "can", "published": false}),
        ])),
    };

    if let Some(extra) = std::env::var("PREVIEW_POSTS")
        .ok()
        .and_then(|raw| raw.parse::<u32>().ok())
    {
        let mut posts = api.posts.lock().expect("mock post list");
        for number in 1..=extra {
            posts.push(json!({"id": 100 + number, "slug": format!("weekly-notes-{number}"), "title": format!("Weekly notes {number}"), "description": "", "published_at": "2026-09-01T09:00:00Z", "author": "can"}));
        }
    }

    let router = Router::new()
        .route("/api/posts", get(list_posts))
        .route("/api/admin/articles", post(publish_post))
        .route("/api/admin/drafts", get(list_drafts))
        .route("/api/admin/drafts/{id}", put(replace_draft))
        .with_state(api);

    let late = std::env::var("PREVIEW_API_LATE_MS")
        .ok()
        .and_then(|raw| raw.parse().ok());
    // A late API refuses connections first: drop the port and take it back.
    let listener = if late.is_some() { None } else { Some(listener) };
    std::thread::spawn(move || {
        let listener = listener.unwrap_or_else(|| {
            std::thread::sleep(std::time::Duration::from_millis(late.unwrap_or(0)));
            let listener = TcpListener::bind(address).expect("late mock API port");
            listener
                .set_nonblocking(true)
                .expect("non-blocking mock listener");
            listener
        });
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("mock API runtime");
        runtime.block_on(async move {
            let listener = tokio::net::TcpListener::from_std(listener).expect("tokio listener");
            axum::serve(listener, router)
                .await
                .expect("mock API server");
        });
    });

    format!("http://{address}")
}

// ── Stand-in hub chrome ─────────────────────────────────────────────────

const HUB_STYLE: &str = r#"
*, *::before, *::after { box-sizing: border-box; }
html, body, #main { height: 100%; margin: 0; }
body {
  background: #000; color: #ebdbb2;
  font-family: var(--font-body, 'Inter', system-ui, sans-serif);
  font-size: 14px; line-height: 1.5;
  -webkit-font-smoothing: antialiased;
}
.hub { display: flex; height: 100%; min-height: 0; }
.hub-sidebar {
  width: 196px; flex: 0 0 196px; display: flex; flex-direction: column;
  gap: 4px; padding: 14px 10px;
  background: var(--panel, #181714); border-right: 1px solid var(--border, #353128);
}
.hub-brand { display: flex; align-items: center; gap: 10px; padding: 4px 8px 16px; }
.hub-mark { width: 18px; height: 18px; background: var(--honey, #e5a72b); transform: rotate(45deg); border-radius: 3px; }
.hub-brand span:last-child { font-family: var(--font-display, 'Space Grotesk', system-ui, sans-serif); font-weight: 700; font-size: 15px; }
.hub-nav {
  display: flex; align-items: center; gap: 10px; width: 100%;
  padding: 8px 10px; border: 0; border-radius: 7px;
  background: var(--card, #24221d); color: var(--honey, #e5a72b);
  font: inherit; font-size: 13.5px; text-align: left; cursor: pointer;
  box-shadow: inset 3px 0 0 var(--honey, #e5a72b);
}
.hub-nav .ico { display: inline-flex; width: 17px; height: 17px; color: currentColor; }
.hub-nav svg { width: 100%; height: 100%; }
.hub-main { display: flex; flex-direction: column; flex: 1; min-width: 0; min-height: 0; }
.hub-topbar {
  display: flex; align-items: center; gap: 12px; height: 54px; flex: 0 0 54px;
  padding: 0 18px; background: var(--panel, #181714);
  border-bottom: 1px solid var(--border, #353128);
}
.hub-topbar h1 { margin: 0; font-family: var(--font-display, 'Space Grotesk', system-ui, sans-serif); font-size: 17px; }
.hub-topbar .hint { color: var(--text-2, #bdae93); font-size: 12px; }
.hub-stage { flex: 1; min-height: 0; display: flex; }
.hub-stage > .plinth-ws { flex: 1; }
"#;

fn app() -> Element {
    let fixture = use_context::<Fixture>();
    rsx! {
        style { dangerous_inner_html: STYLE }
        style { dangerous_inner_html: HUB_STYLE }
        div { class: "hub",
            aside { class: "hub-sidebar",
                div { class: "hub-brand",
                    span { class: "hub-mark", aria_hidden: "true" }
                    span { "fleetix" }
                }
                nav { aria_label: "Workspaces",
                    button { class: "hub-nav", r#type: "button", aria_current: "page",
                        span { class: "ico", aria_hidden: "true", dangerous_inner_html: WORKSPACE_ICON_SVG }
                        span { "{WORKSPACE_LABEL}" }
                    }
                }
            }
            main { class: "hub-main",
                header { class: "hub-topbar",
                    h1 { "{WORKSPACE_LABEL}" }
                    span { class: "hint", "{WORKSPACE_HINT}" }
                }
                div { class: "hub-stage",
                    BlogWorkspace { sites: fixture.sites.clone(), drafts_dir: fixture.drafts_dir.clone() }
                }
            }
        }
    }
}

fn preview_size() -> (f64, f64) {
    let raw = std::env::var("PREVIEW_SIZE").unwrap_or_else(|_| "1440x900".to_string());
    let mut parts = raw.split('x');
    let width = parts
        .next()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(1440.0);
    let height = parts
        .next()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(900.0);
    (width, height)
}

fn main() {
    let fixture = match std::env::var("PREVIEW_FIXTURE").as_deref() {
        Ok("empty") => Fixture::empty(),
        Ok("offline") => Fixture::offline(),
        Ok("badkey") => Fixture::badkey(),
        Ok("readonly") => Fixture::readonly(),
        Ok("fresh") => Fixture::fresh(),
        Ok("dense") => Fixture::dense(),
        _ => Fixture::default_fixture(),
    };

    let (width, height) = preview_size();
    let window = WindowBuilder::new()
        .with_title("plinth-desktop preview")
        .with_inner_size(LogicalSize::new(width, height));

    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(window).with_menu(None))
        .with_context(fixture)
        .launch(app);
}
