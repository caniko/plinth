#[cfg(all(feature = "brick-activity", feature = "legacy-leptos"))]
mod common;

#[cfg(all(feature = "brick-activity", feature = "legacy-leptos"))]
mod enabled {
    use crate::common;

    use std::{
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::Duration as StdDuration,
    };

    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
        middleware,
        routing::{delete, get, post},
    };
    use chrono::{Duration, TimeZone, Utc};
    use kameo::actor::Spawn;
    use leptos::config::LeptosOptions;
    use pgvector::Vector;
    use plinth_forge::{ActivityRef, ForgeClient, ForgeError, ForgeResult};
    use plinth_server::{
        AppState,
        actors::{core_cache::CoreCache, vector_search::search_activity_by_vector},
        api::{admin::auth_middleware, feeds::activity_feed},
        bricks::activity::{
            admin::{delete_activity_handler, patch_activity_handler, publish_activity_item},
            api::{get_activity_item, list_activity_items},
            cache::ActivityCache,
        },
        config::PlinthConfig,
    };
    use plinth_shared::{
        ActivityKind, ActivityListItem, ActivityState, FetchedActivity, Forge,
        PublishActivityRequest,
    };
    use sqlx::{PgPool, Row};
    use tokio::{task::JoinSet, time::sleep};
    use tower::ServiceExt;

    #[cfg(feature = "brick-blog")]
    use plinth_server::bricks::blog::cache::BlogCache;
    #[cfg(feature = "brick-portfolio")]
    use plinth_server::bricks::portfolio::cache::PortfolioCache;
    #[cfg(feature = "brick-todo")]
    use plinth_server::bricks::todo::cache::TodoCache;

    enum MockMode {
        Success { delay: StdDuration, additions: i32 },
        HttpFailure,
    }

    struct MockForge {
        calls: AtomicUsize,
        mode: MockMode,
    }

    impl MockForge {
        fn success(additions: i32) -> Arc<Self> {
            Arc::new(Self {
                calls: AtomicUsize::new(0),
                mode: MockMode::Success {
                    delay: StdDuration::ZERO,
                    additions,
                },
            })
        }

        fn delayed_success(additions: i32, delay: StdDuration) -> Arc<Self> {
            Arc::new(Self {
                calls: AtomicUsize::new(0),
                mode: MockMode::Success { delay, additions },
            })
        }

        fn http_failure() -> Arc<Self> {
            Arc::new(Self {
                calls: AtomicUsize::new(0),
                mode: MockMode::HttpFailure,
            })
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl ForgeClient for MockForge {
        async fn fetch(&self, r: &ActivityRef) -> ForgeResult<FetchedActivity> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            match self.mode {
                MockMode::Success { delay, additions } => {
                    if !delay.is_zero() {
                        sleep(delay).await;
                    }
                    Ok(FetchedActivity {
                        forge: r.forge,
                        repo_owner: r.owner.clone(),
                        repo_name: r.repo.clone(),
                        kind: r.kind,
                        number: r.number,
                        url: format!("https://example.com/{}/{}/{}", r.owner, r.repo, r.number),
                        title: format!("refreshed {}", r.number),
                        body: Some("fresh body".to_string()),
                        state: ActivityState::Closed,
                        created_at: Utc::now() - Duration::days(10),
                        closed_at: Some(Utc::now()),
                        merged_at: Some(Utc::now()),
                        additions: Some(additions),
                        deletions: Some(7),
                        comments_count: Some(8),
                        labels: vec!["refreshed".to_string()],
                        repo_stars: Some(123),
                    })
                }
                MockMode::HttpFailure => Err(ForgeError::Http {
                    forge: r.forge,
                    status: 500,
                    body: "upstream failed".to_string(),
                }),
            }
        }
    }

    fn app_state(pool: PgPool, forge_client: Arc<dyn ForgeClient + Send + Sync>) -> AppState {
        let config = PlinthConfig::default();
        let site_config = config.to_site_config();
        let forge = config.forge.clone();
        let ranking = config.ranking.clone();

        AppState {
            leptos_options: LeptosOptions::builder().output_name("test").build(),
            core_cache: CoreCache::spawn(CoreCache::new(pool.clone())),
            db: pool.clone(),
            immich_config: None,
            http_client: common::test_http_client(),
            config,
            site_config,
            #[cfg(feature = "brick-blog")]
            blog_cache: BlogCache::spawn(BlogCache::new(pool.clone())),
            #[cfg(feature = "brick-blog")]
            vector_search: None,
            #[cfg(feature = "brick-portfolio")]
            portfolio_cache: PortfolioCache::spawn(PortfolioCache::new(pool.clone())),
            activity_cache: ActivityCache::spawn(ActivityCache::new(
                pool.clone(),
                ranking,
                forge,
                forge_client,
            )),
            #[cfg(feature = "brick-todo")]
            todo_cache: TodoCache::spawn(TodoCache::new(pool)),
        }
    }

    fn test_app(state: AppState) -> Router {
        let admin_router = Router::new()
            .route("/api/admin/activity", post(publish_activity_item))
            .route(
                "/api/admin/activity/{id}",
                delete(delete_activity_handler).patch(patch_activity_handler),
            )
            .layer(middleware::from_fn_with_state(
                Some("test_secret".to_string()),
                auth_middleware,
            ));

        Router::new()
            .route("/api/activity", get(list_activity_items))
            .route("/api/activity/{id}", get(get_activity_item))
            .route("/feeds/activity.xml", get(activity_feed))
            .route(
                "/api/search",
                get(plinth_server::api::search::search_articles),
            )
            .merge(admin_router)
            .with_state(state)
    }

    fn noop_forge() -> Arc<dyn ForgeClient + Send + Sync> {
        MockForge::success(0)
    }

    fn activity_request(number: i32, impact: i16) -> PublishActivityRequest {
        PublishActivityRequest {
            forge: Forge::GitHub,
            repo_owner: "openai".to_string(),
            repo_name: "plinth".to_string(),
            kind: ActivityKind::PullRequest,
            number,
            url: format!("https://github.com/openai/plinth/pull/{number}"),
            title: format!("Activity {number}"),
            body: Some("body".to_string()),
            state: ActivityState::Merged,
            created_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
            closed_at: None,
            merged_at: Some(Utc.with_ymd_and_hms(2026, 1, 2, 0, 0, 0).unwrap()),
            impact,
            additions: Some(12),
            deletions: Some(3),
            comments_count: Some(4),
            labels: vec!["rust".to_string()],
            repo_stars: Some(42),
            embedding: Some(unit_vector()),
            featured: false,
            published: true,
            content_hash: Some(format!("hash-{number}-{impact}")),
        }
    }

    fn unit_vector() -> Vec<f32> {
        let mut v = vec![0.0_f32; 384];
        v[0] = 1.0;
        v
    }

    async fn post_json(
        app: Router,
        request: PublishActivityRequest,
        token: Option<&str>,
    ) -> axum::response::Response {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/api/admin/activity")
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(token) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }

        app.oneshot(
            builder
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap()
    }

    async fn get_request(app: Router, uri: &str) -> axum::response::Response {
        app.oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
    }

    async fn response_json<T: serde::de::DeserializeOwned>(
        response: axum::response::Response,
    ) -> T {
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    async fn wait_for_additions(pool: &PgPool, id: i64, expected: i32) {
        for _ in 0..100 {
            let additions: Option<i32> =
                sqlx::query_scalar("SELECT additions FROM activity_items WHERE id = $1")
                    .bind(id)
                    .fetch_one(pool)
                    .await
                    .expect("read additions");
            if additions == Some(expected) {
                return;
            }
            sleep(StdDuration::from_millis(50)).await;
        }
        panic!("activity row {id} did not refresh to additions={expected}");
    }

    async fn wait_for_calls(mock: &MockForge, expected: usize) {
        for _ in 0..100 {
            if mock.calls() >= expected {
                return;
            }
            sleep(StdDuration::from_millis(50)).await;
        }
        panic!("mock forge did not reach {expected} calls");
    }

    async fn embedding(pool: &PgPool, id: i64) -> Vec<f32> {
        let vector: Vector =
            sqlx::query_scalar("SELECT embedding FROM activity_items WHERE id = $1")
                .bind(id)
                .fetch_one(pool)
                .await
                .expect("read embedding");
        vector.to_vec()
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn admin_upsert_requires_bearer_token(pool: PgPool) {
        let app = test_app(app_state(pool, noop_forge()));

        let unauthorized = post_json(app.clone(), activity_request(1, 5), None).await;
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

        let authorized = post_json(app, activity_request(1, 5), Some("test_secret")).await;
        assert_eq!(authorized.status(), StatusCode::OK);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn admin_upsert_creates_and_upserts_by_natural_key(pool: PgPool) {
        let app = test_app(app_state(pool.clone(), noop_forge()));
        let first = activity_request(2, 5);
        let mut second = activity_request(2, 9);
        second.url = first.url.clone();

        assert_eq!(
            post_json(app.clone(), first.clone(), Some("test_secret"))
                .await
                .status(),
            StatusCode::OK
        );
        assert_eq!(
            post_json(app, second, Some("test_secret")).await.status(),
            StatusCode::OK
        );

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM activity_items WHERE url = $1")
            .bind(&first.url)
            .fetch_one(&pool)
            .await
            .expect("count upserted activity rows");
        assert_eq!(count, 1);

        let impact: i16 = sqlx::query_scalar("SELECT impact FROM activity_items WHERE url = $1")
            .bind(&first.url)
            .fetch_one(&pool)
            .await
            .expect("read updated impact");
        assert_eq!(impact, 9);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn public_list_returns_ranked_order(pool: PgPool) {
        let now = Utc::now();
        let high_recent = common::insert_activity(
            &pool, "github", "one", "repo", "pr", 1, 10, now, now, false, None,
        )
        .await
        .expect("insert recent high-impact activity");
        let high_old = common::insert_activity(
            &pool,
            "github",
            "two",
            "repo",
            "pr",
            2,
            10,
            now - Duration::days(365),
            now,
            true,
            None,
        )
        .await
        .expect("insert old high-impact activity");
        let low_recent = common::insert_activity(
            &pool, "github", "three", "repo", "pr", 3, 1, now, now, false, None,
        )
        .await
        .expect("insert recent low-impact activity");

        let app = test_app(app_state(pool, noop_forge()));
        let response = get_request(app.clone(), "/api/activity").await;
        assert_eq!(response.status(), StatusCode::OK);
        let items: Vec<ActivityListItem> = response_json(response).await;
        let ids = items.iter().map(|item| item.id).collect::<Vec<_>>();
        assert_eq!(ids, vec![high_recent, high_old, low_recent]);

        let featured_response = get_request(app, "/api/activity?featured=true").await;
        assert_eq!(featured_response.status(), StatusCode::OK);
        let featured: Vec<ActivityListItem> = response_json(featured_response).await;
        assert_eq!(
            featured.iter().map(|item| item.id).collect::<Vec<_>>(),
            vec![high_old]
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn refresh_on_stale_read_updates_db_from_mocked_forge(pool: PgPool) {
        let stale_fetched_at = Utc::now() - Duration::hours(2);
        let id = common::insert_activity(
            &pool,
            "github",
            "stale",
            "repo",
            "pr",
            10,
            5,
            Utc::now() - Duration::days(5),
            stale_fetched_at,
            false,
            None,
        )
        .await
        .expect("insert stale activity");
        let mock = MockForge::success(99);
        let app = test_app(app_state(pool.clone(), mock.clone()));

        let response = get_request(app, "/api/activity").await;
        assert_eq!(response.status(), StatusCode::OK);
        let stale: Vec<ActivityListItem> = response_json(response).await;
        assert_eq!(stale.len(), 1);

        wait_for_additions(&pool, id, 99).await;
        assert!(mock.calls() >= 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn refresh_failure_keeps_stale_data(pool: PgPool) {
        let id = common::insert_activity(
            &pool,
            "github",
            "failure",
            "repo",
            "pr",
            11,
            5,
            Utc::now() - Duration::days(5),
            Utc::now() - Duration::hours(2),
            false,
            None,
        )
        .await
        .expect("insert stale activity");
        let mock = MockForge::http_failure();
        let app = test_app(app_state(pool.clone(), mock.clone()));

        let response = get_request(app, "/api/activity").await;
        assert_eq!(response.status(), StatusCode::OK);
        wait_for_calls(&mock, 1).await;
        sleep(StdDuration::from_millis(100)).await;

        let row = sqlx::query("SELECT state, additions FROM activity_items WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("read stale row");
        assert_eq!(row.try_get::<String, _>("state").unwrap(), "merged");
        assert_eq!(row.try_get::<Option<i32>, _>("additions").unwrap(), Some(1));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn refresh_is_single_flighted(pool: PgPool) {
        let id = common::insert_activity(
            &pool,
            "github",
            "single",
            "repo",
            "pr",
            12,
            5,
            Utc::now() - Duration::days(5),
            Utc::now() - Duration::hours(2),
            false,
            None,
        )
        .await
        .expect("insert stale activity");
        let mock = MockForge::delayed_success(77, StdDuration::from_millis(300));
        let app = test_app(app_state(pool.clone(), mock.clone()));

        let mut tasks = JoinSet::new();
        for _ in 0..10 {
            let app = app.clone();
            tasks.spawn(async move {
                let response = get_request(app, "/api/activity").await;
                assert_eq!(response.status(), StatusCode::OK);
            });
        }
        while let Some(result) = tasks.join_next().await {
            result.expect("join activity read");
        }

        wait_for_calls(&mock, 1).await;
        wait_for_additions(&pool, id, 77).await;
        assert_eq!(mock.calls(), 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn activity_feed_returns_valid_rss(pool: PgPool) {
        common::insert_activity(
            &pool,
            "github",
            "feed",
            "repo",
            "pr",
            13,
            5,
            Utc::now(),
            Utc::now(),
            false,
            None,
        )
        .await
        .expect("insert feed activity");
        let app = test_app(app_state(pool, noop_forge()));

        let response = get_request(app, "/feeds/activity.xml").await;
        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .expect("content type")
            .to_str()
            .expect("content type text");
        assert!(content_type.contains("application/rss+xml"));

        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let xml = String::from_utf8(body.to_vec()).unwrap();
        assert!(xml.contains("<rss"));
        assert!(xml.contains("<item>"));
        assert!(xml.contains("pr #13 on feed/repo"));
        assert!(xml.contains("<link>https://github.example/feed/repo/pr/13</link>"));
        rss::Channel::read_from(xml.as_bytes()).expect("valid RSS");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn search_union_includes_activity_items(pool: PgPool) {
        let embedding = unit_vector();
        let _blog_id = common::insert_blog_post(&pool, "search-post", "Search Post", &["search"])
            .await
            .expect("insert blog post");
        let activity_id = common::insert_activity(
            &pool,
            "github",
            "search",
            "repo",
            "pr",
            14,
            5,
            Utc::now(),
            Utc::now(),
            false,
            Some(embedding.clone()),
        )
        .await
        .expect("insert searchable activity");

        let hits = search_activity_by_vector(&pool, embedding, 10, 0.0)
            .await
            .expect("activity search SQL");
        let (item, similarity) = hits
            .iter()
            .find(|(item, _)| item.id == activity_id)
            .expect("activity search includes seeded activity item");
        assert_eq!(item.title, "pr #14 on search/repo");
        assert_eq!(*similarity, 1.0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn refresh_does_not_reembed(pool: PgPool) {
        let original_embedding = unit_vector();
        let id = common::insert_activity(
            &pool,
            "github",
            "embed",
            "repo",
            "pr",
            15,
            5,
            Utc::now() - Duration::days(5),
            Utc::now() - Duration::hours(2),
            false,
            Some(original_embedding.clone()),
        )
        .await
        .expect("insert embedded stale activity");
        let mock = MockForge::success(88);
        let app = test_app(app_state(pool.clone(), mock));

        let response = get_request(app, "/api/activity").await;
        assert_eq!(response.status(), StatusCode::OK);
        wait_for_additions(&pool, id, 88).await;

        let after = embedding(&pool, id).await;
        assert_eq!(after, original_embedding);
    }
}

#[cfg(not(feature = "brick-activity"))]
mod disabled {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn activity_admin_route_absent_without_feature() {
        let app: Router = Router::new();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/admin/activity")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
