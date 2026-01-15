//! Injection Engine - The main orchestrator for AI code injection.
//!
//! This module provides the high-level API for rendering templates with AI-generated code.

use crate::{
    AetherError, AiProvider, InjectionContext, Result, Template, SlotKind,
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

// ============================================================
// Internal Types
// ============================================================

/// A simple FNV-1a hasher for stable hashing across runs.
/// This ensures RenderSession cache keys remain stable even if the process restarts.
struct StableHasher(u64);

impl StableHasher {
    fn new() -> Self {
        Self(14695981039346656037)
    }

    fn hash<T: Hash>(t: &T) -> u64 {
        let mut s = Self::new();
        t.hash(&mut s);
        s.finish()
    }
}

impl Hasher for StableHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 ^= byte as u64;
            self.0 = self.0.wrapping_mul(1099511628211);
        }
    }
}

/// Context passed to a generation worker.
struct WorkerContext<P: AiProvider + ?Sized + 'static> {
    provider: Arc<P>,
    validator: Option<Arc<dyn Validator>>,
    cache: Option<Arc<dyn Cache>>,
    observer: Option<ObserverPtr>,
    config: AetherConfig,
}

impl<P: AiProvider + ?Sized + 'static> Clone for WorkerContext<P> {
    fn clone(&self) -> Self {
        Self {
            provider: Arc::clone(&self.provider),
            validator: self.validator.clone(),
            cache: self.cache.clone(),
            observer: self.observer.clone(),
            config: self.config.clone(),
        }
    }
}

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
pub struct InjectionEngine<P: AiProvider + ?Sized> {
    /// The AI provider for code generation.
    provider: Arc<P>,
    
    /// Optional validator for self-healing.
    validator: Option<Arc<dyn Validator>>,

    /// Optional cache for performance/cost optimization.
    cache: Option<Arc<dyn Cache>>,

    /// Engine configuration.
    config: AetherConfig,

    /// Global context applied to all generations.
    global_context: InjectionContext,

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
        StableHasher::hash(t)
    }
}

