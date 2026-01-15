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
use std::sync::Arc;
use tokio::sync::Mutex;

use aether_core::{
    InjectionContext as CoreContext,
    InjectionEngine as CoreEngine,
    Slot as CoreSlot,
    SlotKind as CoreSlotKind,
    Template as CoreTemplate,
    RenderSession as CoreRenderSession,
    AetherRuntime,
    AetherConfig,
    toon::Toon,
};
use aether_ai::{OpenAiProvider, AnthropicProvider, OllamaProvider};
use aether_core::AiProvider;
use rhai::Dynamic;

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

    /// Add a slot with a prompt and optional temperature, model, and max_tokens.
    #[napi]
    pub fn set_slot(&mut self, name: String, prompt: String, temperature: Option<f64>, model: Option<String>, max_tokens: Option<u32>) {
        let mut slot = CoreSlot::new(name, prompt);
        if let Some(temp) = temperature {
            slot.temperature = Some(temp as f32);
        }
        if let Some(m) = model {
            slot.model = Some(m);
        }
        if let Some(mt) = max_tokens {
            slot.max_tokens = Some(mt);
        }
        self.inner = self.inner.clone().configure_slot(slot);
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

/// JavaScript-accessible RenderSession class for incremental rendering.
#[napi]
pub struct RenderSession {
    inner: Mutex<CoreRenderSession>,
}

#[napi]
impl RenderSession {
    /// Create a new empty render session.
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(CoreRenderSession::new()),
        }
    }

    /// Get the number of cached slot results.
    #[napi(getter)]
    pub fn cached_count(&self) -> u32 {
        self.inner.blocking_lock().results.len() as u32
    }

    /// Clear all cached results.
    #[napi]
    pub fn clear(&self) {
        self.inner.blocking_lock().results.clear();
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

    /// Set temperature for this slot.
    #[napi]
    pub fn set_temperature(&mut self, temp: f64) {
        self.inner.temperature = Some(temp as f32);
    }

    /// Set model override for this slot.
    #[napi]
    pub fn set_model(&mut self, model: String) {
        self.inner.model = Some(model);
    }

    /// Set maximum tokens for this slot.
    #[napi]
    pub fn set_max_tokens(&mut self, max_tokens: u32) {
        self.inner.max_tokens = Some(max_tokens);
    }
}

/// Provider type enum for JavaScript.
#[napi(string_enum)]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Ollama,
    Gemini,
    Grok,
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
    pub api_key_url: Option<String>,
}

/// Main Aether engine for JavaScript.
#[napi]
pub struct AetherEngine {
    provider_type: ProviderType,
    model: String,
    api_key: Option<String>,
    context: Option<CoreContext>,
    config: AetherConfig,
    api_key_url: Option<String>,
}

#[napi]
impl AetherEngine {
    /// Create a new engine with OpenAI provider.
    #[napi(factory)]
    pub fn openai(model: Option<String>) -> Result<Self> {
        Ok(Self {
            provider_type: ProviderType::OpenAI,
            model: model.unwrap_or_else(|| "gpt-4o".to_string()),
            api_key: std::env::var("OPENAI_API_KEY").ok(),
            context: None,
            config: AetherConfig::default(),
            api_key_url: None,
        })
    }

    /// Create a new engine with Anthropic provider.
    #[napi(factory)]
    pub fn anthropic(model: Option<String>) -> Result<Self> {
        Ok(Self {
            provider_type: ProviderType::Anthropic,
            model: model.unwrap_or_else(|| "claude-3-5-sonnet-latest".to_string()),
            api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            context: None,
            config: AetherConfig::default(),
            api_key_url: None,
        })
    }

    /// Create a new engine with Google Gemini provider.
    #[napi(factory)]
    pub fn gemini(model: Option<String>) -> Result<Self> {
        Ok(Self {
            provider_type: ProviderType::Gemini,
            model: model.unwrap_or_else(|| "gemini-1.5-pro".to_string()),
            api_key: std::env::var("GOOGLE_API_KEY").ok(),
            context: None,
            config: AetherConfig::default(),
            api_key_url: None,
        })
    }

