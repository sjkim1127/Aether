#![allow(non_local_definitions)]
use pyo3::prelude::*;
use pyo3::types::PyDict;
use aether_core::{
    InjectionEngine, Template as CoreTemplate, Slot as CoreSlot,
    AetherRuntime, ProviderConfig,
    cache::SemanticCache,
    validation::RustValidator,
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

    fn add_slot(&mut self, key: String, prompt: String, temp: Option<f32>) {
        let mut slot = CoreSlot::new(key.clone(), prompt);
        if let Some(t) = temp {
            slot = slot.with_temperature(t);
        }
        self.inner = self.inner.clone().configure_slot(slot);
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
    // Feature Flags
    healing_enabled: bool,
    cache_enabled: bool,
    toon_enabled: bool,
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
            _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Unknown provider: {}", provider))),
        };

        Ok(Engine { 
            provider: provider_kind, 
            runtime: rt,
            healing_enabled: false,
            cache_enabled: false,
            toon_enabled: false,
        })
    }

    /// Enable or disable Self-Healing (automatic validation and retry).
    fn set_healing(&mut self, enabled: bool) {
        self.healing_enabled = enabled;
    }

    /// Enable or disable Semantic Cache (reduces API costs for similar prompts).
    fn set_cache(&mut self, enabled: bool) {
        self.cache_enabled = enabled;
    }

    /// Enable or disable TOON Protocol (token-efficient context injection).
    fn set_toon(&mut self, enabled: bool) {
        self.toon_enabled = enabled;
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
            macro_rules! build_and_render {
                ($provider:expr) => {{
                    let mut engine = InjectionEngine::new($provider.clone());
                    
                    if healing {
                        engine = engine.with_validator(RustValidator);
                    }
                    if caching {
                        if let Ok(cache) = SemanticCache::new() {
                            engine = engine.with_cache(cache);
                        }
                    }
                    if toon {
                        engine = engine.with_toon(true);
                    }

                    engine.render(&template_inner).await
                }};
            }

            let result = match &self.provider {
                ProviderKind::OpenAi(p) => build_and_render!(p),
                ProviderKind::Anthropic(p) => build_and_render!(p),
                ProviderKind::Gemini(p) => build_and_render!(p),
                ProviderKind::Ollama(p) => build_and_render!(p),
            };

            result.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
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
}

// ============================================================
// Module Registration (PyO3 0.20 style)
// ============================================================
#[pymodule]
fn aether(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Engine>()?;
    m.add_class::<Template>()?;
    Ok(())
}
