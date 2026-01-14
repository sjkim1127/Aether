//! AI-specific error types.

use thiserror::Error;

/// Errors specific to AI operations.
#[derive(Debug, Error)]
pub enum AiError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// API returned an error response.
    #[error("API error: {status} - {message}")]
    ApiError {
        status: u16,
        message: String,
    },

    /// Rate limit exceeded.
    #[error("Rate limit exceeded. Retry after {retry_after} seconds")]
    RateLimited {
        retry_after: u64,
    },

    /// Invalid API key.
    #[error("Invalid API key")]
    InvalidApiKey,

    /// Model not found.
    #[error("Model '{0}' not found")]
    ModelNotFound(String),

    /// Response parsing failed.
    #[error("Failed to parse response: {0}")]
    ParseError(String),

    /// Content filter triggered.
    #[error("Content filter triggered: {0}")]
    ContentFiltered(String),
}

impl From<AiError> for aether_core::AetherError {
    fn from(e: AiError) -> Self {
        aether_core::AetherError::ProviderError(e.to_string())
    }
}
