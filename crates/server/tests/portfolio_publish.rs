#[cfg(feature = "brick-portfolio")]
mod common;

#[cfg(feature = "brick-portfolio")]
mod enabled {
    use crate::common;

    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
        middleware,
        routing::{get, post},
    };
    use chrono::{TimeZone, Utc};
    use kameo::actor::Spawn;
    use leptos::config::LeptosOptions;
    #[cfg(feature = "brick-activity")]
    use plinth_forge::{ActivityRef, ForgeClient, ForgeError, ForgeResult};
    use plinth_server::{
        AppState,
        actors::core_cache::CoreCache,
        api::admin::auth_middleware,
        bricks::portfolio::{
            admin::publish_portfolio_item,
            api::list_portfolio_items,
            cache::{GetAllPortfolioItems, PortfolioCache},
        },
        config::PlinthConfig,
    };
    #[cfg(feature = "brick-activity")]
    use plinth_shared::FetchedActivity;
    use plinth_shared::{ContentFormat, PublishPortfolioRequest};
    use sqlx::PgPool;
    #[cfg(feature = "brick-activity")]
    use std::sync::Arc;
    use tower::ServiceExt;

    #[cfg(feature = "brick-activity")]
    use plinth_server::bricks::activity::cache::ActivityCache;
    #[cfg(feature = "brick-blog")]
    use plinth_server::bricks::blog::cache::BlogCache;
    #[cfg(feature = "brick-todo")]
    use plinth_server::bricks::todo::cache::TodoCache;

    #[cfg(feature = "brick-activity")]
    struct NoopForge;

    #[cfg(feature = "brick-activity")]
    #[async_trait::async_trait]
    impl ForgeClient for NoopForge {
        async fn fetch(&self, _r: &ActivityRef) -> ForgeResult<FetchedActivity> {
            Err(ForgeError::Network(
                "not used in portfolio tests".to_string(),
            ))
        }
    }

    fn app_state(pool: PgPool) -> AppState {
        let config = PlinthConfig::default();
        let site_config = config.to_site_config();
        #[cfg(feature = "brick-activity")]
        let forge = config.forge.clone();
        #[cfg(feature = "brick-activity")]
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
            portfolio_cache: PortfolioCache::spawn(PortfolioCache::new(pool.clone())),
            #[cfg(feature = "brick-activity")]
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

    fn test_app(state: AppState) -> Router {
        let admin_router = Router::new()
            .route("/api/admin/portfolio", post(publish_portfolio_item))
            .layer(middleware::from_fn_with_state(
                Some("test_secret".to_string()),
                auth_middleware,
            ));

        Router::new()
            .route("/api/portfolio", get(list_portfolio_items))
            .merge(admin_router)
            .with_state(state)
    }

    fn manifest(title: &str, description: &str) -> PublishPortfolioRequest {
        PublishPortfolioRequest {
            id: None,
            slug: Some("test-tool".to_string()),
            title: title.to_string(),
            description: description.to_string(),
            content: Some("# Test Tool\n\nPortfolio body.".to_string()),
            html_content: None,
            tech_stack: vec!["Rust".to_string(), "Leptos".to_string()],
            link: Some("https://example.com/repo".to_string()),
            demo: Some("https://example.com/demo".to_string()),
            image_url: Some("https://example.com/image.png".to_string()),
            date: Utc.with_ymd_and_hms(2026, 5, 28, 0, 0, 0).unwrap(),
            featured: true,
            order: 7,
            content_format: Some(ContentFormat::Markdown),
        }
    }

    async fn post_manifest(
        app: Router,
        request: PublishPortfolioRequest,
        token: Option<&str>,
    ) -> axum::response::Response {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/api/admin/portfolio")
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

    #[sqlx::test(migrations = "./migrations")]
    async fn posting_valid_manifest_creates_row_and_refreshes_cached_list(pool: PgPool) {
        let state = app_state(pool.clone());
        let app = test_app(state.clone());

        let before = state
            .portfolio_cache
            .ask(GetAllPortfolioItems)
            .await
            .expect("ask portfolio cache before publish");
        assert!(before.is_empty());

        let response = post_manifest(
            app.clone(),
            manifest("Test Tool", "Initial"),
            Some("test_secret"),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM portfolio_items WHERE slug = $1")
            .bind("test-tool")
            .fetch_one(&pool)
            .await
            .expect("count portfolio rows");
        assert_eq!(count, 1);

        let html: Option<String> =
            sqlx::query_scalar("SELECT html_content FROM portfolio_items WHERE slug = $1")
                .bind("test-tool")
                .fetch_one(&pool)
                .await
                .expect("read rendered html");
        assert!(html.unwrap().contains("<h1>Test Tool</h1>"));

        let cached_after = state
            .portfolio_cache
            .ask(GetAllPortfolioItems)
            .await
            .expect("ask portfolio cache after publish");
        assert_eq!(cached_after.len(), 1);
        assert_eq!(cached_after[0].slug, "test-tool");

        let public_response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/portfolio")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(public_response.status(), StatusCode::OK);
        let body = to_bytes(public_response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let items: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(items.as_array().unwrap().len(), 1);
        assert_eq!(items[0]["slug"], "test-tool");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn posting_same_slug_upserts_without_duplicate(pool: PgPool) {
        let app = test_app(app_state(pool.clone()));

        let first = post_manifest(
            app.clone(),
            manifest("Test Tool", "Initial"),
            Some("test_secret"),
        )
        .await;
        assert_eq!(first.status(), StatusCode::OK);

        let second = post_manifest(
            app,
            manifest("Test Tool Updated", "Updated"),
            Some("test_secret"),
        )
        .await;
        assert_eq!(second.status(), StatusCode::OK);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM portfolio_items WHERE slug = $1")
            .bind("test-tool")
            .fetch_one(&pool)
            .await
            .expect("count portfolio rows");
        assert_eq!(count, 1);

        let title: String = sqlx::query_scalar("SELECT title FROM portfolio_items WHERE slug = $1")
            .bind("test-tool")
            .fetch_one(&pool)
            .await
            .expect("read updated title");
        assert_eq!(title, "Test Tool Updated");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn posting_without_bearer_token_returns_401(pool: PgPool) {
        let app = test_app(app_state(pool));
        let response = post_manifest(app, manifest("Test Tool", "Initial"), None).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

#[cfg(not(feature = "brick-portfolio"))]
mod disabled {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn portfolio_admin_route_absent_without_feature() {
        let app: Router = Router::new();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/admin/portfolio")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
