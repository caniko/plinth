#![cfg(all(
    feature = "brick-blog",
    feature = "brick-portfolio",
    feature = "brick-todo",
    feature = "brick-activity",
    not(feature = "legacy-leptos")
))]

mod common;

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use kameo::actor::Spawn;
use plinth_server::{
    AppState,
    actors::core_cache::CoreCache,
    bricks::{
        activity::cache::ActivityCache,
        blog::{
            admin::{list_drafts, publish_article, replace_draft},
            cache::BlogCache,
        },
        portfolio::cache::PortfolioCache,
        todo::cache::TodoCache,
    },
    config::PlinthConfig,
    error::PlinthError,
};
use plinth_shared::{PublishArticleRequest, RankingStrategy, toml_config::RankingConfig};
use sqlx::PgPool;

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
        core_cache: CoreCache::spawn(CoreCache::new(pool.clone())),
        db: pool.clone(),
        immich_config: None,
        http_client: common::test_http_client(),
        config,
        site_config,
        blog_cache: BlogCache::spawn(BlogCache::new(pool.clone())),
        vector_search: None,
        portfolio_cache: PortfolioCache::spawn(PortfolioCache::new(pool.clone())),
        activity_cache: ActivityCache::spawn(ActivityCache::new(
            pool.clone(),
            ranking,
            forge,
            Arc::new(common::NoopForge),
        )),
        todo_cache: TodoCache::spawn(TodoCache::new(pool)),
    }
}

fn post(
    slug: &str,
    title: &str,
    body: &str,
    tags: &[&str],
    published: bool,
) -> PublishArticleRequest {
    PublishArticleRequest {
        title: Some(title.to_string()),
        content: body.to_string(),
        slug: Some(slug.to_string()),
        description: None,
        author: None,
        tags: Some(tags.iter().map(|tag| tag.to_string()).collect()),
        featured: None,
        published: Some(published),
        embedding: None,
        content_format: None,
        html_content: None,
        series: None,
        series_title: None,
        series_position: None,
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn drafts_are_listed_and_replaced_by_id_only(pool: PgPool) {
    let state = app_state(pool.clone());
    let create = |request| publish_article(State(state.clone()), Json(request));
    let replace = |id: i64, request| replace_draft(State(state.clone()), Path(id), Json(request));
    let drafts = || async {
        list_drafts(State(state.clone()))
            .await
            .expect("list drafts")
            .0
    };

    let _ = create(post("live", "Live", "Out there.", &[], true))
        .await
        .expect("publish");
    let live_id: i64 = sqlx::query_scalar("SELECT id FROM blog_posts WHERE slug = 'live'")
        .fetch_one(&pool)
        .await
        .unwrap();
    let mut first = post("idea", "Idea", "First pass.", &["rust", "old"], false);
    first.author = Some("Guest".to_string());
    first.featured = Some(true);
    first.series = Some("notes".to_string());
    first.series_position = Some(3);
    let _ = create(first).await.expect("save a draft");

    let listed = drafts().await;
    assert_eq!(listed.len(), 1, "public posts are not drafts");
    assert_eq!(listed[0].slug, "idea");
    assert_eq!(listed[0].content.trim(), "First pass.");
    assert_eq!(listed[0].content_format, "markdown");
    let opened = listed[0].id;

    // A plain create never overwrites, not even a draft.
    let taken = create(post("idea", "Other", "Clash.", &[], false)).await;
    assert!(matches!(taken, Err(PlinthError::Conflict(_))));

    // Replacing edits the draft, tags included, and keeps the fields the
    // editor did not send.
    let _ = replace(
        opened,
        post("idea", "Idea, revised", "Second pass.", &["rust"], false),
    )
    .await
    .expect("replace the draft");
    let listed = drafts().await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].title, "Idea, revised");
    assert_eq!(listed[0].tags, vec!["rust"]);
    assert_eq!(
        common::blog_tag_names(&pool, "idea").await.unwrap(),
        vec!["rust"]
    );
    let (author, featured, series, position): (String, bool, Option<String>, Option<i32>) =
        sqlx::query_as(
            "SELECT author, featured, series_slug, series_position FROM blog_posts WHERE slug = 'idea'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!((author.as_str(), featured), ("Guest", true));
    assert_eq!((series.as_deref(), position), (Some("notes"), Some(3)));

    // The id changed: an editor still holding the first copy cannot replace
    // the newer one.
    assert_ne!(listed[0].id, opened);
    let stale = replace(opened, post("idea", "Stale", "Old.", &[], false)).await;
    assert!(matches!(stale, Err(PlinthError::Gone(_))));

    // Replacing can rename and promote in one step.
    let _ = replace(
        listed[0].id,
        post("final", "Final", "Third pass.", &["rust"], true),
    )
    .await
    .expect("promote the draft");
    assert!(drafts().await.is_empty());
    let published: bool =
        sqlx::query_scalar("SELECT published FROM blog_posts WHERE slug = 'final'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(published);

    // A published post is never replaced, and a rename onto it is refused
    // without losing the draft.
    let gone = replace(live_id, post("live", "Other", "Clash.", &[], true)).await;
    assert!(matches!(gone, Err(PlinthError::Gone(_))));
    let _ = create(post("next", "Next", "Draft.", &[], false))
        .await
        .expect("draft");
    let next = drafts().await[0].id;
    let onto = replace(next, post("live", "Next", "Draft.", &[], true)).await;
    assert!(matches!(onto, Err(PlinthError::Conflict(_))));
    assert_eq!(drafts().await.len(), 1, "a refused replace keeps the draft");
    let title: String = sqlx::query_scalar("SELECT title FROM blog_posts WHERE slug = 'live'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(title, "Live");
}
