#![allow(non_local_definitions)]
use pyo3::prelude::*;
use pyo3::types::PyDict;
use aether_core::{
    AetherRuntime, ProviderConfig, RenderSession as CoreRenderSession,
    cache::SemanticCache,
    validation::RustValidator,
    AetherConfig,
    InjectionContext as CoreContext,
    InjectionEngine,
};
use aether_ai::{OpenAiProvider, AnthropicProvider, GeminiProvider, OllamaProvider};
use std::collections::HashMap;
use rhai::Dynamic;

// ============================================================
// Provider Wrapper (All providers are Clone, so we store them directly)
// ============================================================
#[derive(Clone)]
enum ProviderKind {
    OpenAi(OpenAiProvider),
    Anthropic(AnthropicProvider),
    Gemini(GeminiProvider),
    Ollama(OllamaProvider),
    Grok(OpenAiProvider),  // Grok uses OpenAI-compatible API
}

// ============================================================
// Template Class
// ============================================================
#[pyclass]
struct Template {
    inner: CoreTemplate,
}

#[pymethods]
impl Template {
    #[new]
    fn new(content: String) -> Self {
        Template { inner: CoreTemplate::new(content) }
    }

    fn add_slot(&mut self, key: String, prompt: String, temp: Option<f32>, model: Option<String>, max_tokens: Option<u32>) {
        let mut slot = CoreSlot::new(key.clone(), prompt);
        if let Some(t) = temp {
            slot = slot.with_temperature(t);
        }
        if let Some(m) = model {
            slot = slot.with_model(m);
        }
        if let Some(mt) = max_tokens {
            slot = slot.with_max_tokens(mt);
        }
        self.inner = self.inner.clone().configure_slot(slot);
    }
}

// ============================================================
// RenderSession Class (Incremental Rendering)
// ============================================================
#[pyclass]
struct RenderSession {
    inner: CoreRenderSession,
}

#[pymethods]
impl RenderSession {
    /// Create a new empty render session.
    #[new]
    fn new() -> Self {
        RenderSession { inner: CoreRenderSession::new() }
    }

    /// Get the number of cached slot results.
    fn cached_count(&self) -> usize {
        self.inner.results.len()
    }

    /// Clear all cached results.
    fn clear(&mut self) {
        self.inner.results.clear();
    }
}

// ============================================================
// Engine Class (Upgraded with Healing, Cache, TOON, Shield)
// Note: unsendable because rhai::Engine (used in execute_script) is !Send
// ============================================================
#[pyclass(unsendable)]
struct Engine {
    provider: ProviderKind,
    runtime: tokio::runtime::Runtime,
    config: AetherConfig,
    global_context: Option<CoreContext>,
    api_key_url: Option<String>,
}

