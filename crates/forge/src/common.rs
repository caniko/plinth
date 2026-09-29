//! Shared HTTP/status helpers for forge clients.
//!
//! Extracted from the near-identical `codeberg`/`github` implementations.
//! Status mapping, retry parsing, timestamp/state normalization, and the
//! send → status → JSON pipeline differ only in the [`Forge`] variant —
//! except GitHub's extra 403 rate-limit arm, kept here behind a variant check.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use plinth_shared::{ActivityState, Forge};

use crate::{ForgeError, ForgeResult};

/// Map a forge HTTP response to success or a typed [`ForgeError`].
pub(crate) async fn map_status(
    resp: reqwest::Response,
    forge: Forge,
) -> Result<reqwest::Response, ForgeError> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let code = status.as_u16();
    let url = resp.url().to_string();
    if code == 403 && forge == Forge::GitHub && rate_limit_exhausted(&resp) {
        return Err(ForgeError::RateLimited {
            forge,
            retry_after: retry_after_for(&resp, forge),
        });
    }
    Err(match code {
        404 | 410 => ForgeError::NotFound {
            forge,
            url,
            status: code,
        },
        429 => ForgeError::RateLimited {
            forge,
            retry_after: retry_after_for(&resp, forge),
        },
        _ => {
            let body = resp.text().await.unwrap_or_default();
            ForgeError::Http {
                forge,
                status: code,
                body,
            }
        }
    })
}

/// Send a request, check status, and decode the JSON body.
pub(crate) async fn fetch_json<T: serde::de::DeserializeOwned>(
    request: reqwest::RequestBuilder,
    forge: Forge,
) -> ForgeResult<T> {
    let resp = request
        .send()
        .await
        .map_err(|e| ForgeError::Network(e.to_string()))?;
    map_status(resp, forge)
        .await?
        .json::<T>()
        .await
        .map_err(|e| ForgeError::Decode(e.to_string()))
}

/// Resolve `merged_at`, logging when a forge claims merged without a timestamp.
pub(crate) fn merge_timestamp(
    forge: Forge,
    merged: Option<bool>,
    merged_at: Option<DateTime<Utc>>,
) -> Option<DateTime<Utc>> {
    if merged == Some(true) && merged_at.is_none() {
        tracing::debug!(forge = ?forge, "forge reported a merged PR without merged_at");
    }
    merged_at
}

/// Map a forge state string plus merge info to [`ActivityState`].
pub(crate) fn normalize_state(state: &str, merged_at: Option<DateTime<Utc>>) -> ActivityState {
    if state == "closed" && merged_at.is_some() {
        ActivityState::Merged
    } else if state == "closed" {
        ActivityState::Closed
    } else {
        ActivityState::Open
    }
}

/// Retry delay for a rate-limited response: `Retry-After` first, then (GitHub
/// only) the `x-ratelimit-reset` epoch timestamp.
fn retry_after_for(resp: &reqwest::Response, forge: Forge) -> Option<Duration> {
    let retry = retry_after_secs(resp);
    if forge == Forge::GitHub {
        retry.or_else(|| github_reset_header(resp))
    } else {
        retry
    }
}

fn retry_after_secs(resp: &reqwest::Response) -> Option<Duration> {
    resp.headers()
        .get("Retry-After")
        .and_then(|h| h.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_secs)
}

fn rate_limit_exhausted(resp: &reqwest::Response) -> bool {
    resp.headers()
        .get("x-ratelimit-remaining")
        .and_then(|h| h.to_str().ok())
        .is_some_and(|remaining| remaining == "0")
}

fn github_reset_header(resp: &reqwest::Response) -> Option<Duration> {
    let reset = resp
        .headers()
        .get("x-ratelimit-reset")
        .and_then(|h| h.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    Some(Duration::from_secs(reset.saturating_sub(now)))
}
