use serde::Deserialize;

/// Error response from the API
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}
