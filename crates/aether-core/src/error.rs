//! Error types for Aether Core.

use thiserror::Error;

/// Result type alias for Aether operations.
pub type Result<T> = std::result::Result<T, AetherError>;

/// Main error type for the Aether framework.
#[derive(Debug, Error)]
pub enum AetherError {
    /// Template parsing failed.
    #[error("Template parse error: {0}")]
    TemplateParse(String),

    /// Slot not found in template.
    #[error("Slot '{0}' not found in template")]
    SlotNotFound(String),

    /// AI provider returned an error.
    #[error("AI provider error: {0}")]
    ProviderError(String),

    /// Network request failed.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Code injection failed.
    #[error("Injection error: {0}")]
    InjectionError(String),

    /// Invalid configuration.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Template rendering failed.
    #[error("Render error: {0}")]
    RenderError(String),

    /// IO operation failed.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON serialization/deserialization failed.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Timeout occurred.
    #[error("Operation timed out after {0} seconds")]
    Timeout(u64),
}
