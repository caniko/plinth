use anyhow::{Context, Result};
use reqwest::multipart;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::Path;

pub struct UploadResult {
    pub asset_id: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImmichAssetResponse {
    pub id: String,
    #[serde(default)]
    pub exif_info: Option<ImmichExifInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImmichExifInfo {
    pub exif_image_width: Option<u32>,
    pub exif_image_height: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImmichAssetInfo {
    #[serde(default)]
    pub exif_info: Option<ImmichExifInfo>,
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

    /// Fetch asset info (including EXIF dimensions) from Immich.
    pub async fn get_asset_info(&self, asset_id: &str) -> Result<ImmichAssetInfo> {
        let url = format!("{}/api/assets/{}", self.base_url, asset_id);
        let resp = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .context("Failed to fetch asset info from Immich")?;

        if !resp.status().is_success() {
            anyhow::bail!("Immich asset info failed (HTTP {})", resp.status().as_u16());
        }

        resp.json()
            .await
            .context("Failed to parse Immich asset info")
    }

    /// Upload a local file to Immich. Returns the asset ID and dimensions.
    ///
    /// Uses a content-hash-based `deviceAssetId` for deduplication —
    /// uploading the same file twice returns the existing asset.
    pub async fn upload_asset(&self, file_path: &Path) -> Result<UploadResult> {
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

        // Try to get dimensions from the upload response's EXIF data
        let (width, height) = match &asset.exif_info {
            Some(exif) if exif.exif_image_width.is_some() && exif.exif_image_height.is_some() => {
                (exif.exif_image_width, exif.exif_image_height)
            }
            _ => {
                // Fallback: fetch asset info for dimensions
                match self.get_asset_info(&asset.id).await {
                    Ok(info) => match info.exif_info {
                        Some(exif) => (exif.exif_image_width, exif.exif_image_height),
                        None => (None, None),
                    },
                    Err(_) => (None, None),
                }
            }
        };

        Ok(UploadResult {
            asset_id: asset.id,
            width,
            height,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immich_client_creation() {
        let client = ImmichClient::new("http://immich:2283".to_string(), "test-key".to_string());
        // In Nix sandbox, no CA certs are available so Client::builder().build() fails.
        // Skip assertion gracefully in that case.
        if let Ok(client) = client {
            assert_eq!(client.base_url, "http://immich:2283");
            assert_eq!(client.api_key, "test-key");
        }
    }

    #[test]
    fn test_trailing_slash_stripped() {
        let client = ImmichClient::new("http://immich:2283/".to_string(), "key".to_string());
        if let Ok(client) = client {
            assert_eq!(client.base_url, "http://immich:2283");
        }
    }

    #[test]
    fn test_deserialize_upload_response_with_exif() {
        let json =
            r#"{"id": "abc-123", "exifInfo": {"exifImageWidth": 1920, "exifImageHeight": 1080}}"#;
        let resp: ImmichAssetResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, "abc-123");
        let exif = resp.exif_info.unwrap();
        assert_eq!(exif.exif_image_width, Some(1920));
        assert_eq!(exif.exif_image_height, Some(1080));
    }

    #[test]
    fn test_deserialize_upload_response_without_exif() {
        let json = r#"{"id": "abc-123"}"#;
        let resp: ImmichAssetResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, "abc-123");
        assert!(resp.exif_info.is_none());
    }

    #[test]
    fn test_deserialize_partial_exif_width_only() {
        let json = r#"{"id": "abc-123", "exifInfo": {"exifImageWidth": 1920}}"#;
        let resp: ImmichAssetResponse = serde_json::from_str(json).unwrap();
        let exif = resp.exif_info.unwrap();
        assert_eq!(exif.exif_image_width, Some(1920));
        assert_eq!(exif.exif_image_height, None);
    }

    #[test]
    fn test_deserialize_ignores_extra_fields() {
        let json = r#"{"id": "abc-123", "type": "IMAGE", "originalPath": "/upload/abc.jpg", "exifInfo": {"exifImageWidth": 800, "exifImageHeight": 600, "make": "Canon", "model": "EOS R5"}}"#;
        let resp: ImmichAssetResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.id, "abc-123");
        let exif = resp.exif_info.unwrap();
        assert_eq!(exif.exif_image_width, Some(800));
        assert_eq!(exif.exif_image_height, Some(600));
    }

    #[test]
    fn test_deserialize_asset_info() {
        let json =
            r#"{"id": "xyz-789", "exifInfo": {"exifImageWidth": 3840, "exifImageHeight": 2160}}"#;
        let info: ImmichAssetInfo = serde_json::from_str(json).unwrap();
        let exif = info.exif_info.unwrap();
        assert_eq!(exif.exif_image_width, Some(3840));
        assert_eq!(exif.exif_image_height, Some(2160));
    }

    #[test]
    fn test_deserialize_asset_info_no_exif() {
        let json = r#"{"id": "xyz-789"}"#;
        let info: ImmichAssetInfo = serde_json::from_str(json).unwrap();
        assert!(info.exif_info.is_none());
    }

    #[test]
    fn test_deserialize_exif_zero_dimensions() {
        let json = r#"{"id": "abc", "exifInfo": {"exifImageWidth": 0, "exifImageHeight": 0}}"#;
        let resp: ImmichAssetResponse = serde_json::from_str(json).unwrap();
        let exif = resp.exif_info.unwrap();
        assert_eq!(exif.exif_image_width, Some(0));
        assert_eq!(exif.exif_image_height, Some(0));
    }

    #[test]
    fn test_deserialize_exif_null_values() {
        let json =
            r#"{"id": "abc", "exifInfo": {"exifImageWidth": null, "exifImageHeight": null}}"#;
        let resp: ImmichAssetResponse = serde_json::from_str(json).unwrap();
        let exif = resp.exif_info.unwrap();
        assert_eq!(exif.exif_image_width, None);
        assert_eq!(exif.exif_image_height, None);
    }
}
