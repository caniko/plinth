use anyhow::{Context, Result};
#[cfg(feature = "brick-activity")]
use plinth_shared::{ActivityListItem, PublishActivityRequest};
use serde::Deserialize;

use super::client::ApiClient;
use super::error::ErrorResponse;

/// Response from the publish activity endpoint.
#[cfg(feature = "brick-activity")]
#[derive(Debug, Deserialize)]
pub struct PublishActivityResponse {
    #[allow(dead_code)]
    pub success: bool,
    pub url: String,
    pub id: Option<i64>,
    #[allow(dead_code)]
    pub message: String,
}

#[cfg(feature = "brick-activity")]
#[derive(Debug, Deserialize)]
struct RawPublishActivityResponse {
    success: bool,
    url: Option<String>,
    id: Option<i64>,
    message: String,
}

#[cfg(feature = "brick-activity")]
impl ApiClient {
    /// Publish (upsert) an activity item.
    pub async fn publish_activity(
        &self,
        request: PublishActivityRequest,
    ) -> Result<PublishActivityResponse> {
        let url = format!("{}/api/admin/activity", self.base_url);
        let activity_url = request.url.clone();
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send activity publish request to API")?;
        let status = response.status();
        if status.is_success() {
            let raw: RawPublishActivityResponse = response
                .json()
                .await
                .context("Failed to parse activity publish response")?;
            Ok(PublishActivityResponse {
                success: raw.success,
                url: raw.url.unwrap_or(activity_url),
                id: raw.id,
                message: raw.message,
            })
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                anyhow::bail!(
                    "API error: {} {}",
                    err.error,
                    err.details.unwrap_or_default()
                );
            }
            anyhow::bail!("Activity publish failed (HTTP {status}): {error_text}");
        }
    }

    /// Delete an activity by numeric id.
    pub async fn delete_activity(&self, id: i64) -> Result<()> {
        let url = format!("{}/api/admin/activity/{id}", self.base_url);
        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("Failed to send delete activity request")?;
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Delete activity {id} failed (HTTP {status}): {error_text}");
        }
        Ok(())
    }

    /// Patch impact and/or featured for an activity by numeric id.
    pub async fn patch_activity(
        &self,
        id: i64,
        impact: Option<i16>,
        featured: Option<bool>,
    ) -> Result<()> {
        let url = format!("{}/api/admin/activity/{id}", self.base_url);
        let body = serde_json::json!({ "impact": impact, "featured": featured });
        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send patch activity request")?;
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Update activity {id} failed (HTTP {status}): {error_text}");
        }
        Ok(())
    }

    /// List all activity items (ranked, server-side). Public endpoint.
    pub async fn list_activities(&self) -> Result<Vec<ActivityListItem>> {
        let url = format!("{}/api/activity", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send list activities request")?;
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("List activities failed (HTTP {status}): {error_text}");
        }
        response
            .json()
            .await
            .context("Failed to parse activities list")
    }
}