#[pymethods]
impl Engine {
    #[new]
    #[pyo3(signature = (provider="openai", api_key=None, model=None))]
    fn new(provider: &str, api_key: Option<String>, model: Option<String>) -> PyResult<Self> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        let provider_kind = match provider.to_lowercase().as_str() {
            "openai" => {
                let key = api_key.or_else(|| std::env::var("OPENAI_API_KEY").ok())
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("OPENAI_API_KEY not set"))?;
                let mod_name = model.or_else(|| std::env::var("OPENAI_MODEL").ok())
                    .unwrap_or_else(|| "gpt-4o".to_string());
                let config = ProviderConfig::new(key, mod_name);
                let p = OpenAiProvider::new(config).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
                ProviderKind::OpenAi(p)
            },
            "anthropic" | "claude" => {
                let key = api_key.or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("ANTHROPIC_API_KEY not set"))?;
                let mod_name = model.or_else(|| std::env::var("ANTHROPIC_MODEL").ok())
                    .unwrap_or_else(|| "claude-3-opus-20240229".to_string());
                let config = ProviderConfig::new(key, mod_name);
                let p = AnthropicProvider::new(config).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
                ProviderKind::Anthropic(p)
            },
            "gemini" => {
                let key = api_key.or_else(|| std::env::var("GOOGLE_API_KEY").ok())
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("GOOGLE_API_KEY not set"))?;
                let mod_name = model.or_else(|| std::env::var("GEMINI_MODEL").ok())
                    .unwrap_or_else(|| "gemini-1.5-pro".to_string());
                let config = ProviderConfig::new(key, mod_name);
                let p = GeminiProvider::new(config).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
                ProviderKind::Gemini(p)
            },
            "ollama" => {
                let mod_name = model.or_else(|| std::env::var("OLLAMA_MODEL").ok())
                    .unwrap_or_else(|| "llama3".to_string());
                let p = OllamaProvider::new(mod_name);
                ProviderKind::Ollama(p)
            },
            "grok" | "xai" => {
                let key = api_key.or_else(|| std::env::var("XAI_API_KEY").ok())
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("XAI_API_KEY not set"))?;
                let mod_name = model.or_else(|| std::env::var("GROK_MODEL").ok())
                    .unwrap_or_else(|| "grok-1".to_string());
                let config = ProviderConfig::new(key, mod_name)
                    .with_base_url("https://api.x.ai/v1/chat/completions");
                let p = OpenAiProvider::new(config).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
                ProviderKind::Grok(p)
            },
            _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Unknown provider: {}", provider))),
        };

        Ok(Engine { 
            provider: provider_kind, 
            runtime: rt,
            config: AetherConfig::default(),
            global_context: None,
            api_key_url: None,
        })
    }

    /// Enable or disable Self-Healing (automatic validation and retry).
    fn set_healing(&mut self, enabled: bool) {
        self.config.healing_enabled = enabled;
    }

    /// Enable or disable Semantic Cache (reduces API costs for similar prompts).
    fn set_cache(&mut self, enabled: bool) {
        self.config.cache_enabled = enabled;
    }

    /// Enable or disable TOON Protocol (token-efficient context injection).
    fn set_toon(&mut self, enabled: bool) {
        self.config.toon_enabled = enabled;
    }

    /// Enable or disable parallel generation.
    fn set_parallel(&mut self, enabled: bool) {
        self.config.parallel = enabled;
    }

    /// Set maximum retries.
    fn set_max_retries(&mut self, retries: u32) {
        self.config.max_retries = retries;
    }

    /// Set the API key URL for remote resolution.
    fn set_api_key_url(&mut self, url: String) {
        self.api_key_url = Some(url);
    }

    /// Set global context for all generations.
    #[pyo3(signature = (project=None, language=None, framework=None))]
    fn set_context(&mut self, project: Option<String>, language: Option<String>, framework: Option<String>) {
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
        self.global_context = Some(ctx);
    }

    /// Render a template using the AI engine.
    fn render(&self, template: &Template) -> PyResult<String> {
        // Clone the provider so we can pass it to InjectionEngine
        let healing = self.healing_enabled;
        let caching = self.cache_enabled;
        let toon = self.toon_enabled;
        let template_inner = template.inner.clone();

        self.runtime.block_on(async {
            // Build a fresh InjectionEngine with the stored flags
            let result = match &self.provider {
                ProviderKind::OpenAi(p) => {
                    let mut p = p.clone();
                    if let Some(ref url) = self.api_key_url {
                        p.config.api_key_url = Some(url.clone());
                    }
                    let mut engine = InjectionEngine::with_config(p, self.config.clone());
                    if let Some(ref ctx) = self.global_context {
                        engine = engine.with_context(ctx.clone());
                    }
                    engine.render(&template_inner).await
                },
                ProviderKind::Anthropic(p) => {
                    let mut engine = InjectionEngine::with_config(p.clone(), self.config.clone());
                    if let Some(ref ctx) = self.global_context {
                        engine = engine.with_context(ctx.clone());
                    }
                    engine.render(&template_inner).await
                },
                ProviderKind::Gemini(p) => {
                    let mut engine = InjectionEngine::with_config(p.clone(), self.config.clone());
                    if let Some(ref ctx) = self.global_context {
                        engine = engine.with_context(ctx.clone());
                    }
                    engine.render(&template_inner).await
                },
                ProviderKind::Ollama(p) => {
                    let mut engine = InjectionEngine::with_config(p.clone(), self.config.clone());
                    if let Some(ref ctx) = self.global_context {
                        engine = engine.with_context(ctx.clone());
                    }
                    engine.render(&template_inner).await
                },
                ProviderKind::Grok(p) => {
                    let mut engine = InjectionEngine::with_config(p.clone(), self.config.clone());
                    if let Some(ref ctx) = self.global_context {
                        engine = engine.with_context(ctx.clone());
                    }
                    if self.config.cache_enabled && engine.cache().is_none() {
                        engine = engine.with_cache(SemanticCache::new().map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?);
                    }
                    engine.render(&template_inner).await
                },
            };

            result.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Render a template incrementally using a session to cache results.
    /// 
    /// Only slots that have changed since the last render will be regenerated.
    /// This is useful for iterative development and reducing API calls.
    /// 
    /// # Arguments
    /// * `template` - The template to render.
    /// * `session` - A RenderSession object that caches results.
    /// 
    /// # Example
    /// ```python
    /// session = RenderSession()
    /// result1 = engine.render_incremental(template, session)  # Full render
    /// result2 = engine.render_incremental(template, session)  # Uses cache
    /// template.add_slot("new_slot", "New prompt")
    /// result3 = engine.render_incremental(template, session)  # Only renders new_slot
    /// ```
    fn render_incremental(&self, template: &Template, session: &mut RenderSession) -> PyResult<String> {
        let template_inner = template.inner.clone();

        self.runtime.block_on(async {
            let result = match &self.provider {
                ProviderKind::OpenAi(p) => {
                    let mut engine = InjectionEngine::with_config(p.clone(), self.config.clone());
                    if let Some(ref ctx) = self.global_context {
                        engine = engine.with_context(ctx.clone());
                    }
                    engine.render_incremental(&template_inner, &mut session.inner).await
                },
                ProviderKind::Anthropic(p) => {
                    let mut engine = InjectionEngine::with_config(p.clone(), self.config.clone());
                    if let Some(ref ctx) = self.global_context {
                        engine = engine.with_context(ctx.clone());
                    }
                    engine.render_incremental(&template_inner, &mut session.inner).await
                },
                ProviderKind::Gemini(p) => {
                    let mut engine = InjectionEngine::with_config(p.clone(), self.config.clone());
                    if let Some(ref ctx) = self.global_context {
                        engine = engine.with_context(ctx.clone());
                    }
                    engine.render_incremental(&template_inner, &mut session.inner).await
                },
                ProviderKind::Ollama(p) => {
                    let mut engine = InjectionEngine::with_config(p.clone(), self.config.clone());
                    if let Some(ref ctx) = self.global_context {
                        engine = engine.with_context(ctx.clone());
                    }
                    engine.render_incremental(&template_inner, &mut session.inner).await
                },
                ProviderKind::Grok(p) => {
                    let mut engine = InjectionEngine::with_config(p.clone(), self.config.clone());
                    if let Some(ref ctx) = self.global_context {
                        engine = engine.with_context(ctx.clone());
                    }
                    engine.render_incremental(&template_inner, &mut session.inner).await
                },
            };

            result.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Deserialize a TOON string back into a JSON structure.
    fn toon_deserialize(&self, toon_str: &str) -> PyResult<String> {
        let val = aether_core::toon::Toon::deserialize(toon_str)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        serde_json::to_string(&val)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    /// Execute a Rhai script directly (Aether Shield core functionality).
    /// 
    /// # Arguments
    /// * `script` - The Rhai script to execute.
    /// * `inputs` - Optional dictionary of input variables.
    /// 
    /// # Returns
    /// The result of the script execution as a string.
    #[pyo3(signature = (script, inputs=None))]
    fn execute_script(&self, script: &str, inputs: Option<&PyDict>) -> PyResult<String> {
        // Create a fresh AetherRuntime for each call (ensures thread safety)
        let rhai_runtime = AetherRuntime::new();
        
        let mut rhai_inputs: HashMap<String, Dynamic> = HashMap::new();

        if let Some(py_dict) = inputs {
            for (key, value) in py_dict.iter() {
                let key_str: String = key.extract()?;
                // Convert Python values to Rhai Dynamic
                if let Ok(v) = value.extract::<i64>() {
                    rhai_inputs.insert(key_str, Dynamic::from(v));
                } else if let Ok(v) = value.extract::<f64>() {
                    rhai_inputs.insert(key_str, Dynamic::from(v));
                } else if let Ok(v) = value.extract::<String>() {
                    rhai_inputs.insert(key_str, Dynamic::from(v));
                } else if let Ok(v) = value.extract::<bool>() {
                    rhai_inputs.insert(key_str, Dynamic::from(v));
                }
            }
        }

        let result = rhai_runtime.execute(script, rhai_inputs)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(result.to_string())
    }

    /// Render a template with streaming output.
    /// 
    /// # Arguments
    /// * `template` - The template to render.
    /// * `slot_name` - The name of the slot to stream (must have exactly one slot).
    /// * `callback` - A Python callable that receives each chunk as a string.
    /// 
    /// # Example
    /// ```python
    /// def on_chunk(chunk):
    ///     print(chunk, end='', flush=True)
    /// 
    /// engine.render_stream(template, "code", on_chunk)
    /// ```
    #[pyo3(signature = (template, slot_name, callback))]
    fn render_stream(
        &self,
        py: Python<'_>,
        template: &Template,
        slot_name: String,
        callback: PyObject,
    ) -> PyResult<String> {
        use futures::StreamExt;
        
        let template_inner = template.inner.clone();

        self.runtime.block_on(async {
            match &self.provider {
                ProviderKind::OpenAi(p) => {
                    let mut engine = InjectionEngine::with_config(p.clone(), self.config.clone());
                    if let Some(ref ctx) = self.global_context {
                        engine = engine.with_context(ctx.clone());
                    }
                    let stream_result = engine.generate_slot_stream(&template_inner, &slot_name);
                    match stream_result {
                        Ok(mut stream) => {
                            let mut full_result = String::new();
                            while let Some(result) = stream.next().await {
                                match result {
                                    Ok(chunk) => {
                                        full_result.push_str(&chunk.delta);
                                        Python::with_gil(|py| {
                                            let _ = callback.call1(py, (chunk.delta.clone(),));
                                        });
                                    }
                                    Err(e) => return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())),
                                }
                            }
                            Ok(full_result)
                        }
                        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
                    }
                },
                ProviderKind::Anthropic(p) => {
                    let mut engine = InjectionEngine::with_config(p.clone(), self.config.clone());
                    if let Some(ref ctx) = self.global_context {
                        engine = engine.with_context(ctx.clone());
                    }
                    let stream_result = engine.generate_slot_stream(&template_inner, &slot_name);
                    match stream_result {
                        Ok(mut stream) => {
                            let mut full_result = String::new();
                            while let Some(result) = stream.next().await {
                                match result {
                                    Ok(chunk) => {
                                        full_result.push_str(&chunk.delta);
                                        Python::with_gil(|py| {
                                            let _ = callback.call1(py, (chunk.delta.clone(),));
                                        });
                                    }
                                    Err(e) => return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())),
                                }
                            }
                            Ok(full_result)
                        }
                        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
                    }
                },
                ProviderKind::Gemini(p) => {
                    let mut engine = InjectionEngine::with_config(p.clone(), self.config.clone());
                    if let Some(ref ctx) = self.global_context {
                        engine = engine.with_context(ctx.clone());
                    }
                    let stream_result = engine.generate_slot_stream(&template_inner, &slot_name);
                    match stream_result {
                        Ok(mut stream) => {
                            let mut full_result = String::new();
                            while let Some(result) = stream.next().await {
                                match result {
                                    Ok(chunk) => {
                                        full_result.push_str(&chunk.delta);
                                        Python::with_gil(|py| {
                                            let _ = callback.call1(py, (chunk.delta.clone(),));
                                        });
                                    }
                                    Err(e) => return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())),
                                }
                            }
                            Ok(full_result)
                        }
                        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
                    }
                },
                ProviderKind::Ollama(p) => {
                    let mut engine = InjectionEngine::with_config(p.clone(), self.config.clone());
                    if let Some(ref ctx) = self.global_context {
                        engine = engine.with_context(ctx.clone());
                    }
                    let stream_result = engine.generate_slot_stream(&template_inner, &slot_name);
                    match stream_result {
                        Ok(mut stream) => {
                            let mut full_result = String::new();
                            while let Some(result) = stream.next().await {
                                match result {
                                    Ok(chunk) => {
                                        full_result.push_str(&chunk.delta);
                                        Python::with_gil(|py| {
                                            let _ = callback.call1(py, (chunk.delta.clone(),));
                                        });
                                    }
                                    Err(e) => return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())),
                                }
                            }
                            Ok(full_result)
                        }
                        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
                    }
                },
                ProviderKind::Grok(p) => {
                    let mut engine = InjectionEngine::with_config(p.clone(), self.config.clone());
                    if let Some(ref ctx) = self.global_context {
                        engine = engine.with_context(ctx.clone());
                    }
                    let stream_result = engine.generate_slot_stream(&template_inner, &slot_name);
                    match stream_result {
                        Ok(mut stream) => {
                            let mut full_result = String::new();
                            while let Some(result) = stream.next().await {
                                match result {
                                    Ok(chunk) => {
                                        full_result.push_str(&chunk.delta);
                                        Python::with_gil(|py| {
                                            let _ = callback.call1(py, (chunk.delta.clone(),));
                                        });
                                    }
                                    Err(e) => return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())),
                                }
                            }
                            Ok(full_result)
                        }
                        Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
                    }
                },
            }
        })
    }
}

// ============================================================
// Module Registration (PyO3 0.20 style)
// ============================================================
#[pymodule]
fn aether(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Engine>()?;
    m.add_class::<Template>()?;
    m.add_class::<RenderSession>()?;
    Ok(())
}
