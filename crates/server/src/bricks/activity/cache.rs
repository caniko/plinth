use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use kameo::Actor;
use kameo::actor::ActorRef;
use kameo::message::{Context, Message};
use plinth_forge::ForgeClient;
use plinth_shared::toml_config::{ForgeConfig, RankingConfig};
use plinth_shared::{ActivityItem, ActivityListItem};
use sqlx::Row;

use crate::PlinthDb;
use crate::bricks::activity::refresh::{self, RefreshOutcome, RefreshTarget};
use crate::services::rows;

const CACHE_TTL: Duration = Duration::from_secs(5 * 60);
const MAX_ITEM_CACHE_SIZE: usize = 500;

#[derive(Clone)]
struct CachedRefreshTarget {
    target: RefreshTarget,
    fetched_at: DateTime<Utc>,
}

// ============================================================================
// PHASE 04 SEAM: lazy stale-while-revalidate refresh.
// This phase (03) intentionally serves only DB-backed cached data. Phase 04
// adds: a single-flight in-progress guard + `last_refresh_attempt`,
// a per-served-entry `fetched_at` TTL check (default 1h from [refresh]/[ranking]
// config), and a non-blocking background refresh that re-pulls forge metadata
// and updates DB + cache. DO NOT block reads on refresh.
// ============================================================================
/// In-memory actor that caches activity items and triggers forge refresh.
#[derive(Actor)]
pub struct ActivityCache {
    db: PlinthDb,
    ranking: RankingConfig,
    items: HashMap<i64, ActivityItem>,
    refresh_targets: HashMap<i64, CachedRefreshTarget>,
    ranked_list_cache: Option<Vec<ActivityListItem>>,
    cache_populated_at: Option<Instant>,
    refreshing: bool,
    backoff_until: Option<Instant>,
    ttl: Duration,
    backoff: Duration,
    forge_client: Arc<dyn ForgeClient + Send + Sync>,
}

impl ActivityCache {
    /// Create a new ActivityCache actor.
    pub fn new(
        db: PlinthDb,
        ranking: RankingConfig,
        forge: ForgeConfig,
        forge_client: Arc<dyn ForgeClient + Send + Sync>,
    ) -> Self {
        Self {
            db,
            ranking,
            items: HashMap::new(),
            refresh_targets: HashMap::new(),
            ranked_list_cache: None,
            cache_populated_at: None,
            refreshing: false,
            backoff_until: None,
            ttl: Duration::from_secs(forge.refresh_ttl_secs),
            backoff: Duration::from_secs(forge.refresh_backoff_secs),
            forge_client,
        }
    }

    fn is_expired(&self) -> bool {
        self.cache_populated_at
            .is_some_and(|t| t.elapsed() > CACHE_TTL)
    }

    fn clear_all(&mut self) {
        self.items.clear();
        self.refresh_targets.clear();
        self.ranked_list_cache = None;
        self.cache_populated_at = None;
    }

    fn touch(&mut self) {
        if self.cache_populated_at.is_none() {
            self.cache_populated_at = Some(Instant::now());
        }
    }

    fn expire_if_stale(&mut self) {
        if self.is_expired() {
            self.clear_all();
        }
    }

    fn target_is_stale(&self, fetched_at: DateTime<Utc>) -> bool {
        Utc::now()
            .signed_duration_since(fetched_at)
            .to_std()
            .is_ok_and(|age| age > self.ttl)
    }

    fn is_stale(&self) -> bool {
        self.refresh_targets
            .values()
            .any(|target| self.target_is_stale(target.fetched_at))
    }

    fn in_backoff(&self) -> bool {
        self.backoff_until.is_some_and(|t| Instant::now() < t)
    }

    fn refresh_targets(&self) -> Vec<RefreshTarget> {
        self.refresh_targets
            .values()
            .map(|target| target.target.clone())
            .collect()
    }

    fn cache_refresh_target(&mut self, item: &ActivityItem) {
        self.refresh_targets.insert(
            item.id,
            CachedRefreshTarget {
                target: RefreshTarget {
                    id: item.id,
                    forge: item.forge,
                    repo_owner: item.repo_owner.clone(),
                    repo_name: item.repo_name.clone(),
                    kind: item.kind,
                    number: item.number,
                },
                fetched_at: item.fetched_at,
            },
        );
    }

