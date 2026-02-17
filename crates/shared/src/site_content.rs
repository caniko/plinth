use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::serde_helpers::deserialize_flexible_id;

/// A piece of site content identified by a unique key (e.g. "home-intro", "about").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteContent {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_flexible_id"
    )]
    pub id: Option<String>,

    /// Unique key identifying this content
    pub key: String,

    /// Optional title
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Raw Typst source content
    pub content: String,

    /// Pre-rendered HTML
    pub html_content: String,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Request payload for updating site content via the admin API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSiteContentRequest {
    /// Raw Typst source content
    pub content: String,

    /// Optional title
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Pre-rendered HTML (compiled from Typst by CLI)
    pub html_content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_site_content_serialization_roundtrip() {
        let content = SiteContent {
            id: None,
            key: "home-intro".to_string(),
            title: Some("Welcome".to_string()),
            content: "Hello world".to_string(),
            html_content: "<p>Hello world</p>".to_string(),
            updated_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&content).unwrap();
        let deserialized: SiteContent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.key, "home-intro");
        assert_eq!(deserialized.title.as_deref(), Some("Welcome"));
    }

    #[test]
    fn test_update_request_skip_none_fields() {
        let req = UpdateSiteContentRequest {
            content: "body".to_string(),
            title: None,
            html_content: "<p>body</p>".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("title"));
        assert!(json.contains("content"));
        assert!(json.contains("html_content"));
    }
}
