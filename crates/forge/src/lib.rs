//! Fetch a single PR or issue from GitHub or Forgejo and normalize it into
//! `plinth_shared::FetchedActivity`.
//!
//! This crate is reqwest-based and must stay out of the WASM client dependency graph.

mod codeberg;
mod error;
mod github;
mod router;

pub use codeberg::CodebergClient;
pub use error::{ForgeError, ForgeResult};
pub use github::GitHubClient;
pub use router::ForgeRouter;

use async_trait::async_trait;
use plinth_shared::{ActivityKind, FetchedActivity, Forge};

/// Identifies one pull request or issue on one repository.
#[derive(Debug, Clone)]
pub struct ActivityRef {
    pub forge: Forge,
    pub owner: String,
    pub repo: String,
    pub kind: ActivityKind,
    pub number: i32,
}

#[async_trait]
pub trait ForgeClient: Send + Sync {
    /// Fetch and normalize a single pull request or issue.
    async fn fetch(&self, r: &ActivityRef) -> ForgeResult<FetchedActivity>;
}

pub(crate) fn build_http_client(base_url: &str) -> reqwest::Client {
    let mut builder = reqwest::Client::builder().user_agent("plinth-forge");
    if base_url.starts_with("http://") {
        builder = builder.tls_certs_only(std::iter::empty::<reqwest::Certificate>());
    }
    builder.build().expect("failed to build forge HTTP client")
}
