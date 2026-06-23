use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_comfyui_url")]
    pub comfyui_url: String,

    #[serde(default = "default_immich_url")]
    pub immich_url: String,

    #[serde(default)]
    pub immich_api_key: String,

    #[serde(default = "default_articles_dir")]
    pub articles_dir: PathBuf,

    #[serde(default)]
    pub workflows: WorkflowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    #[serde(default = "default_cluster_workflow")]
    pub nix_infrastructure: String,

    #[serde(default = "default_cluster_workflow")]
    pub rust_tools: String,

    #[serde(default = "default_cluster_workflow")]
    pub publishing: String,

    #[serde(default = "default_cluster_workflow")]
    pub ai_neural: String,

    #[serde(default = "default_cluster_workflow")]
    pub gaming: String,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            nix_infrastructure: default_cluster_workflow(),
            rust_tools: default_cluster_workflow(),
            publishing: default_cluster_workflow(),
            ai_neural: default_cluster_workflow(),
            gaming: default_cluster_workflow(),
        }
    }
}

fn default_comfyui_url() -> String {
    "http://localhost:8188".into()
}

fn default_immich_url() -> String {
    "https://immich.candee.baby/api".into()
}

fn default_articles_dir() -> PathBuf {
    // relative to the project root — caller can override with env
    PathBuf::from("website/personal/can/posts")
}

fn default_cluster_workflow() -> String {
    "flux_schnell".into()
}

impl Config {
    pub fn load(
        comfyui_url: Option<String>,
        immich_url: Option<String>,
        immich_api_key: Option<String>,
        articles_dir: Option<PathBuf>,
    ) -> Result<Self> {
        let mut cfg = Self::from_file()?.unwrap_or_default();

        if let Some(v) = comfyui_url {
            cfg.comfyui_url = v;
        }
        if let Some(v) = immich_url {
            cfg.immich_url = v;
        }
        if let Some(v) = immich_api_key {
            cfg.immich_api_key = v;
        }
        if let Some(v) = articles_dir {
            cfg.articles_dir = v;
        }

        Ok(cfg)
    }

    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
        PathBuf::from(home).join(".config/pcomfy/config.toml")
    }

    fn from_file() -> Result<Option<Self>> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(None);
        }
        let contents = std::fs::read_to_string(&path)?;
        let cfg: Config = toml::from_str(&contents)?;
        Ok(Some(cfg))
    }

    pub fn create_default() -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let cfg = Config::default();
        let contents = toml::to_string_pretty(&cfg)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            comfyui_url: default_comfyui_url(),
            immich_url: default_immich_url(),
            immich_api_key: String::new(),
            articles_dir: default_articles_dir(),
            workflows: WorkflowConfig::default(),
        }
    }
}
