use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::serde_helpers::deserialize_flexible_id;

/// Full TODO/bucket-list item with all fields (used on detail page)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TodoItem {
    /// Database record ID
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_flexible_id"
    )]
    pub id: Option<String>,

    /// URL-friendly slug
    pub slug: String,

    /// Item title
    pub title: String,

    /// Short description
    pub description: String,

    /// Optional long-form Typst source content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Optional rendered HTML content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_content: Option<String>,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Whether this item is completed
    #[serde(default)]
    pub completed: bool,

    /// When the item was marked completed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,

    /// When the item was created
    pub created_at: DateTime<Utc>,

    /// Display order (lower values first)
    #[serde(default)]
    pub order: i32,
}

/// Lightweight version for list pages (excludes content/html_content)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TodoListItem {
    /// Database record ID
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_flexible_id"
    )]
    pub id: Option<String>,

    /// URL-friendly slug
    pub slug: String,

    /// Item title
    pub title: String,

    /// Short description
    pub description: String,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Whether this item is completed
    #[serde(default)]
    pub completed: bool,

    /// When the item was marked completed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,

    /// When the item was created
    pub created_at: DateTime<Utc>,

    /// Display order
    #[serde(default)]
    pub order: i32,
}

/// Request payload for creating a new TODO item via admin API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTodoRequest {
    /// Item title
    pub title: String,

    /// URL-friendly slug (auto-generated from title if absent)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,

    /// Short description
    pub description: String,

    /// Optional long-form Typst source content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Optional pre-rendered HTML content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_content: Option<String>,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Whether this item starts as completed
    #[serde(default)]
    pub completed: bool,

    /// Display order
    #[serde(default)]
    pub order: i32,
}

/// Request payload for updating an existing TODO item (all fields optional for partial updates)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTodoRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html_content: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<i32>,
}
