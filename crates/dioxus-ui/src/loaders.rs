//! Fullstack loaders for the public Plinth read model.
//!
//! These functions deliberately sit at the UI boundary instead of duplicating
//! SQL in components.  During SSR they consume the request's `AppState` and
//! read through the existing cache actors; in the browser Dioxus serializes the
//! result into the hydration payload and uses the generated server-function
//! endpoint for later navigations.

use dioxus::prelude::*;

#[server(endpoint = "/api/dioxus/config")]
pub async fn load_site_config() -> ServerFnResult<plinth_shared::SiteConfig> {
    #[cfg(feature = "server")]
    {
        let state = backend()?;
        return Ok(state.site_config.clone());
    }

    #[cfg(not(feature = "server"))]
    {
        unreachable!("server function body is replaced on the web target")
    }
}

#[cfg(feature = "server")]
fn backend() -> Result<plinth_server::AppState, ServerFnError> {
    try_consume_context::<plinth_server::AppState>()
        .ok_or_else(|| ServerFnError::new("Plinth backend context is unavailable"))
}

#[cfg(feature = "server")]
fn actor_error(error: impl std::fmt::Display) -> ServerFnError {
    ServerFnError::new(error)
}

#[server(endpoint = "/api/dioxus/content")]
pub async fn load_site_content(key: String) -> ServerFnResult<Option<plinth_shared::SiteContent>> {
    #[cfg(feature = "server")]
    {
        use plinth_server::actors::core_cache::GetSiteContent;

        let state = backend()?;
        let result = state
            .core_cache
            .ask(GetSiteContent(key))
            .await
            .map_err(actor_error)?;
        return Ok(result);
    }

    #[cfg(not(feature = "server"))]
    {
        unreachable!("server function body is replaced on the web target")
    }
}

#[cfg(feature = "brick-blog")]
#[server(endpoint = "/api/dioxus/posts")]
pub async fn load_posts() -> ServerFnResult<Vec<plinth_shared::BlogListItem>> {
    #[cfg(feature = "server")]
    {
        use plinth_server::bricks::blog::cache::GetAllBlogPosts;

        let state = backend()?;
        let result = state
            .blog_cache
            .ask(GetAllBlogPosts)
            .await
            .map_err(actor_error)?;
        return Ok(result);
    }

    #[cfg(not(feature = "server"))]
    {
        unreachable!("server function body is replaced on the web target")
    }
}

#[cfg(feature = "brick-blog")]
#[server(endpoint = "/api/dioxus/post")]
pub async fn load_post(slug: String) -> ServerFnResult<Option<plinth_shared::BlogPost>> {
    #[cfg(feature = "server")]
    {
        use plinth_server::bricks::blog::cache::GetBlogPost;

        let state = backend()?;
        let result = state
            .blog_cache
            .ask(GetBlogPost(slug))
            .await
            .map_err(actor_error)?;
        return Ok(result);
    }

    #[cfg(not(feature = "server"))]
    {
        unreachable!("server function body is replaced on the web target")
    }
}

#[cfg(feature = "brick-blog")]
#[server(endpoint = "/api/dioxus/posts/tag")]
pub async fn load_posts_by_tag(tag: String) -> ServerFnResult<Vec<plinth_shared::BlogListItem>> {
    #[cfg(feature = "server")]
    {
        use plinth_server::bricks::blog::cache::GetPostsByTag;

        let state = backend()?;
        let result = state
            .blog_cache
            .ask(GetPostsByTag(tag))
            .await
            .map_err(actor_error)?;
        return Ok(result);
    }

    #[cfg(not(feature = "server"))]
    {
        unreachable!("server function body is replaced on the web target")
    }
}

#[cfg(feature = "brick-blog")]
#[server(endpoint = "/api/dioxus/series")]
pub async fn load_series() -> ServerFnResult<Vec<plinth_shared::SeriesListItem>> {
    #[cfg(feature = "server")]
    {
        use plinth_server::bricks::blog::cache::GetAllSeries;

        let state = backend()?;
        let result = state
            .blog_cache
            .ask(GetAllSeries)
            .await
            .map_err(actor_error)?;
        return Ok(result);
    }

    #[cfg(not(feature = "server"))]
    {
        unreachable!("server function body is replaced on the web target")
    }
}

#[cfg(feature = "brick-blog")]
#[server(endpoint = "/api/dioxus/series/posts")]
pub async fn load_series_posts(slug: String) -> ServerFnResult<Vec<plinth_shared::BlogListItem>> {
    #[cfg(feature = "server")]
    {
        use plinth_server::bricks::blog::cache::GetSeriesPosts;

        let state = backend()?;
        let result = state
            .blog_cache
            .ask(GetSeriesPosts(slug))
            .await
            .map_err(actor_error)?;
        return Ok(result);
    }

    #[cfg(not(feature = "server"))]
    {
        unreachable!("server function body is replaced on the web target")
    }
}

