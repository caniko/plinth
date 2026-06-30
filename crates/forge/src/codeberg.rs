use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use plinth_shared::{ActivityKind, ActivityState, FetchedActivity, Forge};
use reqwest::header::{ACCEPT, HeaderValue};

use crate::{ActivityRef, ForgeClient, ForgeError, ForgeResult, build_http_client};

/// A forge client that fetches PRs and issues from a Forgejo instance (e.g. Codeberg).
pub struct CodebergClient {
    client: reqwest::Client,
    base_url: String,
    token: Option<String>,
}

impl CodebergClient {
    /// Creates a new `CodebergClient` targeting `https://codeberg.org/api/v1`.
    pub fn new(token: Option<String>) -> ForgeResult<Self> {
        Self::with_base_url("https://codeberg.org/api/v1".into(), token)
    }

    /// Creates a new `CodebergClient` with a custom base URL for a different Forgejo instance.
    pub fn with_base_url(base_url: String, token: Option<String>) -> ForgeResult<Self> {
        let client = build_http_client(&base_url)?;
        let base_url = base_url.trim_end_matches('/');
        let base_url = if base_url.ends_with("/api/v1") {
            base_url.to_string()
        } else {
            format!("{base_url}/api/v1")
        };
        Ok(Self {
            client,
            base_url,
            token,
        })
    }

    fn request(&self, url: String) -> reqwest::RequestBuilder {
        let builder = self
            .client
            .get(url)
            .header(ACCEPT, HeaderValue::from_static("application/json"));
        if let Some(token) = &self.token {
            builder.header("Authorization", format!("token {token}"))
        } else {
            builder
        }
    }

    async fn get_pull(&self, r: &ActivityRef) -> ForgeResult<FjPull> {
        let url = format!(
            "{}/repos/{}/{}/pulls/{}",
            self.base_url, r.owner, r.repo, r.number
        );
        let resp = self
            .request(url)
            .send()
            .await
            .map_err(|e| ForgeError::Network(e.to_string()))?;
        map_status(resp)
            .await?
            .json::<FjPull>()
            .await
            .map_err(|e| ForgeError::Decode(e.to_string()))
    }

    async fn get_issue(&self, r: &ActivityRef) -> ForgeResult<FjIssue> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}",
            self.base_url, r.owner, r.repo, r.number
        );
        let resp = self
            .request(url)
            .send()
            .await
            .map_err(|e| ForgeError::Network(e.to_string()))?;
        map_status(resp)
            .await?
            .json::<FjIssue>()
            .await
            .map_err(|e| ForgeError::Decode(e.to_string()))
    }

    async fn get_repo_stars(&self, r: &ActivityRef) -> ForgeResult<Option<i32>> {
        let url = format!("{}/repos/{}/{}", self.base_url, r.owner, r.repo);
        let resp = self
            .request(url)
            .send()
            .await
            .map_err(|e| ForgeError::Network(e.to_string()))?;
        let repo = map_status(resp)
            .await?
            .json::<FjRepo>()
            .await
            .map_err(|e| ForgeError::Decode(e.to_string()))?;
        Ok(repo.stars_count)
    }
}

#[async_trait]
impl ForgeClient for CodebergClient {
    async fn fetch(&self, r: &ActivityRef) -> ForgeResult<FetchedActivity> {
        match r.kind {
            ActivityKind::PullRequest => {
                let pull = self.get_pull(r).await?;
                let repo_stars = self.get_repo_stars(r).await?;
                Ok(normalize_pull(r, pull, repo_stars))
            }
            ActivityKind::Issue => {
                let issue = self.get_issue(r).await?;
                let repo_stars = self.get_repo_stars(r).await?;
                Ok(normalize_issue(r, issue, repo_stars))
            }
        }
    }
}

async fn map_status(resp: reqwest::Response) -> Result<reqwest::Response, ForgeError> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let code = status.as_u16();
    let url = resp.url().to_string();
    Err(match code {
        404 | 410 => ForgeError::NotFound {
            forge: Forge::Codeberg,
            url,
            status: code,
        },
        429 => ForgeError::RateLimited {
            forge: Forge::Codeberg,
            retry_after: retry_after_from(&resp),
        },
        _ => {
            let body = resp.text().await.unwrap_or_default();
            ForgeError::Http {
                forge: Forge::Codeberg,
                status: code,
                body,
            }
        }
    })
}