impl<P: AiProvider + ?Sized + 'static> InjectionEngine<P> {
    /// Create a new injection engine with the given provider and default config.
    pub fn new(provider: P) -> Self where P: Sized {
        Self::with_config(provider, AetherConfig::default())
    }

    /// Internal: Create a raw engine without full config for script-based calls.
    pub fn new_raw(provider: Arc<P>) -> Self {
        Self {
            provider,
            validator: None,
            cache: None,
            config: AetherConfig::default(),
            global_context: InjectionContext::default(),
            observer: None,
        }
    }

    /// Create a new injection engine with the given provider and config.
    pub fn with_config(provider: P, config: AetherConfig) -> Self where P: Sized {
        Self::with_config_arc(Arc::new(provider), config)
    }

    /// Create a new injection engine with the given provider (Arc) and config.
    pub fn with_config_arc(provider: Arc<P>, config: AetherConfig) -> Self {
        let validator: Option<Arc<dyn Validator>> = if config.healing_enabled {
            Some(Arc::new(crate::validation::MultiValidator::new()))
        } else {
            None
        };

        Self {
            provider,
            validator,
            cache: None,
            config,
            global_context: InjectionContext::default(),
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
        self.config.toon_enabled = enabled;
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
        self.config.parallel = enabled;
        self
    }

    /// Set maximum retries for failed generations.
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    /// Get the current cache.
    pub fn cache(&self) -> Option<Arc<dyn Cache>> {
        self.cache.clone()
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
        let should_use_toon = self.config.toon_enabled || self.config.auto_toon_threshold
            .map(|threshold| base_context.len() >= threshold)
            .unwrap_or(false);

        let mut context_prompt = if should_use_toon {
            // TOON optimization - compress context
            let context_value = serde_json::to_value(&self.global_context)
                .map_err(|e| AetherError::ContextSerializationError(e.to_string()))?;
            let toon_ctx = Toon::serialize(&context_value);
            
            if let Some(ref obs) = self.observer {
                let original_size = base_context.len();
                let compressed_size = toon_ctx.len();
                let saved = if original_size > compressed_size { original_size - compressed_size } else { 0 };
                
                obs.on_metadata("global", "toon_compression_metrics", serde_json::json!({
                    "original_chars": original_size,
                    "compressed_chars": compressed_size,
                    "saved_chars": saved,
                    "ratio": (compressed_size as f64 / original_size.max(1) as f64)
                }));
            }

            format!(
                "{}\n{}\n\n{}",
                self.config.prompt_toon_header,
                toon_ctx,
                self.config.prompt_toon_note
            )
        } else {
            base_context
        };

        // If self-healing is enabled, encourage AI to pass tests
        if self.validator.is_some() {
            context_prompt.push_str(&self.config.prompt_tdd_notice);
        }
        
        let context_prompt = Arc::new(context_prompt);

        if self.config.parallel {
            injections = self
                .generate_parallel(template, context_prompt)
                .await?;
        } else {
            for (name, slot) in &template.slots {
                debug!("Generating code for slot: {}", name);
                let id = uuid::Uuid::new_v4().to_string();

                let request = GenerationRequest {
                    max_tokens: slot.max_tokens,
                    model: slot.model.clone(),
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
            let context = Arc::clone(&context_prompt);
            let worker_ctx = WorkerContext {
                provider: Arc::clone(&self.provider),
                validator: self.validator.clone(),
                cache: self.cache.clone(),
                observer: self.observer.clone(),
                config: self.config.clone(),
            };
            let template_name = template.name.clone();

            join_set.spawn(async move {
                let id = uuid::Uuid::new_v4().to_string();
                let request = GenerationRequest {
                    max_tokens: slot.max_tokens,
                    model: slot.model.clone(),
                    slot,
                    context: Some((*context).clone()),
                    system_prompt: None,
                };

                if let Some(ref obs) = worker_ctx.observer {
                    obs.on_start(&id, &template_name, &name, &request);
                }

                match Self::generate_with_healing_static(worker_ctx.clone(), request, &id).await {
                    Ok(response) => {
                        if let Some(ref obs) = worker_ctx.observer {
                            obs.on_success(&id, &response);
                        }
                        Ok::<_, AetherError>((name, response.code))
                    }
                    Err(e) => {
                        if let Some(ref obs) = worker_ctx.observer {
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
        let worker_ctx = WorkerContext {
            provider: Arc::clone(&self.provider),
            validator: self.validator.clone(),
            cache: self.cache.clone(),
            observer: self.observer.clone(),
            config: self.config.clone(),
        };
        Self::generate_with_healing_static(worker_ctx, request, id).await
    }

    /// Static version of generate with self-healing support.
    async fn generate_with_healing_static(
        ctx: WorkerContext<P>,
        mut request: GenerationRequest,
        id: &str,
    ) -> Result<GenerationResponse> {
        // 0. Check cache first
        let cache_key = if ctx.cache.is_some() {
            // Use stable hash for cache key to optimize memory and maintain consistency
            let mut s = StableHasher::new();
            request.slot.prompt.hash(&mut s);
            request.context.as_deref().unwrap_or("").hash(&mut s);
            request.model.as_deref().unwrap_or("").hash(&mut s);
            request.max_tokens.unwrap_or(0).hash(&mut s);
            Some(format!("aether:cache:{:x}", s.finish()))
        } else {
            None
        };

        if let (Some(ref c), Some(ref key)) = (ctx.cache.as_ref(), &cache_key) {
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
        let mut previous_code: Option<String> = None;

        for attempt in 0..=ctx.config.max_retries {
            // 1. Generate code
            let mut response = match ctx.provider.generate(request.clone()).await {
                Ok(r) => r,
                Err(e) => {
                    debug!("Generation attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                    if attempt < ctx.config.max_retries {
                        tokio::time::sleep(std::time::Duration::from_millis(ctx.config.retry_backoff_ms * (attempt as u64 + 1))).await;
                        continue;
                    }
                    return Err(last_error.unwrap());
                }
            };

            // Detect infinite loops (AI generating exact same failing code)
            if let Some(prev) = &previous_code {
                if prev == &response.code {
                    debug!("Self-healing: AI generated identical code for slot '{}', aborting.", request.slot.name);
                    return Err(AetherError::MaxRetriesExceeded {
                        slot: request.slot.name.clone(),
                        retries: attempt,
                        last_error: "AI stuck in loop (generated identical code)".to_string()
                    });
                }
            }
            previous_code = Some(response.code.clone());

            // 2. Validate and Heal if validator is present
            if let Some(ref val) = ctx.validator {
                // Apply formatting (Linter compliance)
                if let Ok(formatted) = val.format(&request.slot.kind, &response.code) {
                    response.code = formatted;
                }
                
                // Use validate_with_slot to support TDD harnesses
                match val.validate_with_slot(&request.slot, &response.code)? {
                    ValidationResult::Valid => {
                        // Success! Cache if enabled
                        if let (Some(ref c), Some(ref key)) = (ctx.cache.as_ref(), &cache_key) {
                            c.set(key, response.code.clone());
                        }
                        return Ok(response);
                    },
                    ValidationResult::Invalid(err_msg) => {
                        info!("Self-healing: Validation failed for slot '{}', attempt {}. Error: {}", 
                            request.slot.name, attempt + 1, err_msg);
                        
                        if let Some(ref obs) = ctx.observer {
                            obs.on_healing_step(id, attempt + 1, &err_msg);
                        }

                        last_error = Some(AetherError::ValidationFailed { 
                            slot: request.slot.name.clone(), 
                            error: err_msg.clone() 
                        });

                        if attempt < ctx.config.max_retries {
                            // Feedback Loop: Add error to prompt for next attempt
                            request.slot.prompt = format!(
                                "{}\n\n{}{}",
                                request.slot.prompt,
                                ctx.config.prompt_healing_feedback,
                                err_msg
                            );
                            continue;
                        }
                    }
                }
            } else {
                // No validator, just cache and return
                if let (Some(ref c), Some(ref key)) = (ctx.cache.as_ref(), &cache_key) {
                    c.set(key, response.code.clone());
                }
                return Ok(response);
            }
        }

        let final_err = AetherError::MaxRetriesExceeded { 
            slot: request.slot.name, 
            retries: ctx.config.max_retries, 
            last_error: last_error.map(|e| e.to_string()).unwrap_or_else(|| "Healing failed without specific error".to_string())
        };
        Err(final_err)
    }

    /// Generate code for a single slot.
    pub async fn generate_slot(&self, template: &Template, slot_name: &str) -> Result<String> {
        let slot = template
            .slots
            .get(slot_name)
            .ok_or_else(|| AetherError::SlotNotFound(slot_name.to_string()))?;

        let request = GenerationRequest {
            max_tokens: slot.max_tokens,
            model: slot.model.clone(),
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
            max_tokens: slot.max_tokens,
            model: slot.model.clone(),
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
            fn format(&self, _: &SlotKind, code: &str) -> Result<String> {
                Ok(code.to_string())
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
