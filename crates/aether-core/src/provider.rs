//! AI Provider trait and configuration.
//!
//! Defines the interface that AI backends must implement.

use crate::{Result, Slot};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Configuration for an AI provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API key for authentication.
    pub api_key: String,

    /// Model identifier (e.g., "gpt-4", "claude-3").
    pub model: String,

    /// Base URL for the API.
    pub base_url: Option<String>,

    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,

    /// Temperature for generation (0.0 - 2.0).
    pub temperature: Option<f32>,

    /// Request timeout in seconds.
    pub timeout_seconds: Option<u64>,

    /// Optional URL to fetch the API key from (for stealth/security).
    pub api_key_url: Option<String>,
}

impl ProviderConfig {
    /// Create a new provider config with API key and model.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            base_url: None,
            max_tokens: None,
            temperature: None,
            timeout_seconds: None,
            api_key_url: None,
        }
    }

    /// Set an external URL to fetch the API key from.
    pub fn with_api_key_url(mut self, url: impl Into<String>) -> Self {
        self.api_key_url = Some(url.into());
        self
    }

    /// Resolve the API key (literal or remote).
    pub async fn resolve_api_key(&self) -> Result<String> {
        if let Some(ref url) = self.api_key_url {
            let resp = reqwest::get(url)
                .await
                .map_err(|e| crate::AetherError::NetworkError(format!("Failed to fetch API key: {}", e)))?;
            
            let key = resp
                .text()
                .await
                .map_err(|e| crate::AetherError::NetworkError(format!("Failed to read API key body: {}", e)))?;
            
            Ok(key.trim().to_string())
        } else {
            Ok(self.api_key.clone())
        }
    }

    /// Set the base URL.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set max tokens.
    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    /// Set temperature.
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp.clamp(0.0, 2.0));
        self
    }

    /// Set timeout.
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }

    /// Load config from environment variables.
    ///
    /// Expected variables:
    /// - `AETHER_API_KEY` or `OPENAI_API_KEY`
    /// - `AETHER_MODEL` (defaults to "gpt-4")
    /// - `AETHER_BASE_URL` (optional)
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("AETHER_API_KEY")
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .map_err(|_| {
                crate::AetherError::ConfigError(
                    "AETHER_API_KEY or OPENAI_API_KEY must be set".to_string(),
                )
            })?;

        let model = std::env::var("AETHER_MODEL").unwrap_or_else(|_| "gpt-5.2-thinking".to_string());

        let mut config = Self::new(api_key, model);

        if let Ok(url) = std::env::var("AETHER_BASE_URL") {
            config = config.with_base_url(url);
        }

        Ok(config)
    }
}

/// Request for code generation.
#[derive(Debug, Clone)]
pub struct GenerationRequest {
    /// The slot to generate code for.
    pub slot: Slot,

    /// Additional context (e.g., surrounding code).
    pub context: Option<String>,

    /// System prompt override.
    pub system_prompt: Option<String>,
}

use futures::stream::BoxStream;

/// Response from code generation.
#[derive(Debug, Clone)]
pub struct GenerationResponse {
    /// The generated code.
    pub code: String,

    /// Tokens used for the request.
    pub tokens_used: Option<u32>,

    /// Generation metadata.
    pub metadata: Option<serde_json::Value>,
}

/// A single chunk of a streaming response.
#[derive(Debug, Clone)]
pub struct StreamResponse {
    /// The new text chunk.
    pub delta: String,

    /// Final metadata (only sent in the last chunk).
    pub metadata: Option<serde_json::Value>,
}

/// Trait that AI providers must implement.
///
/// This trait defines the interface for generating code from slots.
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Get the provider name.
    fn name(&self) -> &str;

    /// Generate code for a slot.
    ///
    /// # Arguments
    ///
    /// * `request` - The generation request containing slot info
    ///
    /// # Returns
    ///
    /// Generated code response or an error.
    async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse>;

    /// Generate a stream of code for a slot.
    ///
    /// # Arguments
    ///
    /// * `request` - The generation request containing slot info
    ///
    /// # Returns
    ///
    /// A pinned stream of chunks or an error.
    fn generate_stream(
        &self,
        _request: GenerationRequest,
    ) -> BoxStream<'static, Result<StreamResponse>> {
        let name = self.name().to_string();
        Box::pin(async_stream::stream! {
            yield Err(crate::AetherError::ProviderError(format!(
                "Streaming not implemented for provider: {}",
                name
            )));
        })
    }

    /// Generate code for multiple slots in batch.
    ///
    /// Default implementation calls `generate` for each slot sequentially.
    async fn generate_batch(
        &self,
        requests: Vec<GenerationRequest>,
    ) -> Result<Vec<GenerationResponse>> {
        let mut responses = Vec::with_capacity(requests.len());
        for request in requests {
            responses.push(self.generate(request).await?);
        }
        Ok(responses)
    }

    /// Check if the provider is available and configured correctly.
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

/// A mock provider for testing.
#[derive(Debug, Default)]
pub struct MockProvider {
    /// Responses to return (slot_name -> code).
    pub responses: std::collections::HashMap<String, String>,
}

impl MockProvider {
    /// Create a new mock provider.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a mock response.
    pub fn with_response(mut self, slot: impl Into<String>, code: impl Into<String>) -> Self {
        self.responses.insert(slot.into(), code.into());
        self
    }
}

#[async_trait]
impl AiProvider for MockProvider {
    fn name(&self) -> &str {
        "mock"
    }

    async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse> {
        let code = self
            .responses
            .get(&request.slot.name)
            .cloned()
            .unwrap_or_else(|| format!("// Generated code for: {}", request.slot.name));

        Ok(GenerationResponse {
            code,
            tokens_used: Some(10),
            metadata: None,
        })
    }

    fn generate_stream(
        &self,
        request: GenerationRequest,
    ) -> BoxStream<'static, Result<StreamResponse>> {
        let code = self
            .responses
            .get(&request.slot.name)
            .cloned()
            .unwrap_or_else(|| format!("// Generated code for: {}", request.slot.name));

        use futures::StreamExt;
        let words: Vec<String> = code.split_whitespace().map(|s| format!("{} ", s)).collect();
        
        let stream = async_stream::stream! {
            for word in words {
                yield Ok(StreamResponse {
                    delta: word,
                    metadata: None,
                });
            }
        };
        
        Box::pin(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockProvider::new()
            .with_response("button", "<button>Click me</button>");

        let request = GenerationRequest {
            slot: Slot::new("button", "Create a button"),
            context: None,
            system_prompt: None,
        };

        let response = provider.generate(request).await.unwrap();
        assert_eq!(response.code, "<button>Click me</button>");
    }
}
