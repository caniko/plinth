use anyhow::{Context, Result};
use plinth_shared::{SiteContent, UpdateSiteContentRequest};

use super::client::ApiClient;

impl ApiClient {
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

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Update content '{key}' failed (HTTP {status}): {error_text}");
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

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Get content '{key}' failed (HTTP {status}): {error_text}");
        }

        let content = response
            .json()
            .await
            .context("Failed to parse content response")?;
        Ok(content)
    }
}
