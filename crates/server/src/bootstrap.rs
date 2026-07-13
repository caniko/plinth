//! Framework-neutral backend bootstrap shared by the production web entrypoint
//! and integration tests.

use std::net::SocketAddr;

use kameo::actor::Spawn;
#[cfg(feature = "legacy-leptos")]
use leptos::config::get_configuration;
#[cfg(feature = "brick-blog")]
use tracing::info;
use tracing::{error, warn};

use crate::actors::core_cache::CoreCache;
use crate::config::PlinthConfig;
use crate::services::db;
use crate::{AppState, ImmichConfig, observability};

#[cfg(feature = "brick-activity")]
struct UnavailableForge {
    reason: String,
}

#[cfg(feature = "brick-activity")]
#[async_trait::async_trait]
impl plinth_forge::ForgeClient for UnavailableForge {
    async fn fetch(
        &self,
        _reference: &plinth_forge::ActivityRef,
    ) -> plinth_forge::ForgeResult<plinth_shared::FetchedActivity> {
        Err(plinth_forge::ForgeError::Network(self.reason.clone()))
    }
}

/// Fully initialized backend state plus the stable bind/configuration values
/// needed by a web entrypoint. Actors are owned by the returned state.
#[derive(Clone)]
pub struct Backend {
    pub state: AppState,
    pub api_key: Option<String>,
    pub site_addr: SocketAddr,
}

