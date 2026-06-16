use anyhow::{Context, Result};
use reqwest::Client;

/// API client for communicating with the blog server
pub struct ApiClient {
    pub(crate) client: Client,
    pub(crate) base_url: String,
    pub(crate) api_key: String,
}

impl ApiClient {
    /// Create a new API client
    ///
    /// # Arguments
    /// * `base_url` - The base URL of the API (e.g., "http://localhost:3000")
    /// * `api_key` - The API key for authentication
    pub fn new(base_url: String, api_key: String) -> Result<Self> {
        let client = Client::builder()
            .build()
            .context("Failed to build HTTP client (CA certificates may be unavailable)")?;
        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    /// Create API client from environment variables
    ///
    /// Reads from:
    /// - `PLINTH_API_URL` (default: http://localhost:3000)
    /// - `PLINTH_API_KEY` (required)
    #[allow(dead_code)]
    pub fn from_env() -> Result<Self> {
        let base_url =
            std::env::var("PLINTH_API_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        let api_key = std::env::var("PLINTH_API_KEY")
            .context("PLINTH_API_KEY environment variable not set")?;

        Self::new(base_url, api_key)
    }
}
