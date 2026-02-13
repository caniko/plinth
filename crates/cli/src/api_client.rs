use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use shared::PublishArticleRequest;

/// Response from the publish article endpoint
#[derive(Debug, Deserialize)]
pub struct PublishArticleResponse {
    #[allow(dead_code)]
    pub success: bool,
    pub slug: String,
    pub id: Option<String>,
    pub message: String,
}

/// Error response from the API
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

/// API client for communicating with the blog server
pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl ApiClient {
    /// Create a new API client
    ///
    /// # Arguments
    /// * `base_url` - The base URL of the API (e.g., "http://localhost:3000")
    /// * `api_key` - The API key for authentication
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_key,
        }
    }

    /// Create API client from environment variables
    ///
    /// Reads from:
    /// - `BLOG_API_URL` (default: http://localhost:3000)
    /// - `BLOG_API_KEY` (required)
    #[allow(dead_code)]
    pub fn from_env() -> Result<Self> {
        let base_url =
            std::env::var("BLOG_API_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        let api_key =
            std::env::var("BLOG_API_KEY").context("BLOG_API_KEY environment variable not set")?;

        Ok(Self::new(base_url, api_key))
    }

    /// Publish a new article
    pub async fn publish_article(
        &self,
        request: PublishArticleRequest,
    ) -> Result<PublishArticleResponse> {
        let url = format!("{}/api/admin/articles", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to API")?;

        let status = response.status();

        if status.is_success() {
            let publish_response: PublishArticleResponse = response
                .json()
                .await
                .context("Failed to parse success response")?;

            Ok(publish_response)
        } else {
            // Try to parse error response
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Try to parse as ErrorResponse
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                anyhow::bail!(
                    "API error: {} {}",
                    error_response.error,
                    error_response.details.unwrap_or_default()
                );
            }

            anyhow::bail!("API request failed with status {}: {}", status, error_text);
        }
    }

    /// Delete an article by slug (future implementation)
    pub async fn delete_article(&self, slug: &str) -> Result<()> {
        let url = format!("{}/api/admin/articles/{}", self.base_url, slug);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("Failed to send delete request to API")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Delete failed: {}", error_text);
        }

        Ok(())
    }

    /// List all articles (future implementation)
    pub async fn list_articles(&self) -> Result<Vec<serde_json::Value>> {
        let url = format!("{}/api/admin/articles", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("Failed to send list request to API")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("List failed: {}", error_text);
        }

        let articles = response
            .json()
            .await
            .context("Failed to parse articles list")?;

        Ok(articles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_creation() {
        let client = ApiClient::new("http://localhost:3000".to_string(), "test_key".to_string());

        assert_eq!(client.base_url, "http://localhost:3000");
        assert_eq!(client.api_key, "test_key");
    }
}
