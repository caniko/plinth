use super::*;

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
