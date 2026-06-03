#![cfg(all(
    feature = "brick-blog",
    feature = "brick-portfolio",
    feature = "brick-todo",
    feature = "brick-activity"
))]

mod common;

use std::future::Future;
use std::sync::Arc;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
    middleware,
    routing::{delete, get, post, put},
};
use chrono::{TimeZone, Utc};
use futures_util::StreamExt;
use kameo::actor::Spawn;
use leptos::config::LeptosOptions;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list_with_exclusions_and_ssg_and_context};
use plinth_client::App;
use plinth_forge::{ActivityRef, ForgeClient, ForgeError};
use plinth_server::{
    AppState,
    actors::core_cache::CoreCache,
    api::{admin::auth_middleware, public},
    bricks::{
        activity::{
            admin::{delete_activity_handler, patch_activity_handler, publish_activity_item},
            api::{get_activity_item, list_activity_items},
            cache::ActivityCache,
        },
        blog::{
            admin::{add_tag_to_post, delete_article, publish_article, remove_tag_from_post},
            api::{
                get_blog_post, get_series_nav, list_blog_posts, list_blog_posts_by_tag,
                list_series, list_series_posts,
            },
            cache::BlogCache,
        },
        portfolio::{
            admin::publish_portfolio_item,
            api::{get_portfolio_item, list_portfolio_items},
            cache::PortfolioCache,
        },
        todo::{
            admin::{add_tag_to_todo, create_todo, delete_todo, remove_tag_from_todo, update_todo},
            api::{get_todo, list_todos, list_todos_by_tag},
            cache::TodoCache,
        },
    },
    config::PlinthConfig,
};
use plinth_shared::{
    CreateTodoRequest, FetchedActivity, RankingStrategy, UpdateSiteContentRequest,
    UpdateTodoRequest, toml_config::RankingConfig,
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;

const API_KEY: &str = "rendering_modes_secret";

async fn run_ssr<F, T>(future: F) -> T
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    std::thread::spawn(move || {
        tokio::runtime::LocalRuntime::new()
            .expect("create local runtime for Leptos SSR")
            .block_on(future)
    })
    .join()
    .expect("SSR local runtime thread")
}

async fn connect_like(pool: &PgPool) -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect_with((*pool.connect_options()).clone())
        .await
        .expect("connect SSR pool")
}

struct NoopForge;

#[async_trait::async_trait]
impl ForgeClient for NoopForge {
    async fn fetch(&self, _r: &ActivityRef) -> plinth_forge::ForgeResult<FetchedActivity> {
        Err(ForgeError::Network(
            "not used in rendering mode tests".to_string(),
        ))
    }
}

fn leptos_options(test_name: &str) -> LeptosOptions {
    let site_root = format!("target/test-site/rendering-modes/{test_name}");
    std::fs::remove_dir_all(&site_root).ok();
    std::fs::create_dir_all(format!("{site_root}/pkg")).expect("create test site pkg dir");

    LeptosOptions::builder()
        .output_name("plinth")
        .site_root(site_root)
        .site_pkg_dir("pkg")
        .build()
}

fn app_state(pool: PgPool, test_name: &str) -> AppState {
    let config = PlinthConfig::default();
    let site_config = config.to_site_config();
    let forge = config.forge.clone();
    let ranking = RankingConfig {
        strategy: RankingStrategy::Exponential,
        half_life_days: 365.0,
        window_days: 730.0,
    };

    AppState {
        leptos_options: leptos_options(test_name),
        core_cache: CoreCache::spawn(CoreCache::new(pool.clone())),
        db: pool.clone(),
        immich_config: None,
        http_client: reqwest::Client::builder()
            .build()
            .expect("build HTTP client"),
        config,
        site_config,
        blog_cache: BlogCache::spawn(BlogCache::new(pool.clone())),
        vector_search: None,
        portfolio_cache: PortfolioCache::spawn(PortfolioCache::new(pool.clone())),
        activity_cache: ActivityCache::spawn(ActivityCache::new(
            pool.clone(),
            ranking,
            forge,
            Arc::new(NoopForge),
        )),
        todo_cache: TodoCache::spawn(TodoCache::new(pool)),
    }
}

