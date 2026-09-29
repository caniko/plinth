use async_trait::async_trait;
use chrono::{DateTime, Utc};
use plinth_shared::{ActivityKind, FetchedActivity, Forge};
use reqwest::header::{ACCEPT, HeaderValue};

use crate::{
    ActivityRef, ForgeClient, ForgeResult, build_http_client, fetch_json, merge_timestamp,
    normalize_state,
};

/// A forge client that fetches PRs and issues from the GitHub REST API.
pub struct GitHubClient {
    client: reqwest::Client,
    base_url: String,
    token: Option<String>,
}

impl GitHubClient {
    /// Creates a new `GitHubClient` targeting the public GitHub API.
    pub fn new(token: Option<String>) -> ForgeResult<Self> {
        Self::with_base_url("https://api.github.com".into(), token)
    }

    /// Creates a new `GitHubClient` with a custom base URL (e.g. for GitHub Enterprise).
    pub fn with_base_url(base_url: String, token: Option<String>) -> ForgeResult<Self> {
        let client = build_http_client(&base_url)?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
        })
    }

    fn request(&self, url: String) -> reqwest::RequestBuilder {
        let builder = self
            .client
            .get(url)
            .header(
                ACCEPT,
                HeaderValue::from_static("application/vnd.github+json"),
            )
            .header("X-GitHub-Api-Version", "2022-11-28");
        if let Some(token) = &self.token {
            builder.bearer_auth(token)
        } else {
            builder
        }
    }

    async fn get_pull(&self, r: &ActivityRef) -> ForgeResult<GhPull> {
        let url = format!(
            "{}/repos/{}/{}/pulls/{}",
            self.base_url, r.owner, r.repo, r.number
        );
        fetch_json(self.request(url), Forge::GitHub).await
    }

    async fn get_issue(&self, r: &ActivityRef) -> ForgeResult<GhIssue> {
        let url = format!(
            "{}/repos/{}/{}/issues/{}",
            self.base_url, r.owner, r.repo, r.number
        );
        fetch_json(self.request(url), Forge::GitHub).await
    }

    async fn get_repo_stars(&self, r: &ActivityRef) -> ForgeResult<Option<i32>> {
        let url = format!("{}/repos/{}/{}", self.base_url, r.owner, r.repo);
        let repo: GhRepo = fetch_json(self.request(url), Forge::GitHub).await?;
        Ok(repo.stargazers_count)
    }
}

#[async_trait]
impl ForgeClient for GitHubClient {
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

fn normalize_pull(r: &ActivityRef, pull: GhPull, repo_stars: Option<i32>) -> FetchedActivity {
    let merged_at = merge_timestamp(Forge::GitHub, pull.merged, pull.merged_at);
    FetchedActivity {
        forge: Forge::GitHub,
        repo_owner: r.owner.clone(),
        repo_name: r.repo.clone(),
        kind: ActivityKind::PullRequest,
        number: r.number,
        url: pull.html_url.unwrap_or_else(|| {
            format!(
                "https://github.com/{}/{}/pull/{}",
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

fn normalize_issue(r: &ActivityRef, issue: GhIssue, repo_stars: Option<i32>) -> FetchedActivity {
    let merged_at = issue.pull_request.and_then(|meta| meta.merged_at);
    FetchedActivity {
        forge: Forge::GitHub,
        repo_owner: r.owner.clone(),
        repo_name: r.repo.clone(),
        kind: ActivityKind::Issue,
        number: r.number,
        url: issue.html_url.unwrap_or_else(|| {
            format!(
                "https://github.com/{}/{}/issues/{}",
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

#[derive(serde::Deserialize)]
struct GhPull {
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
    labels: Vec<GhLabel>,
    html_url: Option<String>,
}

#[derive(serde::Deserialize)]
struct GhIssue {
    title: String,
    body: Option<String>,
    state: String,
    created_at: DateTime<Utc>,
    closed_at: Option<DateTime<Utc>>,
    comments: Option<i32>,
    labels: Vec<GhLabel>,
    pull_request: Option<GhIssuePrMeta>,
    html_url: Option<String>,
}

#[derive(serde::Deserialize)]
struct GhIssuePrMeta {
    merged_at: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize)]
struct GhLabel {
    name: String,
}

#[derive(serde::Deserialize)]
struct GhRepo {
    stargazers_count: Option<i32>,
}
