use anyhow::{Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
pub struct UploadResponse {
    pub id: String,
    #[serde(default)]
    pub exifInfo: Option<ExifInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ExifInfo {
    #[serde(default)]
    pub exifImageWidth: Option<u32>,
    #[serde(default)]
    pub exifImageHeight: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct UploadResult {
    pub asset_id: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

pub async fn probe(base_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base_url}/server-info/version"))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .context("Failed to connect to Immich")?;

    let info: serde_json::Value = resp
        .json()
        .await
        .context("Failed to parse Immich version response")?;

    let version = info["version"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    Ok(version)
}

pub async fn upload_image(
    base_url: &str,
    api_key: &str,
    image_bytes: &[u8],
    filename: &str,
) -> Result<UploadResult> {
    let client = reqwest::Client::new();
    let now = chrono::Utc::now();

    // SHA-256 for Immich dedup (deviceAssetId)
    let mut hasher = Sha256::new();
    hasher.update(image_bytes);
    let hash = hex::encode(hasher.finalize());
    let device_asset_id = format!("pcomfy-{hash}");

    // Determine content type from extension
    let mime = mime_guess::from_path(filename)
        .first_or_octet_stream()
        .to_string();

    let file_part = reqwest::multipart::Part::bytes(image_bytes.to_vec())
        .file_name(filename.to_string())
        .mime_str(&mime)
        .context("Invalid MIME type")?;

    let form = reqwest::multipart::Form::new()
        .part("assetData", file_part)
        .text("deviceAssetId", device_asset_id)
        .text("deviceId", "pcomfy-cli")
        .text("fileCreatedAt", now.to_rfc3339())
        .text("fileModifiedAt", now.to_rfc3339());

    let resp = client
        .post(format!("{base_url}/assets"))
        .header("x-api-key", api_key)
        .multipart(form)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await
        .context("Failed to upload image to Immich")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Immich upload failed (HTTP {status}): {body}");
    }

    let upload: UploadResponse = resp
        .json()
        .await
        .context("Failed to parse Immich upload response")?;

    let (width, height) = upload
        .exifInfo
        .map(|e| (e.exifImageWidth, e.exifImageHeight))
        .unwrap_or((None, None));

    // If EXIF data is missing, fetch asset info
    if width.is_none() || height.is_none() {
        if let Ok(info) = get_asset_info(base_url, api_key, &upload.id).await {
            return Ok(UploadResult {
                asset_id: upload.id,
                width: info.width,
                height: info.height,
            });
        }
    }

    Ok(UploadResult {
        asset_id: upload.id,
        width,
        height,
    })
}

async fn get_asset_info(
    base_url: &str,
    api_key: &str,
    asset_id: &str,
) -> Result<UploadResult> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base_url}/assets/{asset_id}"))
        .header("x-api-key", api_key)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    let info: serde_json::Value = resp.json().await?;
    let exif = &info["exifInfo"];
    let width = exif["exifImageWidth"].as_u64().map(|v| v as u32);
    let height = exif["exifImageHeight"].as_u64().map(|v| v as u32);

    Ok(UploadResult {
        asset_id: asset_id.to_string(),
        width,
        height,
    })
}

pub fn proxy_url(asset_id: &str, width: Option<u32>, height: Option<u32>) -> String {
    let mut url = format!("/api/images/{asset_id}");
    if let (Some(w), Some(h)) = (width, height) {
        url.push_str(&format!("?w={w}&h={h}"));
    }
    url
}
