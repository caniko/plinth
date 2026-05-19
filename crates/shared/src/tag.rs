use serde::{Deserialize, Serialize};

use crate::serde_helpers::deserialize_flexible_id;

/// A tag for categorizing posts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Database record ID
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_flexible_id"
    )]
    pub id: Option<String>,

    /// Tag display name
    pub name: String,

    /// URL-friendly slug
    pub slug: String,

    /// Number of blog posts with this tag (computed, not stored)
    #[cfg(feature = "brick-blog")]
    #[serde(default)]
    pub post_count: u32,

    /// Number of TODO items with this tag (computed, not stored)
    #[cfg(feature = "brick-todo")]
    #[serde(default)]
    pub todo_count: u32,
}

/// Request payload for adding a tag to a post
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTagRequest {
    pub tag: String,
}
