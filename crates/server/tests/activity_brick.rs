#[cfg(feature = "brick-activity")]
mod common;

#[cfg(feature = "brick-activity")]
mod enabled {
    use crate::common;

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
    use plinth_forge::{ActivityRef, ForgeClient, ForgeError, ForgeResult};
    use plinth_server::{
        AppState,
        actors::core_cache::CoreCache,
        api::admin::auth_middleware,
        bricks::activity::{
            admin::{delete_activity_handler, patch_activity_handler, publish_activity_item},
            api::{get_activity_item, list_activity_items},
            cache::ActivityCache,
        },
        config::PlinthConfig,
    };
    use plinth_shared::FetchedActivity;
    use plinth_shared::toml_config::RankingConfig;
    use plinth_shared::{
        ActivityKind, ActivityListItem, ActivityState, Forge, PublishActivityRequest,
        RankingStrategy,
    };
    use serde_json::json;
    use sqlx::PgPool;
    use std::sync::Arc;
    use tower::ServiceExt;

    #[cfg(feature = "brick-blog")]
    use plinth_server::bricks::blog::cache::BlogCache;
    #[cfg(feature = "brick-portfolio")]
    use plinth_server::bricks::portfolio::cache::PortfolioCache;
    #[cfg(feature = "brick-todo")]
    use plinth_server::bricks::todo::cache::TodoCache;

    struct NoopForge;

    #[async_trait::async_trait]
    impl ForgeClient for NoopForge {
        async fn fetch(&self, _r: &ActivityRef) -> ForgeResult<FetchedActivity> {
            Err(ForgeError::Network(
                "not used in phase 03 tests".to_string(),
            ))
        }
    }