fn retry_after_from(resp: &reqwest::Response) -> Option<Duration> {
    resp.headers()
        .get("Retry-After")
        .and_then(|h| h.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_secs)
}

fn normalize_pull(r: &ActivityRef, pull: FjPull, repo_stars: Option<i32>) -> FetchedActivity {
    let merged_at = merge_timestamp(pull.merged, pull.merged_at);
    FetchedActivity {
        forge: Forge::Codeberg,
        repo_owner: r.owner.clone(),
        repo_name: r.repo.clone(),
        kind: ActivityKind::PullRequest,
        number: r.number,
        url: pull.html_url.unwrap_or_else(|| {
            format!(
                "https://codeberg.org/{}/{}/pulls/{}",
                r.owner, r.repo, r.number
            )
        }),
        title: pull.title,
        body: pull.body,
        state: normalize_state(&pull.state, merged_at),
        created_at: pull.created_at,
        closed_at: pull.closed_at,
        merged_at,
        additions: pull.additions,
        deletions: pull.deletions,
        comments_count: pull.comments,
        labels: pull.labels.into_iter().map(|label| label.name).collect(),
        repo_stars,
    }
}

fn normalize_issue(r: &ActivityRef, issue: FjIssue, repo_stars: Option<i32>) -> FetchedActivity {
    let merged_at = issue
        .pull_request
        .and_then(|meta| merge_timestamp(meta.merged, meta.merged_at));
    FetchedActivity {
        forge: Forge::Codeberg,
        repo_owner: r.owner.clone(),
        repo_name: r.repo.clone(),
        kind: ActivityKind::Issue,
        number: r.number,
        url: issue.html_url.unwrap_or_else(|| {
            format!(
                "https://codeberg.org/{}/{}/issues/{}",
                r.owner, r.repo, r.number
            )
        }),
        title: issue.title,
        body: issue.body,
        state: normalize_state(&issue.state, merged_at),
        created_at: issue.created_at,
        closed_at: issue.closed_at,
        merged_at,
        additions: None,
        deletions: None,
        comments_count: issue.comments,
        labels: issue.labels.into_iter().map(|label| label.name).collect(),
        repo_stars,
    }
}

fn merge_timestamp(
    merged: Option<bool>,
    merged_at: Option<DateTime<Utc>>,
) -> Option<DateTime<Utc>> {
    if merged == Some(true) && merged_at.is_none() {
        tracing::debug!("Forgejo reported a merged PR without merged_at");
    }
    merged_at
}

fn normalize_state(state: &str, merged_at: Option<DateTime<Utc>>) -> ActivityState {
    if state == "closed" && merged_at.is_some() {
        ActivityState::Merged
    } else if state == "closed" {
        ActivityState::Closed
    } else {
        ActivityState::Open
    }
}

#[derive(serde::Deserialize)]
struct FjPull {
    title: String,
    body: Option<String>,
    state: String,
    merged: Option<bool>,
    merged_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    closed_at: Option<DateTime<Utc>>,
    additions: Option<i32>,
    deletions: Option<i32>,
    comments: Option<i32>,
    labels: Vec<FjLabel>,
    html_url: Option<String>,
}

#[derive(serde::Deserialize)]
struct FjIssue {
    title: String,
    body: Option<String>,
    state: String,
    created_at: DateTime<Utc>,
    closed_at: Option<DateTime<Utc>>,
    comments: Option<i32>,
    labels: Vec<FjLabel>,
    pull_request: Option<FjPrMeta>,
    html_url: Option<String>,
}

#[derive(serde::Deserialize)]
struct FjPrMeta {
    merged: Option<bool>,
    merged_at: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize)]
struct FjLabel {
    name: String,
}

#[derive(serde::Deserialize)]
struct FjRepo {
    stars_count: Option<i32>,
}
