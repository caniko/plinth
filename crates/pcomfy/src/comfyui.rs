use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ComfyUIInfo {
    pub version: String,
    pub device: String,
    pub model_count: usize,
}

#[derive(Debug, Clone)]
pub struct WorkflowInfo {
    pub name: String,
    pub node_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResponse {
    pub prompt_id: String,
    pub number: u64,
    pub node_errors: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub outputs: HashMap<String, HistoryOutput>,
    pub status: HistoryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryOutput {
    pub images: Option<Vec<GeneratedImage>>,
    #[serde(rename = "gifs")]
    pub gifs: Option<Vec<GeneratedImage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedImage {
    pub filename: String,
    pub subfolder: String,
    #[serde(rename = "type")]
    pub image_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStatus {
    pub completed: bool,
}

pub async fn probe(base_url: &str) -> Result<ComfyUIInfo> {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base_url}/object_info"))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .context("Failed to connect to ComfyUI")?;

    let obj_info: HashMap<String, serde_json::Value> = resp
        .json()
        .await
        .context("Failed to parse ComfyUI /object_info response")?;

    let model_count = obj_info.len();
    let version = "unknown".to_string();
    let device = "unknown".to_string();

    Ok(ComfyUIInfo {
        version,
        device,
        model_count,
    })
}

pub async fn submit_prompt(
    base_url: &str,
    workflow_json: serde_json::Value,
) -> Result<PromptResponse> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({ "prompt": workflow_json });

    let resp = client
        .post(format!("{base_url}/prompt"))
        .json(&body)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .context("Failed to submit prompt to ComfyUI")?;

    let pr: PromptResponse = resp
        .json()
        .await
        .context("Failed to parse ComfyUI prompt response")?;

    if let Some(errors) = &pr.node_errors
        && !errors.is_empty()
    {
        anyhow::bail!("ComfyUI node errors: {errors:?}");
    }

    Ok(pr)
}

pub async fn poll_history(
    base_url: &str,
    prompt_id: &str,
    poll_interval: Duration,
) -> Result<HistoryEntry> {
    let client = reqwest::Client::new();

    loop {
        tokio::time::sleep(poll_interval).await;

        let resp = client
            .get(format!("{base_url}/history/{prompt_id}"))
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .context("Failed to poll ComfyUI history")?;

        let history: HashMap<String, HistoryEntry> = resp
            .json()
            .await
            .context("Failed to parse ComfyUI history response")?;

        if let Some(entry) = history.get(prompt_id)
            && entry.status.completed
        {
            return Ok(entry.clone());
        }
    }
}

pub async fn download_image(
    base_url: &str,
    filename: &str,
    subfolder: &str,
    image_type: &str,
) -> Result<Vec<u8>> {
    let client = reqwest::Client::new();
    let url =
        format!("{base_url}/view?filename={filename}&subfolder={subfolder}&type={image_type}");
    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .context("Failed to download image from ComfyUI")?;

    let bytes = resp.bytes().await.context("Failed to read image bytes")?;

    Ok(bytes.to_vec())
}

pub async fn list_workflows(base_url: &str) -> Result<Vec<WorkflowInfo>> {
    probe(base_url).await?;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{base_url}/object_info"))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .context("Failed to query ComfyUI object_info")?;

    let obj_info: HashMap<String, serde_json::Value> =
        resp.json().await.context("Failed to parse object_info")?;

    let workflows = obj_info
        .into_keys()
        .map(|name| WorkflowInfo {
            name,
            node_count: 0,
        })
        .collect();

    Ok(workflows)
}
