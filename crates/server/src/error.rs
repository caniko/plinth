use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// Structured error type for the Plinth server.
///
/// Maps to appropriate HTTP status codes and consistent JSON error responses.
#[derive(Debug, thiserror::Error)]
pub enum PlinthError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Actor error: {0}")]
    Actor(String),

    #[error("External service error: {0}")]
    External(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// JSON error response body.
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
    pub details: Option<String>,
}

impl IntoResponse for PlinthError {
    fn into_response(self) -> Response {
        let status = match &self {
            PlinthError::NotFound(_) => StatusCode::NOT_FOUND,
            PlinthError::Validation(_) => StatusCode::BAD_REQUEST,
            PlinthError::Database(_) | PlinthError::Actor(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PlinthError::External(_) => StatusCode::BAD_GATEWAY,
            PlinthError::Serialization(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = ErrorBody {
            error: self.to_string(),
            details: None,
        };

        (status, Json(body)).into_response()
    }
}

impl PlinthError {
    /// Create a Database error from a SurrealDB error.
    pub fn db(e: impl std::fmt::Display) -> Self {
        PlinthError::Database(e.to_string())
    }

    /// Create a Validation error.
    pub fn validation(msg: impl Into<String>) -> Self {
        PlinthError::Validation(msg.into())
    }

    /// Create a NotFound error.
    pub fn not_found(msg: impl Into<String>) -> Self {
        PlinthError::NotFound(msg.into())
    }

    /// Create an Actor error from an actor send error.
    pub fn actor(e: impl std::fmt::Display) -> Self {
        PlinthError::Actor(e.to_string())
    }
}