    async fn load_refresh_targets(&mut self) -> Result<(), String> {
        let rows = sqlx::query(
            r#"
            SELECT id, forge, repo_owner, repo_name, kind, number, fetched_at
            FROM activity_items
            WHERE published = true
            "#,
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| format!("Database error: {e}"))?;

        self.refresh_targets.clear();
        for row in rows {
            let id = row
                .try_get::<i64, _>("id")
                .map_err(|e| format!("Database error: {e}"))?;
            let forge = parse_activity_enum(row.try_get("forge").map_err(db_error)?)?;
            let kind = parse_activity_enum(row.try_get("kind").map_err(db_error)?)?;
            self.refresh_targets.insert(
                id,
                CachedRefreshTarget {
                    target: RefreshTarget {
                        id,
                        forge,
                        repo_owner: row.try_get("repo_owner").map_err(db_error)?,
                        repo_name: row.try_get("repo_name").map_err(db_error)?,
                        kind,
                        number: row.try_get("number").map_err(db_error)?,
                    },
                    fetched_at: row.try_get("fetched_at").map_err(db_error)?,
                },
            );
        }
        Ok(())
    }

    async fn query_ranked(
        &self,
        featured_only: bool,
        limit: Option<i64>,
    ) -> Result<Vec<ActivityListItem>, String> {
        crate::bricks::activity::ranking::query_ranked_list(
            &self.db,
            &self.ranking,
            featured_only,
            limit,
        )
        .await
        .map_err(|e| format!("Database error: {e}"))
    }

    fn maybe_trigger_refresh(&mut self, me: ActorRef<ActivityCache>) {
        if !self.is_stale() || self.refreshing || self.in_backoff() {
            return;
        }

        let targets = self.refresh_targets();
        if targets.is_empty() {
            return;
        }

        self.refreshing = true;
        let db = self.db.clone();
        let forge_client = Arc::clone(&self.forge_client);
        let ranking = self.ranking.clone();

        tokio::spawn(async move {
            let outcome = match tokio::spawn(async move {
                refresh::run_refresh(db, forge_client, ranking, targets).await
            })
            .await
            {
                Ok(outcome) => outcome,
                Err(join_err) => RefreshOutcome::Failed {
                    reason: format!("refresh task panicked/aborted: {join_err}"),
                },
            };
            let _ = me.tell(RefreshDone(outcome)).await;
        });
    }
}

fn db_error(e: sqlx::Error) -> String {
    format!("Database error: {e}")
}

fn parse_activity_enum<T>(value: String) -> Result<T, String>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value.parse::<T>().map_err(|e| {
        db_error(sqlx::Error::Decode(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        ))))
    })
}

/// Ranked public list. `limit`/`featured_only` come from query params.
pub struct GetRankedActivity {
    pub limit: Option<i64>,
    pub featured_only: bool,
}

impl Message<GetRankedActivity> for ActivityCache {
    type Reply = Result<Vec<ActivityListItem>, String>;

    async fn handle(
        &mut self,
        msg: GetRankedActivity,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // ============================================================================
        // PHASE 04 SEAM: lazy stale-while-revalidate refresh.
        // This phase (03) intentionally serves only DB-backed cached data. Phase 04
        // adds: a single-flight in-progress guard + `last_refresh_attempt`,
        // a per-served-entry `fetched_at` TTL check (default 1h from [refresh]/[ranking]
        // config), and a non-blocking background refresh that re-pulls forge metadata
        // and updates DB + cache. DO NOT block reads on refresh.
        // ============================================================================
        self.expire_if_stale();

        let items = if msg.featured_only {
            let list = self.query_ranked(true, msg.limit).await?;
            self.load_refresh_targets().await?;
            apply_limit(list, msg.limit)
        } else {
            if self.ranked_list_cache.is_none() {
                let list = self.query_ranked(false, None).await?;
                self.ranked_list_cache = Some(list);
                self.load_refresh_targets().await?;
                self.touch();
            }
            apply_limit(
                self.ranked_list_cache.clone().unwrap_or_default(),
                msg.limit,
            )
        };

        let me = ctx.actor_ref().clone();
        self.maybe_trigger_refresh(me);
        Ok(items)
    }
}

