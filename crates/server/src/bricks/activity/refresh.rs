//! Background refresh worker for the activity cache.
//!
//! Runs off the actor mailbox so public reads never wait on forge/network I/O.
//! The refresh writes only forge-derived columns and never touches embeddings.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use plinth_forge::{ActivityRef, ForgeClient, ForgeError};
use plinth_shared::toml_config::RankingConfig;
use plinth_shared::{ActivityKind, ActivityListItem, Forge};
use tracing::{info, instrument, warn};

use crate::PlinthDb;

/// Identifies one row to refresh by its natural forge key.
#[derive(Clone, Debug)]
pub struct RefreshTarget {
    pub id: i64,
    pub forge: Forge,
    pub repo_owner: String,
    pub repo_name: String,
    pub kind: ActivityKind,
    pub number: i32,
}

/// Outcome of a single refresh sweep, reported back to the actor.
pub enum RefreshOutcome {
    /// All targets refreshed; the actor should swap in the re-ranked list.
    Refreshed {
        items: Vec<ActivityListItem>,
        refreshed_at: DateTime<Utc>,
    },
    /// The sweep failed. The actor keeps prior data and starts a backoff window.
    Failed { reason: String },
}

/// Re-pull every target from its forge and write a narrow UPDATE per row.
#[instrument(skip(db, client, ranking, targets), fields(n = targets.len()))]
pub async fn run_refresh(
    db: PlinthDb,
    client: Arc<dyn ForgeClient + Send + Sync>,
    ranking: RankingConfig,
    targets: Vec<RefreshTarget>,
) -> RefreshOutcome {
    let refreshed_at = Utc::now();
    let mut rate_limited = false;

    for t in &targets {
        let activity_ref = ActivityRef {
            forge: t.forge,
            owner: t.repo_owner.clone(),
            repo: t.repo_name.clone(),
            kind: t.kind,
            number: t.number,
        };

        let fetched = match client.fetch(&activity_ref).await {
            Ok(fetched) => fetched,
            Err(ForgeError::NotFound { .. }) => {
                warn!(
                    id = t.id,
                    "activity refresh: upstream gone, keeping last-known data"
                );
                continue;
            }
            Err(ForgeError::RateLimited { .. }) => {
                warn!(id = t.id, "activity refresh: rate limited, backing off");
                rate_limited = true;
                break;
            }
            Err(e) => {
                return RefreshOutcome::Failed {
                    reason: format!("forge fetch failed: {e}"),
                };
            }
        };

        let res = sqlx::query(
            r#"
            UPDATE activity_items
            SET state = $2,
                merged_at = $3,
                closed_at = $4,
                additions = $5,
                deletions = $6,
                comments_count = $7,
                repo_stars = $8,
                labels = $9,
                fetched_at = now()
            WHERE id = $1
            "#,
        )
        .bind(t.id)
        .bind(fetched.state.as_str())
        .bind(fetched.merged_at)
        .bind(fetched.closed_at)
        .bind(fetched.additions)
        .bind(fetched.deletions)
        .bind(fetched.comments_count)
        .bind(fetched.repo_stars)
        .bind(&fetched.labels)
        .execute(&db)
        .await;

        if let Err(e) = res {
            return RefreshOutcome::Failed {
                reason: format!("refresh UPDATE failed: {e}"),
            };
        }
    }

    if rate_limited {
        return RefreshOutcome::Failed {
            reason: "rate limited mid-sweep".to_string(),
        };
    }

    match reread_ranked(&db, &ranking).await {
        Ok(items) => {
            info!(n = targets.len(), "activity refresh complete");
            RefreshOutcome::Refreshed {
                items,
                refreshed_at,
            }
        }
        Err(e) => RefreshOutcome::Failed {
            reason: format!("re-read after refresh failed: {e}"),
        },
    }
}

async fn reread_ranked(
    db: &PlinthDb,
    ranking: &RankingConfig,
) -> Result<Vec<ActivityListItem>, sqlx::Error> {
    crate::bricks::activity::ranking::query_ranked_list(db, ranking, false, None).await
}
