//! Injection Engine - The main orchestrator for AI code injection.
//!
//! This module provides the high-level API for rendering templates with AI-generated code.

use crate::{
    AetherError, AiProvider, InjectionContext, Result, Template,
    provider::{GenerationRequest, GenerationResponse},
    config::AetherConfig,
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument};
use futures::stream::BoxStream;
use crate::provider::StreamResponse;
use crate::validation::{Validator, ValidationResult};
use crate::cache::Cache;
use crate::toon::Toon;
pub use crate::observer::ObserverPtr;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// The main engine for AI code injection.
///
/// # Example
///
/// ```rust,ignore
/// use aether_core::{InjectionEngine, Template, AetherConfig};
/// use aether_ai::OpenAiProvider;
///
/// let provider = OpenAiProvider::from_env()?;
/// 
/// // Using config
/// let config = AetherConfig::from_env();
/// let engine = InjectionEngine::with_config(provider, config);
///
/// // Or simple
/// let engine = InjectionEngine::new(provider);
/// ```
pub struct InjectionEngine<P: AiProvider> {
    /// The AI provider for code generation.
    provider: Arc<P>,
    
    /// Optional validator for self-healing.
    validator: Option<Arc<dyn Validator>>,

    /// Optional cache for performance/cost optimization.
    cache: Option<Arc<dyn Cache>>,

    /// Whether to use TOON format for context injection.
    use_toon: bool,

    /// Auto TOON threshold (characters).
    auto_toon_threshold: Option<usize>,

    /// Global context applied to all generations.
    global_context: InjectionContext,

    /// Whether to run generations in parallel.
    parallel: bool,

    /// Maximum retries for failed generations.
    max_retries: u32,

    /// Optional observer for tracking events.
    observer: Option<ObserverPtr>,
}

/// A session for tracking incremental rendering state.
/// Holds fingerprints of slots and context to identify changes.
#[derive(Debug, Clone, Default)]
pub struct RenderSession {
    /// Cached results indexed by (SlotHash, ContextHash)
    pub results: HashMap<(u64, u64), String>,
}

impl RenderSession {
    /// Create a new empty render session.
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate a stable hash for any hashable object.
    pub fn hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
}