    /// Create a new engine with Grok (xAI) provider.
    #[napi(factory)]
    pub fn grok(model: Option<String>) -> Result<Self> {
        Ok(Self {
            provider_type: ProviderType::Grok,
            model: model.unwrap_or_else(|| "grok-1".to_string()),
            api_key: std::env::var("XAI_API_KEY").ok(),
            context: None,
            config: AetherConfig::default(),
            api_key_url: None,
        })
    }

    /// Create a new engine with Ollama provider (local).
    #[napi(factory)]
    pub fn ollama(model: String) -> Result<Self> {
        Ok(Self {
            provider_type: ProviderType::Ollama,
            model,
            api_key: None,
            context: None,
            config: AetherConfig::default(),
            api_key_url: None,
        })
    }

    /// Set the API key.
    #[napi]
    pub fn set_api_key(&mut self, key: String) {
        self.api_key = Some(key);
    }

    /// Set the API key URL for remote resolution.
    #[napi]
    pub fn set_api_key_url(&mut self, url: String) {
        self.api_key_url = Some(url);
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

    /// Enable or disable incremental caching.
    #[napi]
    pub fn set_cache(&mut self, enabled: bool) {
        self.config.cache_enabled = enabled;
    }

    /// Enable or disable TOON formatting.
    #[napi]
    pub fn set_toon(&mut self, enabled: bool) {
        self.config.toon_enabled = enabled;
    }

    /// Enable or disable parallel generation.
    #[napi]
    pub fn set_parallel(&mut self, enabled: bool) {
        self.config.parallel = enabled;
    }

    /// Set maximum retries.
    #[napi]
    pub fn set_max_retries(&mut self, retries: u32) {
        self.config.max_retries = retries;
    }

    /// Enable or disable self-healing.
    #[napi]
    pub fn set_heal(&mut self, enabled: bool) {
        self.config.healing_enabled = enabled;
    }

    /// Deserialize a TOON string back into a JSON structure.
    #[napi]
    pub fn toon_deserialize(&self, toon_str: String) -> Result<String> {
        let val = Toon::deserialize(&toon_str)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        serde_json::to_string(&val)
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Execute a Rhai script directly (Aether Shield core functionality).
    /// 
    /// # Arguments
    /// * `script` - The Rhai script to execute.
    /// * `inputs_json` - Optional JSON string of input variables (e.g., '{"x": 10, "name": "Alice"}').
    /// 
    /// # Returns
    /// The result of the script execution as a string.
    #[napi]
    pub fn execute_script(&self, script: String, inputs_json: Option<String>) -> Result<String> {
        let runtime = AetherRuntime::new();
        
        let mut rhai_inputs: HashMap<String, Dynamic> = HashMap::new();
        
        if let Some(json_str) = inputs_json {
            if let Ok(parsed) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&json_str) {
                for (key, value) in parsed {
                    let dyn_val = match value {
                        serde_json::Value::Bool(b) => Dynamic::from(b),
                        serde_json::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                Dynamic::from(i)
                            } else if let Some(f) = n.as_f64() {
                                Dynamic::from(f)
                            } else {
                                Dynamic::from(0)
                            }
                        },
                        serde_json::Value::String(s) => Dynamic::from(s),
                        _ => continue,
                    };
                    rhai_inputs.insert(key, dyn_val);
                }
            }
        }

        let result = runtime.execute(&script, rhai_inputs)
            .map_err(|e| Error::from_reason(e.to_string()))?;

        Ok(result.to_string())
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

