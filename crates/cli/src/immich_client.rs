use anyhow::{Context, Result};
use reqwest::multipart;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct ImmichAssetResponse {
    pub id: String,
}

pub struct ImmichClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl ImmichClient {
    pub fn new(base_url: String, api_key: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .build()
            .context("Failed to build HTTP client for Immich")?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
        })
    }

    /// Upload a local file to Immich. Returns the asset ID.
    ///
    /// Uses a content-hash-based `deviceAssetId` for deduplication —
    /// uploading the same file twice returns the existing asset.
    pub async fn upload_asset(&self, file_path: &Path) -> Result<String> {
        let url = format!("{}/api/assets", self.base_url);

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("upload")
            .to_string();

        let file_bytes = std::fs::read(file_path)
            .with_context(|| format!("Failed to read image file: {}", file_path.display()))?;

        // Content-hash for deduplication
        let device_asset_id = {
            let mut hasher = Sha256::new();
            hasher.update(&file_bytes);
            format!("plinth-{:x}", hasher.finalize())
        };

        let mime = mime_guess::from_path(file_path)
            .first_or_octet_stream()
            .to_string();

        let now = chrono::Utc::now().to_rfc3339();

        let form = multipart::Form::new()
            .text("deviceAssetId", device_asset_id)
            .text("deviceId", "plinth-cli")
            .text("fileCreatedAt", now.clone())
            .text("fileModifiedAt", now)
            .part(
                "assetData",
                multipart::Part::bytes(file_bytes)
                    .file_name(file_name.clone())
                    .mime_str(&mime)
                    .context("Invalid MIME type")?,
            );

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .multipart(form)
            .send()
            .await
            .with_context(|| format!("Failed to upload {} to Immich", file_name))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Immich upload failed (HTTP {}): {}", status.as_u16(), body);
        }

        let asset: ImmichAssetResponse = response
            .json()
            .await
            .context("Failed to parse Immich upload response")?;

        Ok(asset.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immich_client_creation() {
        let client = ImmichClient::new("http://immich:2283".to_string(), "test-key".to_string());
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.base_url, "http://immich:2283");
        assert_eq!(client.api_key, "test-key");
    }

    #[test]
    fn test_trailing_slash_stripped() {
        let client =
            ImmichClient::new("http://immich:2283/".to_string(), "key".to_string()).unwrap();
        assert_eq!(client.base_url, "http://immich:2283");
    }
}
