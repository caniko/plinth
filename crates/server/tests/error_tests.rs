//! Tests for the PlinthError type and its HTTP status code mapping.

use axum::http::StatusCode;
use axum::response::IntoResponse;
use plinth_server::error::PlinthError;

#[test]
fn test_not_found_returns_404() {
    let error = PlinthError::NotFound("post not found".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_validation_returns_400() {
    let error = PlinthError::Validation("title is required".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_database_returns_500() {
    let error = PlinthError::Database("connection lost".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_actor_returns_500() {
    let error = PlinthError::Actor("mailbox full".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_external_returns_502() {
    let error = PlinthError::External("immich unreachable".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
}

#[test]
fn test_serialization_returns_500() {
    let error = PlinthError::Serialization("invalid json".to_string());
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_error_body_is_json() {
    let error = PlinthError::NotFound("post xyz".to_string());
    let response = error.into_response();

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(parsed["error"].as_str().unwrap().contains("post xyz"));
}

#[test]
fn test_helper_constructors() {
    let db_err = PlinthError::db("timeout");
    assert!(matches!(db_err, PlinthError::Database(_)));
    assert!(db_err.to_string().contains("timeout"));

    let val_err = PlinthError::validation("bad input");
    assert!(matches!(val_err, PlinthError::Validation(_)));

    let nf_err = PlinthError::not_found("missing");
    assert!(matches!(nf_err, PlinthError::NotFound(_)));

    let act_err = PlinthError::actor("dead");
    assert!(matches!(act_err, PlinthError::Actor(_)));
}

#[test]
fn test_display_includes_message() {
    let error = PlinthError::Database("connection refused".to_string());
    let display = error.to_string();
    assert!(display.contains("connection refused"));
    assert!(display.contains("Database error"));
}