    async fn render_internal(&self, template: &CoreTemplate) -> Result<String> {
        match self.provider_type {
            ProviderType::OpenAI => {
                let api_key = self.api_key.clone()
                    .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                    .unwrap_or_default();
                
                let mut config = aether_core::ProviderConfig::new(&api_key, &self.model);
                if let Some(ref url) = self.api_key_url {
                    config = config.with_api_key_url(url);
                }

                let provider = OpenAiProvider::new(config)
                    .map_err(|e| Error::from_reason(e.to_string()))?;
                
                self.render_with_provider(template, provider).await
            }
            ProviderType::Anthropic => {
                let api_key = self.api_key.clone()
                    .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                    .unwrap_or_default();
                
                let mut config = aether_core::ProviderConfig::new(&api_key, &self.model);
                if let Some(ref url) = self.api_key_url {
                    config = config.with_api_key_url(url);
                }

                let provider = AnthropicProvider::new(config)
                    .map_err(|e| Error::from_reason(e.to_string()))?;
                
                self.render_with_provider(template, provider).await
            }
            ProviderType::Gemini => {
                let api_key = self.api_key.clone()
                    .or_else(|| std::env::var("GOOGLE_API_KEY").ok())
                    .unwrap_or_default();
                
                let mut config = aether_core::ProviderConfig::new(&api_key, &self.model);
                if let Some(ref url) = self.api_key_url {
                    config = config.with_api_key_url(url);
                }

                let provider = aether_ai::GeminiProvider::new(config)
                    .map_err(|e| Error::from_reason(e.to_string()))?;
                self.render_with_provider(template, provider).await
            }
            ProviderType::Ollama => {
                let provider = OllamaProvider::new(&self.model);
                self.render_with_provider(template, provider).await
            }
            ProviderType::Grok => {
                let api_key = self.api_key.clone()
                    .or_else(|| std::env::var("XAI_API_KEY").ok())
                    .unwrap_or_default();
                
                let mut config = aether_core::ProviderConfig::new(&api_key, &self.model)
                    .with_base_url("https://api.x.ai/v1/chat/completions");

                if let Some(ref url) = self.api_key_url {
                    config = config.with_api_key_url(url);
                }

                let provider = OpenAiProvider::new(config)
                    .map_err(|e| Error::from_reason(e.to_string()))?;
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
        let mut engine = CoreEngine::with_config(provider, self.config.clone());
        
        if let Some(ref ctx) = self.context {
            engine = engine.with_context(ctx.clone());
        }

        // Apply Premium Features if enabled in config but not yet in engine
        if self.config.cache_enabled && engine.cache().is_none() {
            engine = engine.with_cache(aether_core::cache::SemanticCache::new().map_err(|e| Error::from_reason(e.to_string()))?);
        }
        
        engine.render(template).await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Render a template incrementally using a session to cache results.
    ///
    /// Only slots that have changed since the last render will be regenerated.
    ///
    /// # Example (JavaScript)
    /// ```javascript
    /// const session = new RenderSession();
    /// const result1 = await engine.renderIncremental(template, session);  // Full render
    /// const result2 = await engine.renderIncremental(template, session);  // Uses cache
    /// ```
    pub async fn render_incremental(
        &self,
        template: &Template,
        session: &RenderSession,
    ) -> Result<String> {
        let provider = match self.provider_type {
            ProviderType::OpenAI => {
                let api_key = self.api_key.clone().or_else(|| std::env::var("OPENAI_API_KEY").ok()).unwrap_or_default();
                let mut config = aether_core::ProviderConfig::new(&api_key, &self.model);
                if let Some(ref url) = self.api_key_url { config = config.with_api_key_url(url); }
                Arc::new(OpenAiProvider::new(config).map_err(|e| Error::from_reason(e.to_string()))?) as Arc<dyn AiProvider>
            }
            ProviderType::Anthropic => {
                let api_key = self.api_key.clone().or_else(|| std::env::var("ANTHROPIC_API_KEY").ok()).unwrap_or_default();
                let mut config = aether_core::ProviderConfig::new(&api_key, &self.model);
                if let Some(ref url) = self.api_key_url { config = config.with_api_key_url(url); }
                Arc::new(AnthropicProvider::new(config).map_err(|e| Error::from_reason(e.to_string()))?) as Arc<dyn AiProvider>
            }
            ProviderType::Gemini => {
                let api_key = self.api_key.clone().or_else(|| std::env::var("GOOGLE_API_KEY").ok()).unwrap_or_default();
                let mut config = aether_core::ProviderConfig::new(&api_key, &self.model);
                if let Some(ref url) = self.api_key_url { config = config.with_api_key_url(url); }
                Arc::new(aether_ai::GeminiProvider::new(config).map_err(|e| Error::from_reason(e.to_string()))?) as Arc<dyn AiProvider>
            }
            ProviderType::Ollama => Arc::new(OllamaProvider::new(&self.model)) as Arc<dyn AiProvider>,
            ProviderType::Grok => {
                let api_key = self.api_key.clone().or_else(|| std::env::var("XAI_API_KEY").ok()).unwrap_or_default();
                let config = aether_core::ProviderConfig::new(&api_key, &self.model).with_base_url("https://api.x.ai/v1/chat/completions");
                Arc::new(OpenAiProvider::new(config).map_err(|e| Error::from_reason(e.to_string()))?) as Arc<dyn AiProvider>
            }
        };

        let mut engine = CoreEngine::with_config_arc(provider, self.config.clone());
        if let Some(ref ctx) = self.context { engine = engine.with_context(ctx.clone()); }
        
        engine.render_incremental(&template.inner, &mut *session.inner.lock().await).await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Get streaming chunks as an array (alternative to callback-based streaming).
    /// Returns an array of strings, each representing a chunk of the generated content.
    #[napi]
    pub async fn get_stream_chunks(
        &self,
        template: &Template,
        slot_name: String,
    ) -> Result<Vec<String>> {
        use futures::StreamExt;

        match self.provider_type {
            ProviderType::OpenAI => {
                let api_key = self.api_key.clone()
                    .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                    .unwrap_or_default();
                
                let config = aether_core::ProviderConfig::new(&api_key, &self.model);
                let provider = OpenAiProvider::new(config)
                    .map_err(|e| Error::from_reason(e.to_string()))?;
                
                self.collect_stream_chunks(&template.inner, &slot_name, provider).await
            }
            ProviderType::Anthropic => {
                let api_key = self.api_key.clone()
                    .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                    .unwrap_or_default();
                
                let config = aether_core::ProviderConfig::new(&api_key, &self.model);
                let provider = AnthropicProvider::new(config)
                    .map_err(|e| Error::from_reason(e.to_string()))?;
                
                self.collect_stream_chunks(&template.inner, &slot_name, provider).await
            }
            ProviderType::Gemini => {
                let api_key = self.api_key.clone()
                    .or_else(|| std::env::var("GOOGLE_API_KEY").ok())
                    .unwrap_or_default();
                
                let config = aether_core::ProviderConfig::new(&api_key, &self.model);
                let provider = aether_ai::GeminiProvider::new(config)
                    .map_err(|e| Error::from_reason(e.to_string()))?;
                
                self.collect_stream_chunks(&template.inner, &slot_name, provider).await
            }
            ProviderType::Ollama => {
                let provider = OllamaProvider::new(&self.model);
                self.collect_stream_chunks(&template.inner, &slot_name, provider).await
            }
            ProviderType::Grok => {
                let api_key = self.api_key.clone()
                    .or_else(|| std::env::var("XAI_API_KEY").ok())
                    .unwrap_or_default();
                
                let config = aether_core::ProviderConfig::new(&api_key, &self.model)
                    .with_base_url("https://api.x.ai/v1/chat/completions");
                let provider = OpenAiProvider::new(config)
                    .map_err(|e| Error::from_reason(e.to_string()))?;
                
                self.collect_stream_chunks(&template.inner, &slot_name, provider).await
            }
        }
    }

    async fn collect_stream_chunks<P: AiProvider + 'static>(
        &self,
        template: &CoreTemplate,
        slot_name: &str,
        provider: P,
    ) -> Result<Vec<String>> {
        use futures::StreamExt;

        let mut engine = CoreEngine::with_config(provider, self.config.clone());
        if let Some(ref ctx) = self.context { engine = engine.with_context(ctx.clone()); }
        
        match engine.generate_slot_stream(template, slot_name) {
            Ok(mut stream) => {
                let mut chunks = Vec::new();
                while let Some(result) = stream.next().await {
                    match result {
                        Ok(chunk) => chunks.push(chunk.delta),
                        Err(e) => return Err(Error::from_reason(e.to_string())),
                    }
                }
                Ok(chunks)
            }
            Err(e) => Err(Error::from_reason(e.to_string()))
        }
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
        "ollama" | "local" => AetherEngine::ollama("llama3".to_string())?,
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
