use anyhow::{Context, Result};
#[cfg(feature = "brick-portfolio")]
use plinth_shared::PublishPortfolioRequest;
use serde::Deserialize;

use super::client::ApiClient;
use super::error::ErrorResponse;

/// Response from the publish portfolio endpoint
#[cfg(feature = "brick-portfolio")]
#[derive(Debug, Deserialize)]
pub struct PublishPortfolioResponse {
    #[allow(dead_code)]
    pub success: bool,
    pub slug: String,
    pub id: Option<String>,
    pub message: String,
}

#[cfg(feature = "brick-portfolio")]
impl ApiClient {
    /// Publish or update a portfolio item
    pub async fn publish_portfolio(
        &self,
        request: PublishPortfolioRequest,
    ) -> Result<PublishPortfolioResponse> {
        let url = format!("{}/api/admin/portfolio", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send portfolio publish request to API")?;

        let status = response.status();

        if status.is_success() {
            let publish_response: PublishPortfolioResponse = response
                .json()
                .await
                .context("Failed to parse portfolio publish response")?;

            Ok(publish_response)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&error_text) {
                anyhow::bail!(
                    "API error: {} {}",
                    error_response.error,
                    error_response.details.unwrap_or_default()
                );
            }

            anyhow::bail!(
                "Portfolio publish failed with status {}: {}",
                status,
                error_text
            );
        }
    }
}