fn shell(options: LeptosOptions) -> impl IntoView {
    use leptos_meta::MetaTags;

    view! {
        <!DOCTYPE html>
        <html lang="en" class="dark">
            <head>
                <meta charset="utf-8"/>
                <MetaTags/>
                <HydrationScripts options islands=true/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

fn full_app(state: AppState) -> Router {
    let route_context = {
        let db = state.db.clone();
        let site_config = state.site_config.clone();
        move || {
            provide_context(db.clone());
            provide_context(site_config.clone());
        }
    };
    let ssr_shell = {
        let options = state.leptos_options.clone();
        move || shell(options.clone())
    };
    let (routes, _static_route_generator) = generate_route_list_with_exclusions_and_ssg_and_context(
        ssr_shell.clone(),
        None,
        route_context.clone(),
    );

    let admin_router = Router::new()
        .route(
            "/admin/content/{key}",
            put(plinth_server::api::admin::update_site_content)
                .get(plinth_server::api::admin::get_admin_site_content),
        )
        .route("/admin/articles", post(publish_article))
        .route("/admin/articles/{slug}", delete(delete_article))
        .route("/admin/posts/{post_slug}/tags", post(add_tag_to_post))
        .route(
            "/admin/posts/{post_slug}/tags/{tag_slug}",
            delete(remove_tag_from_post),
        )
        .route("/admin/portfolio", post(publish_portfolio_item))
        .route("/admin/todos", post(create_todo))
        .route("/admin/todos/{slug}", put(update_todo).delete(delete_todo))
        .route("/admin/todos/{todo_slug}/tags", post(add_tag_to_todo))
        .route(
            "/admin/todos/{todo_slug}/tags/{tag_slug}",
            delete(remove_tag_from_todo),
        )
        .route("/admin/activity", post(publish_activity_item))
        .route(
            "/admin/activity/{id}",
            delete(delete_activity_handler).patch(patch_activity_handler),
        )
        .layer(middleware::from_fn_with_state(
            Some(API_KEY.to_string()),
            auth_middleware,
        ));

    let public_api_router = Router::new()
        .route("/config", get(public::get_site_config))
        .route("/content/{key}", get(public::get_site_content))
        .route("/posts", get(list_blog_posts))
        .route("/posts/{slug}", get(get_blog_post))
        .route("/posts/tag/{tag}", get(list_blog_posts_by_tag))
        .route("/posts/{slug}/series-nav", get(get_series_nav))
        .route("/series", get(list_series))
        .route("/series/{slug}/posts", get(list_series_posts))
        .route("/portfolio", get(list_portfolio_items))
        .route("/portfolio/{slug}", get(get_portfolio_item))
        .route("/activity", get(list_activity_items))
        .route("/activity/{id}", get(get_activity_item))
        .route("/todos", get(list_todos))
        .route("/todos/{slug}", get(get_todo))
        .route("/todos/tag/{tag}", get(list_todos_by_tag));

    Router::new()
        .nest(
            "/api",
            admin_router
                .merge(public_api_router)
                .with_state(state.clone()),
        )
        .leptos_routes_with_context(&state, routes, route_context.clone(), ssr_shell.clone())
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>({
            let db = state.db.clone();
            let site_config = state.site_config.clone();
            move |options| {
                provide_context(db.clone());
                provide_context(site_config.clone());
                shell(options)
            }
        }))
        .with_state(state)
}

async fn get_html(app: Router, uri: &str) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .header(header::ACCEPT, "text/html")
                .header(header::ACCEPT_ENCODING, "identity")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK, "GET {uri}");
    let body = to_bytes(response.into_body(), 4 * 1024 * 1024)
        .await
        .unwrap();
    String::from_utf8(body.to_vec()).unwrap()
}

