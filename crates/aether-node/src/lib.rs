//! # Aether Node.js Bindings
//!
//! Node.js native module bindings for the Aether AI code injection framework.
//!
//! This crate provides JavaScript/TypeScript APIs for generating AI-powered code
//! using templates with slot-based injection.
//!
//! ## Usage from Node.js
//!
//! ```javascript
//! const { AetherEngine, Template } = require('@aether/codegen');
//!
//! // One-line AI code generation
//! const code = await AetherEngine.generate("Create a login form");
//!
//! // Or with templates
//! const template = new Template("<div>{{AI:content}}</div>");
//! template.setSlot("content", "Generate a welcome message");
//! const result = await engine.render(template);
//! ```

#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;

use aether_core::{
    InjectionContext as CoreContext,
    InjectionEngine as CoreEngine,
    Slot as CoreSlot,
    SlotKind as CoreSlotKind,
    Template as CoreTemplate,
};
use aether_ai::{OpenAiProvider, AnthropicProvider, OllamaProvider};
use aether_core::AiProvider;

/// JavaScript-accessible Template class.
#[napi]
pub struct Template {
    inner: CoreTemplate,
}

#[napi]
impl Template {
    /// Create a new template from content string.
    ///
    /// Use `{{AI:slot_name}}` syntax to define injection points.
    #[napi(constructor)]
    pub fn new(content: String) -> Self {
        Self {
            inner: CoreTemplate::new(content),
        }
    }

    /// Set the template name.
    #[napi]
    pub fn set_name(&mut self, name: String) {
        self.inner.name = name;
    }

    /// Add a slot with a prompt.
    #[napi]
    pub fn set_slot(&mut self, name: String, prompt: String) {
        self.inner = self.inner.clone().with_slot(name, prompt);
    }

    /// Get all slot names in this template.
    #[napi]
    pub fn get_slot_names(&self) -> Vec<String> {
        self.inner.slot_names().iter().map(|s| s.to_string()).collect()
    }

    /// Get the template content.
    #[napi(getter)]
    pub fn content(&self) -> String {
        self.inner.content.clone()
    }
}

/// JavaScript-accessible Slot class.
#[napi]
pub struct Slot {
    inner: CoreSlot,
}

#[napi]
impl Slot {
    /// Create a new slot.
    #[napi(constructor)]
    pub fn new(name: String, prompt: String) -> Self {
        Self {
            inner: CoreSlot::new(name, prompt),
        }
    }

    /// Set the slot kind.
    #[napi]
    pub fn set_kind(&mut self, kind: String) {
        let slot_kind = match kind.to_lowercase().as_str() {
            "html" => CoreSlotKind::Html,
            "css" => CoreSlotKind::Css,
            "javascript" | "js" => CoreSlotKind::JavaScript,
            "function" => CoreSlotKind::Function,
            "class" => CoreSlotKind::Class,
            "component" => CoreSlotKind::Component,
            _ => CoreSlotKind::Raw,
        };
        self.inner.kind = slot_kind;
    }

    /// Set maximum lines constraint.
    #[napi]
    pub fn set_max_lines(&mut self, max: u32) {
        let constraints = self.inner.constraints.get_or_insert_with(Default::default);
        constraints.max_lines = Some(max as usize);
    }
}

/// Provider type enum for JavaScript.
#[napi(string_enum)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Ollama,
}

/// Configuration for AI providers.
#[napi(object)]
pub struct ProviderConfig {
    pub provider: ProviderType,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
}

/// Main Aether engine for JavaScript.
#[napi]
pub struct AetherEngine {
    provider_type: ProviderType,
    model: String,
    api_key: Option<String>,
    context: Option<CoreContext>,
    parallel: bool,
    max_retries: u32,
}

#[napi]
impl AetherEngine {
    /// Create a new engine with OpenAI provider.
    #[napi(factory)]
    pub fn openai(model: Option<String>) -> Result<Self> {
        Ok(Self {
            provider_type: ProviderType::OpenAI,
            model: model.unwrap_or_else(|| "gpt-5.2-thinking".to_string()),
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            context: None,
            parallel: true,
            max_retries: 2,
        })
    }

    /// Create a new engine with Anthropic provider.
    #[napi(factory)]
    pub fn anthropic(model: Option<String>) -> Result<Self> {
        Ok(Self {
            provider_type: ProviderType::Anthropic,
            model: model.unwrap_or_else(|| "claude-opus-4-5".to_string()),
            api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            context: None,
            parallel: true,
            max_retries: 2,
        })
    }

