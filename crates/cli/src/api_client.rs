use anyhow::{Context, Result};
use plinth_shared::{
    AddTagRequest, CreateTodoRequest, PublishArticleRequest, SiteContent, Tag, TodoListItem,
    UpdateSiteContentRequest, UpdateTodoRequest,
};
use reqwest::Client;
use serde::Deserialize;

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

    /// List all tags
    pub async fn list_tags(&self) -> Result<Vec<Tag>> {
        let url = format!("{}/api/admin/tags", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("Failed to send list tags request")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("List tags failed: {}", error_text);
        }

        let tags = response.json().await.context("Failed to parse tags list")?;

        Ok(tags)
    }

    /// Add a tag to a post
    pub async fn add_tag_to_post(&self, post_slug: &str, tag: &str) -> Result<()> {
        let url = format!("{}/api/admin/posts/{}/tags", self.base_url, post_slug);
        let request = AddTagRequest {
            tag: tag.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send add tag request")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Add tag failed: {}", error_text);
        }

        Ok(())
    }

    /// Remove a tag from a post
    pub async fn remove_tag_from_post(&self, post_slug: &str, tag_slug: &str) -> Result<()> {
        let url = format!(
            "{}/api/admin/posts/{}/tags/{}",
            self.base_url, post_slug, tag_slug
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("Failed to send remove tag request")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Remove tag failed: {}", error_text);
        }

        Ok(())
    }

    /// Update site content by key
    pub async fn update_site_content(
        &self,
        key: &str,
        request: UpdateSiteContentRequest,
    ) -> Result<()> {
        let url = format!("{}/api/admin/content/{}", self.base_url, key);

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send update content request")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Update content failed: {}", error_text);
        }

        Ok(())
    }

    /// Get site content by key
    pub async fn get_site_content(&self, key: &str) -> Result<Option<SiteContent>> {
        let url = format!("{}/api/admin/content/{}", self.base_url, key);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("Failed to send get content request")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Get content failed: {}", error_text);
        }

        let content = response
            .json()
            .await
            .context("Failed to parse content response")?;
        Ok(content)
    }

    /// Create a new TODO item
    pub async fn create_todo(&self, request: CreateTodoRequest) -> Result<()> {
        let url = format!("{}/api/admin/todos", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send create TODO request")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Create TODO failed: {}", error_text);
        }

        Ok(())
    }

    /// Update an existing TODO item
    pub async fn update_todo(&self, slug: &str, request: UpdateTodoRequest) -> Result<()> {
        let url = format!("{}/api/admin/todos/{}", self.base_url, slug);

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send update TODO request")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Update TODO failed: {}", error_text);
        }

        Ok(())
    }

    /// Delete a TODO item
    pub async fn delete_todo(&self, slug: &str) -> Result<()> {
        let url = format!("{}/api/admin/todos/{}", self.base_url, slug);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("Failed to send delete TODO request")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Delete TODO failed: {}", error_text);
        }

        Ok(())
    }

    /// List all TODO items
    pub async fn list_todos(&self) -> Result<Vec<TodoListItem>> {
        let url = format!("{}/api/admin/todos", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("Failed to send list TODOs request")?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("List TODOs failed: {}", error_text);
        }

        let items = response.json().await.context("Failed to parse TODO list")?;
        Ok(items)
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
        // Use builder().build() instead of Client::new() to avoid panic
        // when system CA certs are unavailable (e.g. Nix sandbox)
        let http_client = match Client::builder().build() {
            Ok(c) => c,
            Err(_) => return, // Skip test if HTTP client can't be built
        };
        let client = ApiClient {
            client: http_client,
            base_url: "http://localhost:3000".to_string(),
            api_key: "test_key".to_string(),
        };

        assert_eq!(client.base_url, "http://localhost:3000");
        assert_eq!(client.api_key, "test_key");
    }
}
