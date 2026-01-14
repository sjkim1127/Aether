//! Injection Engine - The main orchestrator for AI code injection.
//!
//! This module provides the high-level API for rendering templates with AI-generated code.

use crate::{
    AetherError, AiProvider, InjectionContext, Result, Template,
    provider::{GenerationRequest, GenerationResponse},
};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument};
use futures::stream::BoxStream;
use crate::provider::StreamResponse;

/// The main engine for AI code injection.
///
/// # Example
///
/// ```rust,ignore
/// use aether_core::{InjectionEngine, Template};
/// use aether_ai::OpenAiProvider;
///
/// let provider = OpenAiProvider::from_env()?;
/// let engine = InjectionEngine::new(provider);
///
/// let template = Template::new("<div>{{AI:content}}</div>")
///     .with_slot("content", "Generate a welcome message");
///
/// let result = engine.render(&template).await?;
/// ```
pub struct InjectionEngine<P: AiProvider> {
    /// The AI provider for code generation.
    provider: Arc<P>,

    /// Global context applied to all generations.
    global_context: InjectionContext,

    /// Whether to run generations in parallel.
    parallel: bool,

    /// Maximum retries for failed generations.
    max_retries: u32,
}

impl<P: AiProvider + 'static> InjectionEngine<P> {
    /// Create a new injection engine with the given provider.
    pub fn new(provider: P) -> Self {
        Self {
            provider: Arc::new(provider),
            global_context: InjectionContext::default(),
            parallel: true,
            max_retries: 2,
        }
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

    /// Generate code for all slots in a template.
    async fn generate_all(
        &self,
        template: &Template,
        extra_context: Option<InjectionContext>,
    ) -> Result<HashMap<String, String>> {
        let mut injections = HashMap::new();

        let context_prompt = if let Some(ref ctx) = extra_context {
            format!("{}\n{}", self.global_context.to_prompt(), ctx.to_prompt())
        } else {
            self.global_context.to_prompt()
        };

        if self.parallel {
            injections = self
                .generate_parallel(template, &context_prompt)
                .await?;
        } else {
            for (name, slot) in &template.slots {
                debug!("Generating code for slot: {}", name);

                let request = GenerationRequest {
                    slot: slot.clone(),
                    context: Some(context_prompt.clone()),
                    system_prompt: None,
                };

                let response = self.generate_with_retry(request).await?;
                injections.insert(name.clone(), response.code);
            }
        }

        Ok(injections)
    }

    /// Generate code for all slots in parallel.
    async fn generate_parallel(
        &self,
        template: &Template,
        context_prompt: &str,
    ) -> Result<HashMap<String, String>> {
        use tokio::task::JoinSet;

        let mut join_set = JoinSet::new();

        for (name, slot) in template.slots.clone() {
            let provider = Arc::clone(&self.provider);
            let context = context_prompt.to_string();
            let max_retries = self.max_retries;

            join_set.spawn(async move {
                let request = GenerationRequest {
                    slot,
                    context: Some(context),
                    system_prompt: None,
                };

                let response = Self::generate_with_retry_static(&provider, request, max_retries).await?;
                Ok::<_, AetherError>((name, response.code))
            });
        }

        let mut injections = HashMap::new();
        while let Some(result) = join_set.join_next().await {
            let (name, code) = result.map_err(|e| AetherError::InjectionError(e.to_string()))??;
            injections.insert(name, code);
        }

        Ok(injections)
    }

    /// Generate with retry logic.
    async fn generate_with_retry(&self, request: GenerationRequest) -> Result<GenerationResponse> {
        Self::generate_with_retry_static(&self.provider, request, self.max_retries).await
    }

    /// Static version of generate_with_retry for use in spawned tasks.
    async fn generate_with_retry_static(
        provider: &Arc<P>,
        request: GenerationRequest,
        max_retries: u32,
    ) -> Result<GenerationResponse> {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match provider.generate(request.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    debug!("Generation attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                    if attempt < max_retries {
                        tokio::time::sleep(std::time::Duration::from_millis(100 * (attempt as u64 + 1))).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| AetherError::ProviderError("Unknown error".to_string())))
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

        let response = self.generate_with_retry(request).await?;
        Ok(response.code)
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
}