    /// Create a new engine with Ollama provider.
    #[napi(factory)]
    pub fn ollama(model: Option<String>) -> Result<Self> {
        Ok(Self {
            provider_type: ProviderType::Ollama,
            model: model.unwrap_or_else(|| "codellama".to_string()),
            api_key: None,
            context: None,
            parallel: true,
            max_retries: 2,
        })
    }

    /// Set the API key.
    #[napi]
    pub fn set_api_key(&mut self, key: String) {
        self.api_key = Some(key);
    }

    /// Set context for generation.
    #[napi]
    pub fn set_context(&mut self, project: Option<String>, language: Option<String>, framework: Option<String>) {
        let mut ctx = CoreContext::new();
        if let Some(p) = project {
            ctx = ctx.with_project(p);
        }
        if let Some(l) = language {
            ctx = ctx.with_language(l);
        }
        if let Some(f) = framework {
            ctx = ctx.with_framework(f);
        }
        self.context = Some(ctx);
    }

    /// Enable or disable parallel generation.
    #[napi]
    pub fn set_parallel(&mut self, enabled: bool) {
        self.parallel = enabled;
    }

    /// Set maximum retries.
    #[napi]
    pub fn set_max_retries(&mut self, retries: u32) {
        self.max_retries = retries;
    }

    /// Generate code with a simple prompt (one-liner).
    #[napi]
    pub async fn generate(&self, prompt: String) -> Result<String> {
        let template = CoreTemplate::new("{{AI:generated}}")
            .with_slot("generated", prompt);
        
        self.render_internal(&template).await
    }

    /// Render a template with AI-generated code.
    #[napi]
    pub async fn render(&self, template: &Template) -> Result<String> {
        self.render_internal(&template.inner).await
    }

    /// Internal render implementation.
    async fn render_internal(&self, template: &CoreTemplate) -> Result<String> {
        match self.provider_type {
            ProviderType::OpenAI => {
                let api_key = self.api_key.clone()
                    .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                    .ok_or_else(|| Error::from_reason("OPENAI_API_KEY not set"))?;
                
                let config = aether_core::ProviderConfig::new(&api_key, &self.model);
                let provider = OpenAiProvider::new(config)
                    .map_err(|e| Error::from_reason(e.to_string()))?;
                
                self.render_with_provider(template, provider).await
            }
            ProviderType::Anthropic => {
                let api_key = self.api_key.clone()
                    .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                    .ok_or_else(|| Error::from_reason("ANTHROPIC_API_KEY not set"))?;
                
                let config = aether_core::ProviderConfig::new(&api_key, &self.model);
                let provider = AnthropicProvider::new(config)
                    .map_err(|e| Error::from_reason(e.to_string()))?;
                
                self.render_with_provider(template, provider).await
            }
            ProviderType::Ollama => {
                let provider = OllamaProvider::new(&self.model);
                self.render_with_provider(template, provider).await
            }
        }
    }

    /// Render with a specific provider.
    async fn render_with_provider<P: AiProvider + 'static>(
        &self,
        template: &CoreTemplate,
        provider: P,
    ) -> Result<String> {
        let mut engine = CoreEngine::new(provider)
            .parallel(self.parallel)
            .max_retries(self.max_retries);
        
        if let Some(ref ctx) = self.context {
            engine = engine.with_context(ctx.clone());
        }
        
        engine.render(template).await
            .map_err(|e| Error::from_reason(e.to_string()))
    }
}

/// One-line code generation function.
///
/// # Example
/// ```javascript
/// const code = await generate("Create a login form");
/// ```
#[napi]
pub async fn generate(prompt: String, provider: Option<String>) -> Result<String> {
    let provider_str = provider.unwrap_or_else(|| "openai".to_string());
    
    let engine = match provider_str.to_lowercase().as_str() {
        "anthropic" | "claude" => AetherEngine::anthropic(None)?,
        "ollama" | "local" => AetherEngine::ollama(None)?,
        _ => AetherEngine::openai(None)?,
    };
    
    engine.generate(prompt).await
}

/// Quick template rendering.
#[napi]
pub async fn render_template(content: String, slots: HashMap<String, String>) -> Result<String> {
    let mut template = CoreTemplate::new(content);
    
    for (name, prompt) in slots {
        template = template.with_slot(name, prompt);
    }
    
    let engine = AetherEngine::openai(None)?;
    engine.render_internal(&template).await
}