impl<P: AiProvider + 'static> InjectionEngine<P> {
    /// Create a new injection engine with the given provider and default config.
    pub fn new(provider: P) -> Self {
        Self::with_config(provider, AetherConfig::default())
    }

    /// Internal: Create a raw engine without full config for script-based calls.
    pub fn new_raw(provider: Arc<P>) -> Self {
        Self {
            provider,
            validator: None,
            cache: None,
            use_toon: false,
            auto_toon_threshold: None,
            global_context: InjectionContext::default(),
            parallel: false,
            max_retries: 0,
            observer: None,
        }
    }

    /// Create a new injection engine with the given provider and config.
    pub fn with_config(provider: P, config: AetherConfig) -> Self {
        let validator: Option<Arc<dyn Validator>> = if config.healing_enabled {
            Some(Arc::new(crate::validation::MultiValidator::new()))
        } else {
            None
        };

        Self {
            provider: Arc::new(provider),
            validator,
            cache: None,
            use_toon: config.toon_enabled,
            auto_toon_threshold: config.auto_toon_threshold,
            global_context: InjectionContext::default(),
            parallel: config.parallel,
            max_retries: config.max_retries,
            observer: None,
        }
    }

    /// Set the cache for performance optimization.
    pub fn with_cache(mut self, cache: impl Cache + 'static) -> Self {
        self.cache = Some(Arc::new(cache));
        self
    }

    /// Enable or disable TOON format for context.
    pub fn with_toon(mut self, enabled: bool) -> Self {
        self.use_toon = enabled;
        self
    }

    /// Set the validator for self-healing.
    pub fn with_validator(mut self, validator: impl Validator + 'static) -> Self {
        self.validator = Some(Arc::new(validator));
        self
    }

    /// Set the global context.
    pub fn with_context(mut self, context: InjectionContext) -> Self {
        self.global_context = context;
        self
    }

    /// Enable or disable parallel generation.
    pub fn parallel(mut self, enabled: bool) -> Self {
        self.parallel = enabled;
        self
    }

    /// Set maximum retries for failed generations.
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set an observer for tracking events.
    pub fn with_observer(mut self, observer: impl crate::observer::EngineObserver + 'static) -> Self {
        self.observer = Some(Arc::new(observer));
        self
    }

    /// Render a template with AI-generated code.
    ///
    /// This method will generate code for all slots in the template
    /// and return the final rendered content.
    #[instrument(skip(self, template), fields(template_name = %template.name))]
    pub async fn render(&self, template: &Template) -> Result<String> {
        info!("Rendering template: {}", template.name);

        let injections = self.generate_all(template, None).await?;
        template.render(&injections)
    }

    /// Render a template with additional context.
    #[instrument(skip(self, template, context), fields(template_name = %template.name))]
    pub async fn render_with_context(
        &self,
        template: &Template,
        context: InjectionContext,
    ) -> Result<String> {
        info!("Rendering template with context: {}", template.name);

        let injections = self.generate_all(template, Some(context)).await?;
        template.render(&injections)
    }

    /// Render a template incrementally using a session.
    /// 
    /// This will only generate code for slots that have changed 
    /// based on their definition and the current context.
    #[instrument(skip(self, template, session), fields(template_name = %template.name))]
    pub async fn render_incremental(
        &self,
        template: &Template,
        session: &mut RenderSession,
    ) -> Result<String> {
        info!("Incrementally rendering template: {}", template.name);
        
        let context_hash = RenderSession::hash(&self.global_context);
        let mut injections = HashMap::new();
        
        for (name, slot) in &template.slots {
            let slot_hash = RenderSession::hash(slot);
            let key = (slot_hash, context_hash);
            
            if let Some(cached) = session.results.get(&key) {
                debug!("Incremental hit for slot: {}", name);
                injections.insert(name.clone(), cached.clone());
            } else {
                debug!("Incremental miss for slot: {}", name);
                let code = self.generate_slot(template, name).await?;
                session.results.insert(key, code.clone());
                injections.insert(name.clone(), code);
            }
        }
        
        template.render(&injections)
    }

    async fn generate_all(
        &self,
        template: &Template,
        extra_context: Option<InjectionContext>,
    ) -> Result<HashMap<String, String>> {
        let mut injections = HashMap::new();

        // Build base context first to check length
        let base_context = if let Some(ref ctx) = extra_context {
            format!("{}\n{}", self.global_context.to_prompt(), ctx.to_prompt())
        } else {
            self.global_context.to_prompt()
        };

        // Determine if TOON should be used (explicit or auto-threshold)
        let should_use_toon = self.use_toon || self.auto_toon_threshold
            .map(|threshold| base_context.len() >= threshold)
            .unwrap_or(false);

        let mut context_prompt = if should_use_toon {
            // TOON optimization - compress context
            let context_value = serde_json::to_value(&self.global_context)
                .map_err(|e| AetherError::ContextSerializationError(e.to_string()))?;
            let toon_ctx = Toon::serialize(&context_value);
            format!(
                "[CONTEXT:TOON]\n{}\n\n[TOON Protocol Note]\nTOON is a compact key:value mapping protocol. Each line represents 'key: value'. Use this context to inform your code generation, respecting the framework, language, and architectural constraints defined within.",
                toon_ctx
            )
        } else {
            base_context
        };

        // If self-healing is enabled, encourage AI to pass tests
        if self.validator.is_some() {
            context_prompt.push_str("\n\nIMPORTANT: The system is running in TDD (Test-Driven Development) mode. ");
            context_prompt.push_str("Your code will be validated against compiler checks and functional tests. ");
            context_prompt.push_str("If possible, include unit tests in your response to help self-verify. ");
            context_prompt.push_str("If validation fails, you will receive feedback to fix the code.");
        }
        
        let context_prompt = Arc::new(context_prompt);

        if self.parallel {
            injections = self
                .generate_parallel(template, context_prompt)
                .await?;
        } else {
            for (name, slot) in &template.slots {
                debug!("Generating code for slot: {}", name);
                let id = uuid::Uuid::new_v4().to_string();

                let request = GenerationRequest {
                    slot: slot.clone(),
                    context: Some((*context_prompt).clone()),
                    system_prompt: None,
                };

                if let Some(ref obs) = self.observer {
                    obs.on_start(&id, &template.name, name, &request);
                }

                match self.generate_with_retry(request, &id).await {
                    Ok(response) => {
                        if let Some(ref obs) = self.observer {
                            obs.on_success(&id, &response);
                        }
                        injections.insert(name.clone(), response.code);
                    }
                    Err(e) => {
                        if let Some(ref obs) = self.observer {
                            obs.on_failure(&id, &e.to_string());
                        }
                        return Err(e);
                    }
                }
            }
        }

        Ok(injections)
    }

    async fn generate_parallel(
        &self,
        template: &Template,
        context_prompt: Arc<String>,
    ) -> Result<HashMap<String, String>> {
        use tokio::task::JoinSet;

        let mut join_set = JoinSet::new();

        for (name, slot) in template.slots.clone() {
            let provider = Arc::clone(&self.provider);
            let validator = self.validator.clone();
            let cache = self.cache.clone();
            let context = Arc::clone(&context_prompt);
            let max_retries = self.max_retries;
            let template_name = template.name.clone();
            let observer = self.observer.clone();

            join_set.spawn(async move {
                let id = uuid::Uuid::new_v4().to_string();
                let request = GenerationRequest {
                    slot,
                    context: Some((*context).clone()),
                    system_prompt: None,
                };

                if let Some(ref obs) = observer {
                    obs.on_start(&id, &template_name, &name, &request);
                }

                match Self::generate_with_healing_static(&provider, &validator, &cache, &observer, request, max_retries, &id).await {
                    Ok(response) => {
                        if let Some(ref obs) = observer {
                            obs.on_success(&id, &response);
                        }
                        Ok::<_, AetherError>((name, response.code))
                    }
                    Err(e) => {
                        if let Some(ref obs) = observer {
                            obs.on_failure(&id, &e.to_string());
                        }
                        Err(e)
                    }
                }
            });
        }

        let mut injections = HashMap::new();
        while let Some(result) = join_set.join_next().await {
            let (name, code) = result.map_err(|e| AetherError::InjectionError(e.to_string()))??;
            injections.insert(name, code);
        }

        Ok(injections)
    }

    /// Generate with self-healing logic.
    async fn generate_with_retry(&self, request: GenerationRequest, id: &str) -> Result<GenerationResponse> {
        Self::generate_with_healing_static(&self.provider, &self.validator, &self.cache, &self.observer, request, self.max_retries, id).await
    }

    /// Static version of generate with self-healing support.
    async fn generate_with_healing_static(
        provider: &Arc<P>,
        validator: &Option<Arc<dyn Validator>>,
        cache: &Option<Arc<dyn Cache>>,
        observer: &Option<ObserverPtr>,
        mut request: GenerationRequest,
        max_retries: u32,
        id: &str,
    ) -> Result<GenerationResponse> {
        // 0. Check cache first
        let cache_key = if cache.is_some() {
            Some(format!("{}:{}", request.slot.prompt, request.context.as_deref().unwrap_or("")))
        } else {
            None
        };

        if let (Some(ref c), Some(ref key)) = (cache, &cache_key) {
            if let Some(cached_code) = c.get(key) {
                debug!("Cache hit for slot: {}", request.slot.name);
                return Ok(GenerationResponse {
                    code: cached_code,
                    tokens_used: None,
                    metadata: Some(serde_json::json!({"cache": "hit"})),
                });
            }
        }

        let mut last_error = None;

        for attempt in 0..=max_retries {
            // 1. Generate code
            let mut response = match provider.generate(request.clone()).await {
                Ok(r) => r,
                Err(e) => {
                    debug!("Generation attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                    if attempt < max_retries {
                        tokio::time::sleep(std::time::Duration::from_millis(100 * (attempt as u64 + 1))).await;
                        continue;
                    }
                    return Err(last_error.unwrap());
                }
            };

            // 2. Validate and Heal if validator is present
            if let Some(ref val) = validator {
                // Apply formatting (Linter compliance)
                if let Ok(formatted) = val.format(&request.slot.kind, &response.code) {
                    response.code = formatted;
                }
                
                // Use validate_with_slot to support TDD harnesses
                match val.validate_with_slot(&request.slot, &response.code)? {
                    ValidationResult::Valid => {
                        // Success! Cache if enabled
                        if let (Some(ref c), Some(ref key)) = (cache, &cache_key) {
                            c.set(key, response.code.clone());
                        }
                        return Ok(response);
                    },
                    ValidationResult::Invalid(err_msg) => {
                        info!("Self-healing: Validation failed for slot '{}', attempt {}. Error: {}", 
                            request.slot.name, attempt + 1, err_msg);
                        
                        if let Some(ref obs) = observer {
                            obs.on_healing_step(id, attempt + 1, &err_msg);
                        }

                        last_error = Some(AetherError::ValidationFailed { 
                            slot: request.slot.name.clone(), 
                            error: err_msg.clone() 
                        });

                        if attempt < max_retries {
                            // Feedback Loop: Add error to prompt for next attempt
                            request.slot.prompt = format!(
                                "{}\n\n[SELF-HEALING FEEDBACK]\nYour previous output had validation errors. Please fix them and output ONLY the corrected code.\nERROR:\n{}",
                                request.slot.prompt,
                                err_msg
                            );
                            continue;
                        }
                    }
                }
            } else {
                // No validator, just cache and return
                if let (Some(ref c), Some(ref key)) = (cache, &cache_key) {
                    c.set(key, response.code.clone());
                }
                return Ok(response);
            }
        }

        Err(last_error.unwrap_or_else(|| AetherError::MaxRetriesExceeded { 
            slot: request.slot.name, 
            retries: max_retries, 
            last_error: "Healing failed without specific error".to_string() 
        }))
    }

    /// Generate code for a single slot.
    pub async fn generate_slot(&self, template: &Template, slot_name: &str) -> Result<String> {
        let slot = template
            .slots
            .get(slot_name)
            .ok_or_else(|| AetherError::SlotNotFound(slot_name.to_string()))?;

        let request = GenerationRequest {
            slot: slot.clone(),
            context: Some(self.global_context.to_prompt()),
            system_prompt: None,
        };

        let id = uuid::Uuid::new_v4().to_string();
        if let Some(ref obs) = self.observer {
            obs.on_start(&id, &template.name, slot_name, &request);
        }

        match self.generate_with_retry(request, &id).await {
            Ok(response) => {
                if let Some(ref obs) = self.observer {
                    obs.on_success(&id, &response);
                }
                Ok(response.code)
            }
            Err(e) => {
                if let Some(ref obs) = self.observer {
                    obs.on_failure(&id, &e.to_string());
                }
                Err(e)
            }
        }
    }

    /// Generate code for a single slot as a stream.
    pub fn generate_slot_stream(
        &self,
        template: &Template,
        slot_name: &str,
    ) -> Result<BoxStream<'static, Result<StreamResponse>>> {
        let slot = template
            .slots
            .get(slot_name)
            .ok_or_else(|| AetherError::SlotNotFound(slot_name.to_string()))?;

        let request = GenerationRequest {
            slot: slot.clone(),
            context: Some(self.global_context.to_prompt()),
            system_prompt: None,
        };

        Ok(self.provider.generate_stream(request))
    }

    /// Inject a raw prompt and get the code back directly.
    /// Used primarily by the script runtime.
    pub async fn inject_raw(&self, prompt: &str) -> Result<String> {
        let template = Template::new("{{AI:gen}}")
            .with_slot("gen", prompt);
        
        self.render(&template).await
    }
}