    fn app_state_with(pool: PgPool, strategy: RankingStrategy) -> AppState {
        let config = PlinthConfig::default();
        let site_config = config.to_site_config();
        let forge = config.forge.clone();
        let ranking = RankingConfig {
            strategy,
            half_life_days: 365.0,
            window_days: 730.0,
        };

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
                Arc::new(NoopForge),
            )),
            #[cfg(feature = "brick-todo")]
            todo_cache: TodoCache::spawn(TodoCache::new(pool)),
        }
    }

    fn app_state(pool: PgPool) -> AppState {
        app_state_with(pool, RankingStrategy::Exponential)
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
            .merge(admin_router)
            .with_state(state)
    }

    fn activity_request(number: i32, title: &str, impact: i16) -> PublishActivityRequest {
        PublishActivityRequest {
            forge: Forge::GitHub,
            repo_owner: "openai".to_string(),
            repo_name: "plinth".to_string(),
            kind: ActivityKind::PullRequest,
            number,
            url: format!("https://github.com/openai/plinth/pull/{number}"),
            title: title.to_string(),
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
            embedding: None,
            featured: false,
            published: true,
            content_hash: Some(format!("hash-{number}-{impact}")),
        }
    }

    async fn post_activity(
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

    async fn response_json<T: serde::de::DeserializeOwned>(
        response: axum::response::Response,
    ) -> T {
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        serde_json::from_slice(&body).unwrap()
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

    async fn seed_ranked_pair(pool: &PgPool) {
        let now = Utc::now();
        let old = now - Duration::days(800);
        let recent = now - Duration::days(1);
        sqlx::query(
            r#"
            INSERT INTO activity_items (
                forge, repo_owner, repo_name, kind, number, url, title, body, state,
                created_at, closed_at, merged_at, impact, additions, deletions,
                comments_count, labels, repo_stars, embedding, fetched_at,
                featured, published, content_hash
            )
            VALUES
                ('github', 'old', 'repo', 'pr', 1, 'https://example.com/old', 'old-high', NULL, 'merged',
                 $1, NULL, $1, 10, NULL, NULL, NULL, '{}', NULL, NULL, now(), false, true, NULL),
                ('github', 'recent', 'repo', 'pr', 2, 'https://example.com/recent', 'recent-low', NULL, 'merged',
                 $2, NULL, $2, 3, NULL, NULL, NULL, '{}', NULL, NULL, now(), false, true, NULL)
            "#,
        )
        .bind(old)
        .bind(recent)
        .execute(pool)
        .await
        .expect("seed activity ranking rows");
    }

    async fn ranked_items(pool: PgPool, strategy: RankingStrategy) -> Vec<ActivityListItem> {
        seed_ranked_pair(&pool).await;
        let app = test_app(app_state_with(pool, strategy));
        let response = get_request(app, "/api/activity").await;
        assert_eq!(response.status(), StatusCode::OK);
        response_json(response).await
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn admin_upsert_then_public_get_returns_it(pool: PgPool) {
        let app = test_app(app_state(pool.clone()));
        let request = activity_request(101, "Initial activity", 5);

        let response = post_activity(app.clone(), request.clone(), Some("test_secret")).await;
        assert_eq!(response.status(), StatusCode::OK);

        let public_response = get_request(app, "/api/activity").await;
        assert_eq!(public_response.status(), StatusCode::OK);
        let items: Vec<ActivityListItem> = response_json(public_response).await;
        assert!(items.iter().any(|item| item.url == request.url));

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM activity_items WHERE url = $1")
            .bind(&request.url)
            .fetch_one(&pool)
            .await
            .expect("count activity rows");
        assert_eq!(count, 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn admin_upsert_is_idempotent_on_natural_key(pool: PgPool) {
        let app = test_app(app_state(pool.clone()));
        let first = activity_request(102, "First", 2);
        let mut second = activity_request(102, "Second", 9);
        second.url = first.url.clone();

        assert_eq!(
            post_activity(app.clone(), first, Some("test_secret"))
                .await
                .status(),
            StatusCode::OK
        );
        assert_eq!(
            post_activity(app, second, Some("test_secret"))
                .await
                .status(),
            StatusCode::OK
        );

        let row: (i64, i16) = sqlx::query_as(
            "SELECT COUNT(*)::bigint, max(impact)::smallint FROM activity_items WHERE forge = 'github' AND repo_owner = 'openai' AND repo_name = 'plinth' AND kind = 'pr' AND number = 102",
        )
        .fetch_one(&pool)
        .await
        .expect("read idempotent upsert row");
        assert_eq!(row.0, 1);
        assert_eq!(row.1, 9);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn public_get_by_id_returns_item_and_404_semantics(pool: PgPool) {
        let app = test_app(app_state(pool.clone()));
        let request = activity_request(103, "Fetch by id", 4);
        assert_eq!(
            post_activity(app.clone(), request, Some("test_secret"))
                .await
                .status(),
            StatusCode::OK
        );

        let id: i64 = sqlx::query_scalar("SELECT id FROM activity_items WHERE number = 103")
            .fetch_one(&pool)
            .await
            .expect("read activity id");

        let response = get_request(app.clone(), &format!("/api/activity/{id}")).await;
        assert_eq!(response.status(), StatusCode::OK);
        let item: serde_json::Value = response_json(response).await;
        assert_eq!(item["id"], id);

        let missing = get_request(app, "/api/activity/999999").await;
        assert_eq!(missing.status(), StatusCode::OK);
        let missing_body: serde_json::Value = response_json(missing).await;
        assert_eq!(missing_body, serde_json::Value::Null);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn ranking_orders_two_rows_exponential(pool: PgPool) {
        let items = ranked_items(pool, RankingStrategy::Exponential).await;
        assert_eq!(items[0].title, "recent-low");
        assert_eq!(items[1].title, "old-high");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn ranking_orders_two_rows_linear(pool: PgPool) {
        let items = ranked_items(pool, RankingStrategy::Linear).await;
        assert_eq!(items[0].title, "recent-low");
        assert_eq!(items[1].title, "old-high");
        assert_eq!(items[1].score, 0.0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn ranking_orders_two_rows_pure(pool: PgPool) {
        let items = ranked_items(pool, RankingStrategy::Pure).await;
        assert_eq!(items[0].title, "old-high");
        assert_eq!(items[1].title, "recent-low");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn admin_requires_bearer_token(pool: PgPool) {
        let app = test_app(app_state(pool));
        let response = post_activity(app, activity_request(104, "No auth", 1), None).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn patch_updates_impact_and_featured(pool: PgPool) {
        let app = test_app(app_state(pool.clone()));
        assert_eq!(
            post_activity(
                app.clone(),
                activity_request(105, "Patch me", 2),
                Some("test_secret")
            )
            .await
            .status(),
            StatusCode::OK
        );
        let id: i64 = sqlx::query_scalar("SELECT id FROM activity_items WHERE number = 105")
            .fetch_one(&pool)
            .await
            .expect("read id");

        let response = app
            .oneshot(
                Request::builder()
                    .method("PATCH")
                    .uri(format!("/api/admin/activity/{id}"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::AUTHORIZATION, "Bearer test_secret")
                    .body(Body::from(
                        json!({ "impact": 7, "featured": true }).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let row: (i16, bool) =
            sqlx::query_as("SELECT impact, featured FROM activity_items WHERE id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .expect("read patched row");
        assert_eq!(row, (7, true));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn delete_removes_row(pool: PgPool) {
        let app = test_app(app_state(pool.clone()));
        assert_eq!(
            post_activity(
                app.clone(),
                activity_request(106, "Delete me", 2),
                Some("test_secret")
            )
            .await
            .status(),
            StatusCode::OK
        );
        let id: i64 = sqlx::query_scalar("SELECT id FROM activity_items WHERE number = 106")
            .fetch_one(&pool)
            .await
            .expect("read id");

        let delete_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/admin/activity/{id}"))
                    .header(header::AUTHORIZATION, "Bearer test_secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(delete_response.status(), StatusCode::OK);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM activity_items WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("count deleted rows");
        assert_eq!(count, 0);

        let second_delete = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/admin/activity/{id}"))
                    .header(header::AUTHORIZATION, "Bearer test_secret")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second_delete.status(), StatusCode::NOT_FOUND);
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
