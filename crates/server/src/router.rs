use axum::{
    Router,
    routing::{delete, get, post, put},
};
use plinth_server::api::admin::auth_middleware;
use plinth_server::{AppState, api};
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;

/// Compose the framework-neutral API and feed surface. The UI entrypoint owns
/// the page fallback and static asset serving; this router owns everything
/// consumed by the CLI and external integrations.
pub fn build_api_router(api_key: Option<String>) -> Router<AppState> {
    Router::new()
        .nest(
            "/api",
            build_admin_router(api_key).merge(build_public_api_router()),
        )
        .merge(build_feed_router())
}

/// Build the admin API router with core routes and brick-specific routes.
/// Rate limiter: ~10 requests per minute per IP.
pub fn build_admin_router(api_key: Option<String>) -> Router<AppState> {
    let admin_governor_conf = GovernorConfigBuilder::default()
        .per_second(6)
        .burst_size(10)
        .finish()
        .unwrap_or_else(|| {
            tracing::error!("Failed to build admin rate limiter config");
            std::process::exit(1);
        });

    let mut admin_router = Router::new()
        .route("/admin/tags", get(api::admin::list_tags))
        .route(
            "/admin/content/{key}",
            put(api::admin::update_site_content).get(api::admin::get_admin_site_content),
        );

    #[cfg(feature = "brick-blog")]
    {
        admin_router = admin_router
            .route(
                "/admin/articles",
                post(plinth_server::bricks::blog::admin::publish_article),
            )
            .route(
                "/admin/articles/{slug}",
                delete(plinth_server::bricks::blog::admin::delete_article),
            )
            .route(
                "/admin/posts/{post_slug}/tags",
                post(plinth_server::bricks::blog::admin::add_tag_to_post),
            )
            .route(
                "/admin/posts/{post_slug}/tags/{tag_slug}",
                delete(plinth_server::bricks::blog::admin::remove_tag_from_post),
            );
    }

    #[cfg(feature = "brick-todo")]
    {
        admin_router = admin_router
            .route(
                "/admin/todos",
                post(plinth_server::bricks::todo::admin::create_todo),
            )
            .route(
                "/admin/todos/{slug}",
                put(plinth_server::bricks::todo::admin::update_todo)
                    .delete(plinth_server::bricks::todo::admin::delete_todo),
            )
            .route(
                "/admin/todos/{todo_slug}/tags",
                post(plinth_server::bricks::todo::admin::add_tag_to_todo),
            )
            .route(
                "/admin/todos/{todo_slug}/tags/{tag_slug}",
                delete(plinth_server::bricks::todo::admin::remove_tag_from_todo),
            );
    }

    #[cfg(feature = "brick-portfolio")]
    {
        admin_router = admin_router
            .route(
                "/admin/portfolio",
                post(plinth_server::bricks::portfolio::admin::publish_portfolio_item),
            )
            .route(
                "/admin/portfolio/sync",
                post(plinth_server::bricks::portfolio::admin::sync_portfolio_items),
            );
    }

    #[cfg(feature = "brick-activity")]
    {
        admin_router = admin_router
            .route(
                "/admin/activity",
                post(plinth_server::bricks::activity::admin::publish_activity_item),
            )
            .route(
                "/admin/activity/{id}",
                delete(plinth_server::bricks::activity::admin::delete_activity_handler)
                    .patch(plinth_server::bricks::activity::admin::patch_activity_handler),
            );
    }

    admin_router = admin_router.layer(axum::middleware::from_fn_with_state(
        api_key,
        auth_middleware,
    ));

    admin_router.layer(GovernorLayer::new(admin_governor_conf))
}

/// Build the public API router with health, image proxy, content, and brick-specific routes.
/// Rate limiter: ~60 requests per minute per IP.
pub fn build_public_api_router() -> Router<AppState> {
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(60)
        .finish()
        .unwrap_or_else(|| {
            tracing::error!("Failed to build rate limiter config");
            std::process::exit(1);
        });

    let mut public_api_router = Router::new()
        .route("/config", get(api::public::get_site_config))
        .route("/content/{key}", get(api::public::get_site_content))
        .route("/health", get(api::health::health_check))
        .route("/images/{asset_id}", get(api::images::serve_image));

    #[cfg(feature = "brick-blog")]
    {
        public_api_router = public_api_router
            .route(
                "/posts",
                get(plinth_server::bricks::blog::api::list_blog_posts),
            )
            .route(
                "/posts/{slug}",
                get(plinth_server::bricks::blog::api::get_blog_post),
            )
            .route(
                "/posts/tag/{tag}",
                get(plinth_server::bricks::blog::api::list_blog_posts_by_tag),
            )
            .route(
                "/posts/{slug}/series-nav",
                get(plinth_server::bricks::blog::api::get_series_nav),
            )
            .route(
                "/series",
                get(plinth_server::bricks::blog::api::list_series),
            )
            .route(
                "/series/{slug}/posts",
                get(plinth_server::bricks::blog::api::list_series_posts),
            )
            .route("/search", get(api::search::search_articles))
            .route(
                "/articles/{slug}/related",
                get(api::search::related_articles),
            )
            .route("/opinion", get(api::search::track_opinion));
    }

    #[cfg(feature = "brick-portfolio")]
    {
        public_api_router = public_api_router
            .route(
                "/portfolio",
                get(plinth_server::bricks::portfolio::api::list_portfolio_items),
            )
            .route(
                "/portfolio/{slug}",
                get(plinth_server::bricks::portfolio::api::get_portfolio_item),
            );
    }

    #[cfg(feature = "brick-activity")]
    {
        public_api_router = public_api_router
            .route(
                "/activity",
                get(plinth_server::bricks::activity::api::list_activity_items),
            )
            .route(
                "/activity/{id}",
                get(plinth_server::bricks::activity::api::get_activity_item),
            );
    }

    #[cfg(feature = "brick-todo")]
    {
        public_api_router = public_api_router
            .route("/todos", get(plinth_server::bricks::todo::api::list_todos))
            .route(
                "/todos/{slug}",
                get(plinth_server::bricks::todo::api::get_todo),
            )
            .route(
                "/todos/tag/{tag}",
                get(plinth_server::bricks::todo::api::list_todos_by_tag),
            );
    }

    public_api_router.layer(GovernorLayer::new(governor_conf))
}

/// Build the feed/sitemap router with brick-specific feed routes.
pub fn build_feed_router() -> Router<AppState> {
    let mut feed_app = Router::new().route("/sitemap.xml", get(api::feeds::sitemap_xml));

    #[cfg(feature = "brick-blog")]
    {
        feed_app = feed_app
            .route("/feeds/blog.xml", get(api::feeds::blog_feed))
            .route("/feeds/series/{slug}", get(api::feeds::series_feed))
            .route("/feed.xml", get(api::feeds::blog_feed));
    }

    #[cfg(feature = "brick-portfolio")]
    {
        feed_app = feed_app.route("/feeds/projects.xml", get(api::feeds::projects_feed));
    }

    #[cfg(feature = "brick-activity")]
    {
        feed_app = feed_app.route("/feeds/activity.xml", get(api::feeds::activity_feed));
    }

    feed_app
}
