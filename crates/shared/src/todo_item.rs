use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::serde_helpers::deserialize_flexible_id;

/// Full TODO/bucket-list item with all fields (used on detail page)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TodoItem {
    /// SurrealDB record ID
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
    /// SurrealDB record ID
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

impl TodoItem {
    /// Generate a URL-friendly slug from title
    pub fn slugify(title: &str) -> String {
        title
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() || c == '-' {
                    '-'
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(TodoItem::slugify("Learn Rust"), "learn-rust");
        assert_eq!(TodoItem::slugify("Visit Japan!"), "visit-japan_");
        assert_eq!(TodoItem::slugify("  spaces  "), "spaces");
    }

    #[test]
    fn test_todo_item_serialization_roundtrip() {
        let item = TodoItem {
            id: None,
            slug: "learn-rust".to_string(),
            title: "Learn Rust".to_string(),
            description: "Deep dive into Rust programming".to_string(),
            content: None,
            html_content: None,
            tags: vec!["programming".to_string(), "goals".to_string()],
            completed: false,
            completed_at: None,
            created_at: chrono::Utc::now(),
            order: 0,
        };
        let json = serde_json::to_string(&item).unwrap();
        let deserialized: TodoItem = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.slug, "learn-rust");
        assert_eq!(deserialized.tags, vec!["programming", "goals"]);
        assert!(!deserialized.completed);
    }

    #[test]
    fn test_todo_list_item_serialization_roundtrip() {
        let item = TodoListItem {
            id: Some("todos:abc".to_string()),
            slug: "learn-rust".to_string(),
            title: "Learn Rust".to_string(),
            description: "Deep dive into Rust".to_string(),
            tags: vec![],
            completed: true,
            completed_at: Some(chrono::Utc::now()),
            created_at: chrono::Utc::now(),
            order: 1,
        };
        let json = serde_json::to_string(&item).unwrap();
        let deserialized: TodoListItem = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.slug, "learn-rust");
        assert!(deserialized.completed);
        assert!(deserialized.completed_at.is_some());
    }

    #[test]
    fn test_create_request_skip_none_fields() {
        let req = CreateTodoRequest {
            title: "Test".to_string(),
            slug: None,
            description: "Desc".to_string(),
            content: None,
            html_content: None,
            tags: vec![],
            completed: false,
            order: 0,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("slug"));
        assert!(!json.contains("content"));
        assert!(!json.contains("html_content"));
        assert!(json.contains("title"));
    }

    #[test]
    fn test_update_request_all_none() {
        let req = UpdateTodoRequest {
            title: None,
            description: None,
            content: None,
            html_content: None,
            tags: None,
            completed: None,
            order: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(json, "{}");
    }
}