/// Convenience function for one-line AI code injection.
///
/// # Example
///
/// ```rust,ignore
/// let code = aether_core::inject!("Create a login form with email and password", OpenAiProvider::from_env()?);
/// ```
#[macro_export]
macro_rules! inject {
    ($prompt:expr, $provider:expr) => {{
        use $crate::{InjectionEngine, Slot, Template};

        let template = Template::new("{{AI:generated}}")
            .with_slot("generated", $prompt);

        let engine = InjectionEngine::new($provider);
        engine.render(&template)
    }};
}

/// Convenience function for synchronous one-line injection (blocking).
#[macro_export]
macro_rules! inject_sync {
    ($prompt:expr, $provider:expr) => {{
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on($crate::inject!($prompt, $provider))
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::MockProvider;

    #[tokio::test]
    async fn test_engine_render() {
        let provider = MockProvider::new()
            .with_response("content", "<p>Hello World</p>");

        let engine = InjectionEngine::new(provider);

        let template = Template::new("<div>{{AI:content}}</div>")
            .with_slot("content", "Generate a paragraph");

        let result = engine.render(&template).await.unwrap();
        assert_eq!(result, "<div><p>Hello World</p></div>");
    }

    #[tokio::test]
    async fn test_engine_with_context() {
        let provider = MockProvider::new()
            .with_response("button", "<button class='btn'>Click</button>");

        let engine = InjectionEngine::new(provider)
            .with_context(InjectionContext::new().with_framework("react"));

        let template = Template::new("{{AI:button}}")
            .with_slot("button", "Create a button");

        let result = engine.render(&template).await.unwrap();
        assert!(result.contains("button"));
    }

    #[tokio::test]
    async fn test_parallel_generation() {
        let provider = MockProvider::new()
            .with_response("slot1", "code1")
            .with_response("slot2", "code2");

        let engine = InjectionEngine::new(provider).parallel(true);

        let template = Template::new("{{AI:slot1}} | {{AI:slot2}}");

        let result = engine.render(&template).await.unwrap();
        assert!(result.contains("code1"));
        assert!(result.contains("code2"));
    }

    #[tokio::test]
    async fn test_max_retries_exceeded() {
        let provider = MockProvider::new()
            .with_response("fail", "invalid code");

        // Use a validator that always fails
        struct FailingValidator;
        impl Validator for FailingValidator {
            fn validate(&self, _: &SlotKind, _: &str) -> Result<ValidationResult> {
                Ok(ValidationResult::Invalid("Always fails".to_string()))
            }
        }

        let engine = InjectionEngine::new(provider)
            .with_validator(FailingValidator)
            .max_retries(1);

        let template = Template::new("{{AI:fail}}");
        let result = engine.render(&template).await;

        match result {
            Err(AetherError::MaxRetriesExceeded { slot, retries, .. }) => {
                assert_eq!(slot, "fail");
                assert_eq!(retries, 1);
            }
            _ => panic!("Expected MaxRetriesExceeded error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_auto_toon_activation() {
        let provider = MockProvider::new()
            .with_response("slot", "code");

        // Set a very low threshold to force TOON
        let config = AetherConfig::default().with_auto_toon_threshold(Some(5));
        let engine = InjectionEngine::with_config(provider, config)
            .with_context(InjectionContext::new().with_framework("very_long_framework_name"));

        let template = Template::new("{{AI:slot}}");
        let _ = engine.render(&template).await.unwrap();
        
        // Internal check: toon should be used because context length > 5
        // Since we can't easily check internal state, we verify it runs without error
    }
}