/// Initialize configuration, persistence, migrations, declarative content, and
/// all enabled brick actors without constructing a UI router.
pub async fn initialize() -> Backend {
    let config = match PlinthConfig::load() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Failed to load configuration: {error}");
            std::process::exit(1);
        }
    };

    let observability_config =
        observability::ObservabilityConfig::from_config(&config.observability);
    if let Err(error) = observability::init_observability(observability_config) {
        eprintln!("Failed to initialize observability: {error}");
        std::process::exit(1);
    }

    let db = match db::init_db(&config.database).await {
        Ok(db) => db,
        Err(error) => {
            error!(%error, "Failed to initialize Postgres");
            std::process::exit(1);
        }
    };

    if let Err(error) = crate::services::migrations::run_migrations(&db).await {
        error!(%error, "Failed to run database migrations");
        std::process::exit(1);
    }

    if let Err(error) = db::seed_sample_data(&db).await {
        warn!(%error, "Failed to seed sample data");
    }

    #[cfg(feature = "brick-blog")]
    if let Some(ref content_dir) = config.content.content_dir {
        match crate::services::declarative_content::load_declarative_articles(
            &db,
            content_dir,
            &config,
        )
        .await
        {
            Ok(stats) => info!(
                inserted = stats.inserted,
                updated = stats.updated,
                deleted = stats.deleted,
                skipped = stats.skipped,
                "Declarative articles loaded"
            ),
            Err(error) => {
                error!(%error, "Failed to load declarative articles");
                std::process::exit(1);
            }
        }
    }

    let core_cache = CoreCache::spawn(CoreCache::new(db.clone()));

    #[cfg(feature = "brick-blog")]
    let blog_cache = {
        use crate::bricks::blog::cache::BlogCache;
        BlogCache::spawn(BlogCache::new(db.clone()))
    };

    #[cfg(feature = "brick-blog")]
    let vector_search = {
        use crate::actors::vector_search::VectorSearch;
        match VectorSearch::new(db.clone(), config.content.vector_truncation) {
            Ok(search) => Some(VectorSearch::spawn(search)),
            Err(error) => {
                warn!(%error, "VectorSearch disabled");
                None
            }
        }
    };

    #[cfg(feature = "brick-blog")]
    if config.content.content_dir.is_some()
        && let Some(ref vector_search) = vector_search
    {
        let db = db.clone();
        let vector_search = vector_search.clone();
        let truncation = config.content.vector_truncation;
        tokio::task::spawn_local(async move {
            crate::services::declarative_content::backfill_embeddings(
                db,
                vector_search,
                truncation,
            )
            .await;
            info!("Embedding backfill task finished");
        });
    }

    #[cfg(feature = "brick-portfolio")]
    let portfolio_cache = {
        use crate::bricks::portfolio::cache::PortfolioCache;
        PortfolioCache::spawn(PortfolioCache::new(db.clone()))
    };

    #[cfg(feature = "brick-activity")]
    let activity_cache = {
        use crate::bricks::activity::cache::ActivityCache;
        use plinth_forge::{CodebergClient, ForgeClient, ForgeRouter, GitHubClient};
        use std::sync::Arc;

        let forge = config.forge.clone();
        let github_token = std::env::var("GITHUB_TOKEN").ok();
        let codeberg_token = std::env::var("CODEBERG_TOKEN").ok();
        let forge_client: Arc<dyn ForgeClient + Send + Sync> = match (
            GitHubClient::with_base_url(forge.github_base_url.clone(), github_token),
            CodebergClient::with_base_url(forge.codeberg_base_url.clone(), codeberg_token),
        ) {
            (Ok(github), Ok(codeberg)) => Arc::new(ForgeRouter { github, codeberg }),
            (github, codeberg) => {
                let reason = format!(
                    "GitHub client: {:?}; Codeberg client: {:?}",
                    github.err(),
                    codeberg.err()
                );
                Arc::new(UnavailableForge { reason })
            }
        };
        ActivityCache::spawn(ActivityCache::new(
            db.clone(),
            config.ranking.clone(),
            forge,
            forge_client,
        ))
    };

    #[cfg(feature = "brick-todo")]
    let todo_cache = {
        use crate::bricks::todo::cache::TodoCache;
        TodoCache::spawn(TodoCache::new(db.clone()))
    };

    let configured_site_addr = std::env::var("PLINTH_SITE_ADDR")
        .or_else(|_| std::env::var("LEPTOS_SITE_ADDR"))
        .ok()
        .and_then(|address| address.parse().ok());

    #[cfg(feature = "legacy-leptos")]
    let (site_addr, leptos_options) = {
        let mut options = match get_configuration(None) {
            Ok(configuration) => configuration.leptos_options,
            Err(error) => {
                error!(%error, "Failed to load compatibility configuration");
                std::process::exit(1);
            }
        };
        if let Some(site_addr) = configured_site_addr {
            options.site_addr = site_addr;
        }
        if let Ok(site_root) = std::env::var("LEPTOS_SITE_ROOT") {
            options.site_root = site_root.into();
        }
        (options.site_addr, options)
    };

    #[cfg(not(feature = "legacy-leptos"))]
    let site_addr = configured_site_addr.unwrap_or_else(|| {
        "127.0.0.1:3000"
            .parse()
            .expect("valid default site address")
    });

    let immich_config = match (
        config.immich.api_url.is_empty(),
        std::env::var("IMMICH_API_KEY").ok(),
    ) {
        (false, Some(api_key)) => Some(ImmichConfig {
            base_url: config.immich.api_url.trim_end_matches('/').to_string(),
            api_key,
        }),
        _ => None,
    };
    let http_client = match reqwest::Client::builder().build() {
        Ok(client) => client,
        Err(error) => {
            error!(%error, "Failed to construct HTTP client");
            std::process::exit(1);
        }
    };
    let site_config = config.to_site_config();
    let api_key = std::env::var("PLINTH_API_KEY").ok();
    Backend {
        state: AppState {
            #[cfg(feature = "legacy-leptos")]
            leptos_options,
            core_cache,
            db,
            immich_config,
            http_client,
            config,
            site_config,
            #[cfg(feature = "brick-blog")]
            blog_cache,
            #[cfg(feature = "brick-blog")]
            vector_search,
            #[cfg(feature = "brick-portfolio")]
            portfolio_cache,
            #[cfg(feature = "brick-activity")]
            activity_cache,
            #[cfg(feature = "brick-todo")]
            todo_cache,
        },
        api_key,
        site_addr,
    }
}
