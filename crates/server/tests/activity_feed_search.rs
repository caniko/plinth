#[cfg(feature = "brick-activity")]
mod common;

#[cfg(feature = "brick-activity")]
mod enabled {
    use crate::common;

    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
        routing::get,
    };
    use chrono::{TimeZone, Utc};
    use kameo::actor::Spawn;
    use leptos::config::LeptosOptions;
    use pgvector::Vector;
    use plinth_forge::{ActivityRef, ForgeClient, ForgeError, ForgeResult};
    use plinth_server::{
        AppState,
        actors::{core_cache::CoreCache, vector_search::search_activity_by_vector},
        api::feeds::activity_feed,
        bricks::activity::cache::ActivityCache,
        config::PlinthConfig,
    };
    use plinth_shared::FetchedActivity;
    use plinth_shared::RankingStrategy;
    use plinth_shared::toml_config::RankingConfig;
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
                "not used in phase 07 feed/search tests".to_string(),
            ))
        }
    }

    fn app_state(pool: PgPool) -> AppState {
        let config = PlinthConfig::default();
        let site_config = config.to_site_config();
        let forge = config.forge.clone();
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
                Arc::new(NoopForge),
            )),
            #[cfg(feature = "brick-todo")]
            todo_cache: TodoCache::spawn(TodoCache::new(pool)),
        }
    }

    fn test_app(state: AppState) -> Router {
        Router::new()
            .route("/feeds/activity.xml", get(activity_feed))
            .with_state(state)
    }

    fn unit_vector() -> Vec<f32> {
        let mut v = vec![0.0_f32; 384];
        v[0] = 1.0;
        v
    }

    async fn seed_activity(pool: &PgPool, title: &str, embedding: Vec<f32>) -> i64 {
        sqlx::query_scalar(
            r#"
            INSERT INTO activity_items (
                forge, repo_owner, repo_name, kind, number, url, title, body, state,
                created_at, closed_at, merged_at, impact, additions, deletions,
                comments_count, labels, repo_stars, embedding, fetched_at,
                featured, published, content_hash
            )
            VALUES (
                'github', 'caniko', 'plinth', 'pr', 7007, 'https://github.com/caniko/plinth/pull/7007',
                $1, NULL, 'merged', $2, NULL, $2, 5, NULL, NULL, NULL,
                ARRAY['parser']::text[], NULL, $3, now(), false, true, NULL
            )
            RETURNING id
            "#,
        )
        .bind(title)
        .bind(Utc.with_ymd_and_hms(2026, 6, 1, 12, 0, 0).unwrap())
        .bind(Vector::from(embedding))
        .fetch_one(pool)
        .await
        .expect("seed activity row")
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn activity_feed_returns_valid_xml_with_entries(pool: PgPool) {
        seed_activity(&pool, "Fix the parser", unit_vector()).await;
        let app = test_app(app_state(pool));

        let resp = app
            .oneshot(
                Request::builder()
                    .uri("/feeds/activity.xml")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let content_type = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(content_type.starts_with("application/rss+xml"));
        let cache_control = resp
            .headers()
            .get(header::CACHE_CONTROL)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(cache_control, "public, max-age=3600");

        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let xml = String::from_utf8(body.to_vec()).unwrap();
        assert!(xml.contains("<rss"));
        assert!(xml.contains("<channel>"));
        assert!(xml.contains("Fix the parser"));

        let channel = rss::Channel::read_from(xml.as_bytes()).expect("valid RSS");
        assert!(
            channel
                .items()
                .iter()
                .any(|item| item.title() == Some("Fix the parser"))
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn search_returns_seeded_activity_above_min_similarity(pool: PgPool) {
        let embedding = unit_vector();
        let id = seed_activity(&pool, "Fix the parser", embedding.clone()).await;
        let config = PlinthConfig::default();

        let hits = search_activity_by_vector(&pool, embedding, 10, config.search.min_similarity)
            .await
            .expect("activity search");

        let (item, similarity) = hits
            .iter()
            .find(|(item, _)| item.id == id)
            .expect("seeded activity hit");
        assert_eq!(item.title, "Fix the parser");
        assert!(*similarity >= config.search.min_similarity);
        assert_eq!(*similarity, 1.0);
    }
}

#[cfg(not(feature = "brick-activity"))]
mod disabled {
    #[test]
    fn brick_activity_disabled_compiles() {}
}
