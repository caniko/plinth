use std::time::Duration;

use plinth_shared::Forge;

/// Errors returned by forge clients.
#[derive(Debug, thiserror::Error)]
pub enum ForgeError {
    /// 404 or 410: the PR, issue, or repository no longer exists upstream.
    #[error("forge resource not found ({forge}: {url}, http {status})", forge = forge.as_str())]
    NotFound {
        forge: Forge,
        url: String,
        status: u16,
    },

    /// 429, or 403 with an exhausted quota.
    #[error("forge rate limited ({forge}; retry after {retry_after:?})", forge = forge.as_str())]
    RateLimited {
        forge: Forge,
        retry_after: Option<Duration>,
    },

    /// Any other non-success HTTP status.
    #[error("forge http error ({forge}: http {status})", forge = forge.as_str())]
    Http {
        forge: Forge,
        status: u16,
        body: String,
    },

    /// Transport failure.
    #[error("forge network error: {0}")]
    Network(String),

    /// Body decode or JSON shape mismatch.
    #[error("forge decode error: {0}")]
    Decode(String),
}

/// Alias for `Result<T, ForgeError>` returned by all forge client operations.
pub type ForgeResult<T> = Result<T, ForgeError>;