async fn admin_json<T: serde::Serialize>(app: Router, method: &str, uri: &str, payload: &T) {
    let response = app
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(header::AUTHORIZATION, format!("Bearer {API_KEY}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "{method} {uri} returned {}",
        response.status()
    );
}

async fn seed_site_content(pool: &PgPool, key: &str, title: &str, html: &str) {
    sqlx::query(
        r#"
        INSERT INTO site_content (key, title, content, html_content, updated_at)
        VALUES ($1, $2, $3, $4, now())
        ON CONFLICT (key) DO UPDATE SET
            title = EXCLUDED.title,
            content = EXCLUDED.content,
            html_content = EXCLUDED.html_content,
            updated_at = now()
        "#,
    )
    .bind(key)
    .bind(title)
    .bind(format!("{title} source"))
    .bind(html)
    .execute(pool)
    .await
    .expect("seed site content");
}

async fn seed_portfolio(pool: &PgPool, slug: &str, title: &str) {
    sqlx::query(
        r#"
        INSERT INTO portfolio_items (
            slug, title, description, content, html_content, tech_stack,
            link, demo, image_url, date, featured, "order"
        )
        VALUES ($1, $2, $3, $4, $5, ARRAY['Rust','Leptos']::text[],
                NULL, NULL, NULL, $6, false, 0)
        ON CONFLICT (slug) DO UPDATE SET
            title = EXCLUDED.title,
            description = EXCLUDED.description,
            content = EXCLUDED.content,
            html_content = EXCLUDED.html_content
        "#,
    )
    .bind(slug)
    .bind(title)
    .bind(format!("Description for {title}"))
    .bind(format!("# {title}"))
    .bind(format!("<h1>{title}</h1>"))
    .bind(Utc.with_ymd_and_hms(2026, 6, 1, 0, 0, 0).unwrap())
    .execute(pool)
    .await
    .expect("seed portfolio");
}

async fn seed_activity(pool: &PgPool, number: i32, title: &str) -> i64 {
    let id = common::insert_activity(
        pool,
        "github",
        "rendering",
        "modes",
        "pr",
        number,
        9,
        Utc.with_ymd_and_hms(2026, 6, 2, 0, 0, 0).unwrap(),
        Utc::now(),
        true,
        None,
    )
    .await
    .expect("seed activity");

    sqlx::query("UPDATE activity_items SET title = $1 WHERE id = $2")
        .bind(title)
        .bind(id)
        .execute(pool)
        .await
        .expect("set activity title");

    id
}

async fn seed_core_content(pool: &PgPool) {
    common::insert_blog_post(
        pool,
        "rendering-data-path-post",
        "Rendering Data Path Post",
        &["rendering"],
    )
    .await
    .expect("seed blog post");
    seed_portfolio(
        pool,
        "rendering-data-path-project",
        "Rendering Data Path Project",
    )
    .await;
    common::insert_todo(
        pool,
        "rendering-data-path-todo",
        "Rendering Data Path Todo",
        0,
        false,
        &["rendering"],
    )
    .await
    .expect("seed todo");
}

fn assert_html_contains(html: &str, needle: &str) {
    assert!(html.contains(needle), "expected HTML to contain {needle:?}");
    assert!(
        !html.contains("Could not load"),
        "route returned load failure fallback"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn rendering_data_path(pool: PgPool) {
    seed_core_content(&pool).await;
    run_ssr(async move {
        let ssr_pool = connect_like(&pool).await;
        let app = full_app(app_state(ssr_pool, "rendering_data_path"));

        let posts = get_html(app.clone(), "/posts").await;
        assert_html_contains(&posts, "Rendering Data Path Post");

        let projects = get_html(app.clone(), "/projects").await;
        assert_html_contains(&projects, "Rendering Data Path Project");

        let todos = get_html(app, "/todos").await;
        assert_html_contains(&todos, "Rendering Data Path Todo");
    })
    .await;
}

#[sqlx::test(migrations = "./migrations")]
async fn static_routes_no_per_request_sql(pool: PgPool) {
    seed_site_content(
        &pool,
        "about",
        "About",
        "<p>Static Cached About Version One</p>",
    )
    .await;
    let connect_options = (*pool.connect_options()).clone();

    run_ssr(async move {
        let ssr_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(connect_options.clone())
            .await
            .expect("connect static SSR pool");
        let app = full_app(app_state(
            ssr_pool.clone(),
            "static_routes_no_per_request_sql",
        ));

        let first = get_html(app.clone(), "/about").await;
        assert_html_contains(&first, "Static Cached About Version One");

        ssr_pool.close().await;
        let second = get_html(app, "/about").await;
        assert_html_contains(&second, "Static Cached About Version One");

        let reopened_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(connect_options)
            .await
            .expect("reopen sqlx test pool");
        let app = full_app(app_state(
            reopened_pool,
            "static_routes_no_per_request_sql_regenerated",
        ));
        let update = UpdateSiteContentRequest {
            title: Some("About".to_string()),
            content: "Static Cached About Version Two".to_string(),
            html_content: "<p>Static Cached About Version Two</p>".to_string(),
        };
        admin_json(app.clone(), "PUT", "/api/admin/content/about", &update).await;

        let regenerated =
            poll_html_contains(app, "/about", "Static Cached About Version Two").await;
        assert_html_contains(&regenerated, "Static Cached About Version Two");
    })
    .await;
}

#[sqlx::test(migrations = "./migrations")]
async fn home_streams_out_of_order(pool: PgPool) {
    seed_site_content(
        &pool,
        "home-intro",
        "Intro",
        "<p>Streaming Intro Marker</p>",
    )
    .await;
    common::insert_blog_post(
        &pool,
        "streaming-blog-marker",
        "Streaming Blog Marker",
        &["streaming"],
    )
    .await
    .expect("seed streaming blog");
    seed_portfolio(
        &pool,
        "streaming-project-marker",
        "Streaming Project Marker",
    )
    .await;
    seed_activity(&pool, 606, "Streaming Activity Marker").await;

    run_ssr(async move {
        let ssr_pool = connect_like(&pool).await;
        let _guard = EnvVarGuard::set("PLINTH_TEST_ACTIVITY_DELAY_MS", "900");
        let app = full_app(app_state(ssr_pool, "home_streams_out_of_order"));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/")
                    .header(header::ACCEPT, "text/html")
                    .header(header::ACCEPT_ENCODING, "identity")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let mut stream = response.into_body().into_data_stream();
        let mut prefix = String::new();
        let mut saw_activity = false;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.unwrap();
            prefix.push_str(&String::from_utf8_lossy(&chunk));
            if prefix.contains("Streaming Activity Marker") {
                saw_activity = true;
                break;
            }
            if prefix.contains("Streaming Intro Marker")
                && prefix.contains("Streaming Blog Marker")
                && prefix.contains("Streaming Project Marker")
            {
                break;
            }
        }

        assert!(
            !saw_activity,
            "delayed activity arrived before other sections"
        );
        assert_html_contains(&prefix, "Streaming Intro Marker");
        assert_html_contains(&prefix, "Streaming Blog Marker");
        assert_html_contains(&prefix, "Streaming Project Marker");

        let mut rest = prefix;
        while let Some(chunk) = stream.next().await {
            rest.push_str(&String::from_utf8_lossy(&chunk.unwrap()));
            if rest.contains("Streaming Activity Marker") {
                return;
            }
        }
        panic!("stream never yielded delayed activity marker");
    })
    .await;
}

#[sqlx::test(migrations = "./migrations")]
async fn islands_selective_hydration(pool: PgPool) {
    seed_site_content(
        &pool,
        "about",
        "About",
        "<p>Server Rendered About Island Boundary Check</p>",
    )
    .await;

    run_ssr(async move {
        let ssr_pool = connect_like(&pool).await;
        let app = full_app(app_state(ssr_pool, "islands_selective_hydration"));

        let html = get_html(app, "/about").await;
        assert_html_contains(&html, "Server Rendered About Island Boundary Check");
        assert!(
            html.contains("data-island") || html.contains("leptos-island"),
            "expected island boundary markers in served HTML"
        );
        assert!(
            html.contains("plinth") && html.contains(".wasm"),
            "expected island runtime assets in served HTML"
        );
        assert!(
            !html.contains("wasm-bindgen(start)"),
            "content page should not ship a full CSR start bundle marker"
        );
    })
    .await;
}

#[sqlx::test(migrations = "./migrations")]
async fn dynamic_routes_fresh(pool: PgPool) {
    run_ssr(async move {
        let ssr_pool = connect_like(&pool).await;
        let app = full_app(app_state(ssr_pool.clone(), "dynamic_routes_fresh"));

        seed_activity(&ssr_pool, 701, "Dynamic Activity One").await;
        let first_activity = get_html(app.clone(), "/activity").await;
        assert_html_contains(&first_activity, "Dynamic Activity One");

        seed_activity(&ssr_pool, 702, "Dynamic Activity Two").await;
        let second_activity = get_html(app.clone(), "/activity").await;
        assert_html_contains(&second_activity, "Dynamic Activity Two");

        let create = CreateTodoRequest {
            title: "Dynamic Todo One".to_string(),
            slug: Some("dynamic-todo".to_string()),
            description: "Dynamic todo description".to_string(),
            content: None,
            html_content: None,
            tags: vec!["dynamic".to_string()],
            completed: false,
            order: 0,
        };
        admin_json(app.clone(), "POST", "/api/admin/todos", &create).await;
        let first_todos = get_html(app.clone(), "/todos").await;
        assert_html_contains(&first_todos, "Dynamic Todo One");

        let update = UpdateTodoRequest {
            title: Some("Dynamic Todo Two".to_string()),
            description: None,
            content: None,
            html_content: None,
            tags: None,
            completed: None,
            order: None,
        };
        admin_json(app.clone(), "PUT", "/api/admin/todos/dynamic-todo", &update).await;
        let second_todos = get_html(app, "/todos").await;
        assert_html_contains(&second_todos, "Dynamic Todo Two");
    })
    .await;
}

async fn poll_html_contains(app: Router, uri: &str, needle: &str) -> String {
    for _ in 0..20 {
        let html = get_html(app.clone(), uri).await;
        if html.contains(needle) {
            return html;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    get_html(app, uri).await
}

struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var(key).ok();
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        unsafe {
            if let Some(previous) = &self.previous {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }
}