/// Fetch a single activity item by id from cache or database.
pub struct GetActivityItem(pub i64);

impl Message<GetActivityItem> for ActivityCache {
    type Reply = Result<Option<ActivityItem>, String>;

    async fn handle(
        &mut self,
        msg: GetActivityItem,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.expire_if_stale();
        let id = msg.0;

        let item = if let Some(item) = self.items.get(&id) {
            Some(item.clone())
        } else {
            let row = sqlx::query(
                "SELECT * FROM activity_items WHERE id = $1 AND published = true LIMIT 1",
            )
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| format!("Database error: {e}"))?;

            let item = row
                .map(rows::activity_item)
                .transpose()
                .map_err(|e| format!("Database error: {e}"))?;

            if let Some(ref item) = item {
                self.cache_refresh_target(item);
                if self.items.len() < MAX_ITEM_CACHE_SIZE {
                    self.items.insert(id, item.clone());
                    self.touch();
                }
            }
            item
        };

        let me = ctx.actor_ref().clone();
        self.maybe_trigger_refresh(me);
        Ok(item)
    }
}

/// Signal the cache actor to clear all cached data.
pub struct ActivityInvalidateCache;

impl Message<ActivityInvalidateCache> for ActivityCache {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: ActivityInvalidateCache,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.clear_all();
    }
}

/// Lightweight fire-and-forget trigger: consider a stale-while-revalidate refresh
/// WITHOUT returning data. Sent by the SSR page read path (via
/// [`ActivityRefreshHandle`]) so a page visit drives freshness, mirroring what the
/// `GetRankedActivity`/`GetActivityItem` read handlers do internally.
pub struct PokeRefresh;

impl Message<PokeRefresh> for ActivityCache {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: PokeRefresh,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Ensure refresh targets exist so staleness can be evaluated even when no
        // prior read has populated them (cold actor / page-only traffic).
        if self.refresh_targets.is_empty()
            && let Err(e) = self.load_refresh_targets().await
        {
            tracing::warn!(error = %e, "activity poke: failed to load refresh targets");
            return;
        }
        let me = ctx.actor_ref().clone();
        self.maybe_trigger_refresh(me);
    }
}

/// Type-erased [`plinth_shared::ActivityRefreshHook`] backed by the cache actor.
///
/// Installed into the Leptos SSR context by `main` so the activity page's
/// `#[server]` functions (which read the database directly, per the established
/// plinth pattern) can still trigger the actor's stale-while-revalidate refresh on
/// a visit, without the WASM client depending on this crate.
#[derive(Clone)]
pub struct ActivityRefreshHandle(pub ActorRef<ActivityCache>);

impl plinth_shared::ActivityRefreshHook for ActivityRefreshHandle {
    fn poke(&self) {
        let actor = self.0.clone();
        tokio::spawn(async move {
            let _ = actor.tell(PokeRefresh).await;
        });
    }
}

/// Message sent back to the cache actor when a background refresh completes.
pub struct RefreshDone(pub RefreshOutcome);

impl Message<RefreshDone> for ActivityCache {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: RefreshDone,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.refreshing = false;
        match msg.0 {
            RefreshOutcome::Refreshed {
                items,
                refreshed_at,
            } => {
                self.ranked_list_cache = Some(items);
                self.items.clear();
                for target in self.refresh_targets.values_mut() {
                    target.fetched_at = refreshed_at;
                }
                self.cache_populated_at = Some(Instant::now());
                self.backoff_until = None;
            }
            RefreshOutcome::Failed { reason } => {
                tracing::warn!(
                    %reason,
                    "activity refresh failed; keeping stale data and backing off"
                );
                self.backoff_until = Some(Instant::now() + self.backoff);
            }
        }
    }
}

fn apply_limit(mut list: Vec<ActivityListItem>, limit: Option<i64>) -> Vec<ActivityListItem> {
    if let Some(n) = limit {
        list.truncate(n.max(0) as usize);
    }
    list
}