#[cfg(feature = "brick-portfolio")]
#[server(endpoint = "/api/dioxus/projects")]
pub async fn load_projects() -> ServerFnResult<Vec<plinth_shared::PortfolioItem>> {
    #[cfg(feature = "server")]
    {
        use plinth_server::bricks::portfolio::cache::GetAllPortfolioItems;

        let state = backend()?;
        let result = state
            .portfolio_cache
            .ask(GetAllPortfolioItems)
            .await
            .map_err(actor_error)?;
        return Ok(result);
    }

    #[cfg(not(feature = "server"))]
    {
        unreachable!("server function body is replaced on the web target")
    }
}

#[cfg(feature = "brick-portfolio")]
#[server(endpoint = "/api/dioxus/project")]
pub async fn load_project(slug: String) -> ServerFnResult<Option<plinth_shared::PortfolioItem>> {
    #[cfg(feature = "server")]
    {
        use plinth_server::bricks::portfolio::cache::GetPortfolioItem;

        let state = backend()?;
        let result = state
            .portfolio_cache
            .ask(GetPortfolioItem(slug))
            .await
            .map_err(actor_error)?;
        return Ok(result);
    }

    #[cfg(not(feature = "server"))]
    {
        unreachable!("server function body is replaced on the web target")
    }
}

#[cfg(feature = "brick-activity")]
#[server(endpoint = "/api/dioxus/activity")]
pub async fn load_activity() -> ServerFnResult<Vec<plinth_shared::ActivityListItem>> {
    #[cfg(feature = "server")]
    {
        use plinth_server::bricks::activity::{cache::PokeRefresh, ranking};

        if let Ok(delay) = std::env::var("PLINTH_TEST_ACTIVITY_DELAY_MS")
            && let Ok(delay) = delay.parse::<u64>()
        {
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        }
        let state = backend()?;
        let result = ranking::query_ranked_list(&state.db, &state.config.ranking, false, Some(50))
            .await
            .map_err(actor_error)?;
        // Refresh is deliberately fire-and-forget: SSR freshness must not
        // wait for a forge network round trip.
        let _ = state.activity_cache.tell(PokeRefresh).await;
        return Ok(result);
    }

    #[cfg(not(feature = "server"))]
    {
        unreachable!("server function body is replaced on the web target")
    }
}

#[cfg(feature = "brick-activity")]
#[server(endpoint = "/api/dioxus/activity/item")]
pub async fn load_activity_item(id: i64) -> ServerFnResult<Option<plinth_shared::ActivityItem>> {
    #[cfg(feature = "server")]
    {
        use plinth_server::bricks::activity::{cache::PokeRefresh, ranking};

        let state = backend()?;
        let result = ranking::query_item(&state.db, id)
            .await
            .map_err(actor_error)?;
        let _ = state.activity_cache.tell(PokeRefresh).await;
        return Ok(result);
    }

    #[cfg(not(feature = "server"))]
    {
        unreachable!("server function body is replaced on the web target")
    }
}

#[cfg(feature = "brick-todo")]
#[server(endpoint = "/api/dioxus/todos")]
pub async fn load_todos() -> ServerFnResult<Vec<plinth_shared::TodoListItem>> {
    #[cfg(feature = "server")]
    {
        use plinth_server::bricks::todo::cache::GetAllTodos;

        let state = backend()?;
        let result = state
            .todo_cache
            .ask(GetAllTodos)
            .await
            .map_err(actor_error)?;
        return Ok(result);
    }

    #[cfg(not(feature = "server"))]
    {
        unreachable!("server function body is replaced on the web target")
    }
}

#[cfg(feature = "brick-todo")]
#[server(endpoint = "/api/dioxus/todo")]
pub async fn load_todo(slug: String) -> ServerFnResult<Option<plinth_shared::TodoItem>> {
    #[cfg(feature = "server")]
    {
        use plinth_server::bricks::todo::cache::GetTodoItem;

        let state = backend()?;
        let result = state
            .todo_cache
            .ask(GetTodoItem(slug))
            .await
            .map_err(actor_error)?;
        return Ok(result);
    }

    #[cfg(not(feature = "server"))]
    {
        unreachable!("server function body is replaced on the web target")
    }
}

#[cfg(feature = "brick-todo")]
#[server(endpoint = "/api/dioxus/todos/tag")]
pub async fn load_todos_by_tag(tag: String) -> ServerFnResult<Vec<plinth_shared::TodoListItem>> {
    #[cfg(feature = "server")]
    {
        use plinth_server::bricks::todo::cache::GetTodosByTag;

        let state = backend()?;
        let result = state
            .todo_cache
            .ask(GetTodosByTag(tag))
            .await
            .map_err(actor_error)?;
        return Ok(result);
    }

    #[cfg(not(feature = "server"))]
    {
        unreachable!("server function body is replaced on the web target")
    }
}
