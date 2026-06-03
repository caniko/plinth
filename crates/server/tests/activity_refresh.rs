#[cfg(feature = "brick-activity")]
mod common;

#[cfg(feature = "brick-activity")]
mod enabled {
    use crate::common;

    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration as StdDuration;

    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Request, StatusCode},
        routing::get,
    };
    use chrono::{Duration, Utc};
    use kameo::actor::Spawn;
    use leptos::config::LeptosOptions;
    use plinth_forge::{ActivityRef, ForgeClient, ForgeError, ForgeResult};
    use plinth_server::{
        AppState,
        actors::core_cache::CoreCache,
        bricks::activity::{
            api::list_activity_items,
            cache::{ActivityCache, GetRankedActivity},
        },
        config::PlinthConfig,
    };
    use plinth_shared::toml_config::{ForgeConfig, RankingConfig};
    use plinth_shared::{ActivityListItem, ActivityState, FetchedActivity, RankingStrategy};
    use sqlx::PgPool;
    use tokio::time::{sleep, timeout};
    use tower::ServiceExt;

    #[cfg(feature = "brick-blog")]
    use plinth_server::bricks::blog::cache::BlogCache;
    #[cfg(feature = "brick-portfolio")]
    use plinth_server::bricks::portfolio::cache::PortfolioCache;
    #[cfg(feature = "brick-todo")]
    use plinth_server::bricks::todo::cache::TodoCache;

    #[derive(Clone, Copy)]
    enum MockMode {
        Success,
        RateLimited,
    }

    struct MockForge {
        fetch_count: AtomicUsize,
        sweep_count: AtomicUsize,
        delay: StdDuration,
        mode: MockMode,
    }

    impl MockForge {
        fn success(delay: StdDuration) -> Arc<Self> {
            Arc::new(Self {
                fetch_count: AtomicUsize::new(0),
                sweep_count: AtomicUsize::new(0),
                delay,
                mode: MockMode::Success,
            })
        }

        fn rate_limited() -> Arc<Self> {
            Arc::new(Self {
                fetch_count: AtomicUsize::new(0),
                sweep_count: AtomicUsize::new(0),
                delay: StdDuration::ZERO,
                mode: MockMode::RateLimited,
            })
        }

        fn fetch_count(&self) -> usize {
            self.fetch_count.load(Ordering::SeqCst)
        }

        fn sweep_count(&self) -> usize {
            self.sweep_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl ForgeClient for MockForge {
        async fn fetch(&self, r: &ActivityRef) -> ForgeResult<FetchedActivity> {
            self.fetch_count.fetch_add(1, Ordering::SeqCst);
            if r.number == 1 {
                self.sweep_count.fetch_add(1, Ordering::SeqCst);
            }
            if !self.delay.is_zero() {
                sleep(self.delay).await;
            }
            match self.mode {
                MockMode::Success => Ok(FetchedActivity {
                    forge: r.forge,
                    repo_owner: r.owner.clone(),
                    repo_name: r.repo.clone(),
                    kind: r.kind,
                    number: r.number,
                    url: format!("https://example.com/{}/{}", r.repo, r.number),
                    title: format!("refreshed {}", r.number),
                    body: None,
                    state: ActivityState::Merged,
                    created_at: Utc::now() - Duration::days(10),
                    closed_at: Some(Utc::now()),
                    merged_at: Some(Utc::now()),
                    additions: Some(100 + r.number),
                    deletions: Some(10 + r.number),
                    comments_count: Some(5 + r.number),
                    labels: vec!["refreshed".to_string()],
                    repo_stars: Some(9000 + r.number),
                }),
                MockMode::RateLimited => Err(ForgeError::RateLimited {
                    forge: r.forge,
                    retry_after: None,
                }),
            }
        }
    }

    fn app_state_with(
        pool: PgPool,
        forge: ForgeConfig,
        forge_client: Arc<dyn ForgeClient + Send + Sync>,
    ) -> AppState {
        let config = PlinthConfig::default();
        let site_config = config.to_site_config();
        let ranking = RankingConfig {
            strategy: RankingStrategy::Exponential,
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
                forge_client,
            )),
            #[cfg(feature = "brick-todo")]
            todo_cache: TodoCache::spawn(TodoCache::new(pool)),
        }
    }

    fn forge_config(ttl_secs: u64, backoff_secs: u64) -> ForgeConfig {
        ForgeConfig {
            refresh_ttl_secs: ttl_secs,
            refresh_backoff_secs: backoff_secs,
            github_base_url: "http://github.invalid".to_string(),
            codeberg_base_url: "http://codeberg.invalid/api/v1".to_string(),
        }
    }

    fn test_app(state: AppState) -> Router {
        Router::new()
            .route("/api/activity", get(list_activity_items))
            .with_state(state)
    }

    async fn seed_rows(pool: &PgPool, fetched_at: chrono::DateTime<Utc>) {
        sqlx::query(
            r#"
            INSERT INTO activity_items (
                forge, repo_owner, repo_name, kind, number, url, title, body, state,
                created_at, closed_at, merged_at, impact, additions, deletions,
                comments_count, labels, repo_stars, embedding, fetched_at,
                featured, published, content_hash
            )
            VALUES
                ('github', 'owner', 'repo', 'pr', 1, 'https://example.com/one', 'one', NULL, 'open',
                 $1, NULL, NULL, 5, NULL, NULL, NULL, '{}', NULL, NULL, $2, false, true, NULL),
                ('github', 'owner', 'repo', 'pr', 2, 'https://example.com/two', 'two', NULL, 'open',
                 $1, NULL, NULL, 4, NULL, NULL, NULL, '{}', NULL, NULL, $2, false, true, NULL)
            "#,
        )
        .bind(Utc::now() - Duration::days(30))
        .bind(fetched_at)
        .execute(pool)
        .await
        .expect("seed activity rows");
    }

    async fn wait_for_count(counter: &AtomicUsize, expected: usize) {
        for _ in 0..50 {
            if counter.load(Ordering::SeqCst) >= expected {
                return;
            }
            sleep(StdDuration::from_millis(20)).await;
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn fresh_data_fires_no_refresh(pool: PgPool) {
        seed_rows(&pool, Utc::now()).await;
        let mock = MockForge::success(StdDuration::from_millis(10));
        let forge_client: Arc<dyn ForgeClient + Send + Sync> = mock.clone();
        let state = app_state_with(pool, forge_config(3600, 900), forge_client);

        let items = state
            .activity_cache
            .ask(GetRankedActivity {
                limit: None,
                featured_only: false,
            })
            .await
            .expect("ask activity cache");
        assert_eq!(items.len(), 2);

        for _ in 0..5 {
            tokio::task::yield_now().await;
        }
        assert_eq!(mock.fetch_count(), 0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn stale_data_single_flight_under_concurrency(pool: PgPool) {
        seed_rows(&pool, Utc::now() - Duration::hours(2)).await;
        let mock = MockForge::success(StdDuration::from_millis(50));
        let forge_client: Arc<dyn ForgeClient + Send + Sync> = mock.clone();
        let state = app_state_with(pool, forge_config(3600, 900), forge_client);

        let mut handles = Vec::new();
        for _ in 0..20 {
            let cache = state.activity_cache.clone();
            handles.push(tokio::spawn(async move {
                cache
                    .ask(GetRankedActivity {
                        limit: None,
                        featured_only: false,
                    })
                    .await
            }));
        }

        let results = timeout(StdDuration::from_secs(1), async {
            let mut results = Vec::new();
            for handle in handles {
                results.push(handle.await.expect("join read task"));
            }
            results
        })
        .await
        .expect("reads should not block on refresh");

        for result in results {
            assert!(!result.expect("ask succeeds").is_empty());
        }

        wait_for_count(&mock.fetch_count, 2).await;
        assert_eq!(mock.fetch_count(), 2);
        assert_eq!(mock.sweep_count(), 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn refresh_error_keeps_prior_data_and_200s(pool: PgPool) {
        seed_rows(&pool, Utc::now() - Duration::hours(2)).await;
        let mock = MockForge::rate_limited();
        let forge_client: Arc<dyn ForgeClient + Send + Sync> = mock.clone();
        let state = app_state_with(pool.clone(), forge_config(3600, 900), forge_client);

        let items = state
            .activity_cache
            .ask(GetRankedActivity {
                limit: None,
                featured_only: false,
            })
            .await
            .expect("ask activity cache");
        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|item| item.state == ActivityState::Open));

        wait_for_count(&mock.fetch_count, 1).await;
        sleep(StdDuration::from_millis(50)).await;
        let calls_after_first_refresh = mock.fetch_count();

        let second = state
            .activity_cache
            .ask(GetRankedActivity {
                limit: None,
                featured_only: false,
            })
            .await
            .expect("second ask activity cache");
        assert!(second.iter().all(|item| item.state == ActivityState::Open));
        assert_eq!(mock.fetch_count(), calls_after_first_refresh);

        let open_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM activity_items WHERE state = 'open'")
                .fetch_one(&pool)
                .await
                .expect("count open rows");
        assert_eq!(open_count, 2);

        let app = test_app(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/activity")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let http_items: Vec<ActivityListItem> = serde_json::from_slice(&body).unwrap();
        assert!(
            http_items
                .iter()
                .all(|item| item.state == ActivityState::Open)
        );
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
    async fn activity_refresh_route_absent_without_feature() {
        let app: Router = Router::new();
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/activity")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
