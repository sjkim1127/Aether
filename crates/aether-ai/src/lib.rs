//! # Aether AI
//!
//! AI provider implementations for the Aether code injection framework.
//!
//! This crate provides ready-to-use AI backends:
//!
//! - **OpenAI**: GPT-4, GPT-3.5-turbo
//! - **Anthropic**: Claude models
//! - **Local**: Ollama and other local providers
//!
//! ## Example
//!
//! ```rust,ignore
//! use aether_ai::OpenAiProvider;
//! use aether_core::{InjectionEngine, Template};
//!
//! // One-line initialization from environment
//! let provider = OpenAiProvider::from_env()?;
//!
//! let engine = InjectionEngine::new(provider);
//! let template = Template::new("{{AI:greeting}}");
//! let result = engine.render(&template).await?;
//! ```

pub mod openai;
pub mod anthropic;
pub mod ollama;
pub mod gemini;
pub mod error;

pub use openai::OpenAiProvider;
pub use anthropic::AnthropicProvider;
pub use ollama::OllamaProvider;
pub use gemini::GeminiProvider;
pub use error::AiError;

/// Re-export core types for convenience.
pub use aether_core::{
    AiProvider, ProviderConfig, InjectionEngine, Template, Slot,
    InjectionContext, AetherError, Result,
};

/// Create an OpenAI provider with a single line.
///
/// # Example
///
/// ```rust,ignore
/// let provider = aether_ai::openai("gpt-5.2-thinking");
/// ```
pub fn openai(model: &str) -> Result<OpenAiProvider> {
    OpenAiProvider::from_env_with_model(model)
}

/// Create an Anthropic provider with a single line.
///
/// # Example
///
/// ```rust,ignore
/// let provider = aether_ai::anthropic("claude-opus-4-5");
/// ```
pub fn anthropic(model: &str) -> Result<AnthropicProvider> {
    AnthropicProvider::from_env_with_model(model)
}

/// Create a Google Gemini provider with a single line.
///
/// # Example
///
/// ```rust,ignore
/// let provider = aether_ai::gemini("gemini-1.5-pro");
/// ```
pub fn gemini(model: &str) -> Result<GeminiProvider> {
    match GeminiProvider::from_env() {
        Ok(mut p) => {
             let api_key = std::env::var("GOOGLE_API_KEY")
                .map_err(|_| AetherError::ConfigError("GOOGLE_API_KEY not set".to_string()))?;
             let config = ProviderConfig::new(api_key, model);
             GeminiProvider::new(config)
        },
        Err(e) => Err(e)
    }
}

/// Create a Grok (xAI) provider with a single line.
///
/// Uses the OpenAI-compatible API from xAI.
/// Requires `XAI_API_KEY` environment variable.
///
/// # Example
///
/// ```rust,ignore
/// let provider = aether_ai::grok("grok-1");
/// ```
pub fn grok(model: &str) -> Result<OpenAiProvider> {
    let api_key = std::env::var("XAI_API_KEY")
        .map_err(|_| AetherError::ConfigError("XAI_API_KEY not set".to_string()))?;

    let config = ProviderConfig::new(api_key, model)
        .with_base_url("https://api.x.ai/v1/chat/completions"); // OpenAI impl appends nothing, wait.
    
    // Check openai.rs implementation. It appends nothing to base_url?
    // openai.rs: let url = self.config.base_url.as_deref().unwrap_or(OPENAI_API_URL);
    // OPENAI_API_URL is full path.
    // So we need full path here.
    
    let config = config.with_base_url("https://api.x.ai/v1/chat/completions");

    OpenAiProvider::new(config)
}

/// Create an Ollama provider with a single line.
///
/// # Example
///
/// ```rust,ignore
/// let provider = aether_ai::ollama("codellama");
/// ```
pub fn ollama(model: &str) -> OllamaProvider {
    OllamaProvider::new(model)
}
