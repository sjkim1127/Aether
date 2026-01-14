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
pub mod error;

pub use openai::OpenAiProvider;
pub use anthropic::AnthropicProvider;
pub use ollama::OllamaProvider;
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
