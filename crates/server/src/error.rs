use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::error;

/// Structured error type for the Plinth server.
///
/// Maps to appropriate HTTP status codes and consistent JSON error responses.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PlinthError {
    #[error("Database error: {0}")]
    Database(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Actor error: {0}")]
    Actor(#[source] Box<dyn std::error::Error + Send + Sync>),

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

        // Server errors (5xx): log the full detail server-side but return only a
        // generic message — internal details (DB errors, etc.) must never reach
        // the client. Client errors (4xx) carry a message that is safe and
        // useful to return verbatim.
        let client_error = if status.is_server_error() {
            error!(error = %self, "request failed");
            match &self {
                PlinthError::External(_) => "Upstream service error",
                _ => "Internal server error",
            }
            .to_string()
        } else {
            self.to_string()
        };

        let body = ErrorBody {
            error: client_error,
            details: None,
        };

        (status, Json(body)).into_response()
    }
}

impl PlinthError {
    /// Create a Database error from a lower-level database error.
    pub fn db(e: impl std::fmt::Display) -> Self {
        PlinthError::Database(e.to_string().into())
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
        PlinthError::Actor(e.to_string().into())
    }
}

impl From<sqlx::Error> for PlinthError {
    fn from(e: sqlx::Error) -> Self {
        PlinthError::Database(Box::new(e))
    }
}

impl From<serde_json::Error> for PlinthError {
    fn from(e: serde_json::Error) -> Self {
        PlinthError::Serialization(e.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for PlinthError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        PlinthError::Actor(e)
    }
}
