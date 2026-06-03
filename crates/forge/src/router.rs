use async_trait::async_trait;
use plinth_shared::{FetchedActivity, Forge};

use crate::{ActivityRef, CodebergClient, ForgeClient, ForgeResult, GitHubClient};

/// Dispatches forge fetches to the backend-specific client selected by `ActivityRef::forge`.
pub struct ForgeRouter {
    pub github: GitHubClient,
    pub codeberg: CodebergClient,
}

impl ForgeRouter {
    /// Build a router from optional tokens, using the default forge base URLs.
    pub fn new(github_token: Option<String>, codeberg_token: Option<String>) -> Self {
        Self {
            github: GitHubClient::new(github_token),
            codeberg: CodebergClient::new(codeberg_token),
        }
    }
}

#[async_trait]
impl ForgeClient for ForgeRouter {
    async fn fetch(&self, r: &ActivityRef) -> ForgeResult<FetchedActivity> {
        match r.forge {
            Forge::GitHub => self.github.fetch(r).await,
            Forge::Codeberg => self.codeberg.fetch(r).await,
        }
    }
}
