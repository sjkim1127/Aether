use pyo3::prelude::*;
use aether_core::{InjectionEngine, Template as CoreTemplate, Slot as CoreSlot};
use aether_ai::{OpenAiProvider, AnthropicProvider, GeminiProvider, OllamaProvider};

// Internal Enum to hold the concrete engine type
enum EngineBackend {
    OpenAi(InjectionEngine<OpenAiProvider>),
    Anthropic(InjectionEngine<AnthropicProvider>),
    Gemini(InjectionEngine<GeminiProvider>),
    Ollama(InjectionEngine<OllamaProvider>),
}

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

#[pyclass]
struct Engine {
    backend: EngineBackend,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl Engine {
    #[new]
    #[pyo3(signature = (provider="openai", api_key=None, model=None))]
    fn new(provider: &str, api_key: Option<String>, model: Option<String>) -> PyResult<Self> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        // Helper logic to resolve params: Arg -> Env -> Default
        let backend = match provider.to_lowercase().as_str() {
            "openai" => {
                let key = api_key.or_else(|| std::env::var("OPENAI_API_KEY").ok())
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("OPENAI_API_KEY not set"))?;
                
                let mod_name = model.or_else(|| std::env::var("OPENAI_MODEL").ok())
                    .unwrap_or_else(|| "gpt-3.5-turbo".to_string());
                
                let config = aether_core::ProviderConfig::new(key, mod_name);
                let p = OpenAiProvider::new(config).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
                
                EngineBackend::OpenAi(InjectionEngine::new(p))
            },
            "anthropic" | "claude" => {
                let key = api_key.or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("ANTHROPIC_API_KEY not set"))?;
                
                let mod_name = model.or_else(|| std::env::var("ANTHROPIC_MODEL").ok())
                    .unwrap_or_else(|| "claude-3-opus-20240229".to_string());
                
                let config = aether_core::ProviderConfig::new(key, mod_name);
                let p = AnthropicProvider::new(config).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
                
                EngineBackend::Anthropic(InjectionEngine::new(p))
            },
            "gemini" => {
                let key = api_key.or_else(|| std::env::var("GOOGLE_API_KEY").ok())
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("GOOGLE_API_KEY not set"))?;
                
                let mod_name = model.or_else(|| std::env::var("GEMINI_MODEL").ok())
                    .unwrap_or_else(|| "gemini-1.5-pro".to_string());
                
                let config = aether_core::ProviderConfig::new(key, mod_name);
                let p = GeminiProvider::new(config).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
                
                EngineBackend::Gemini(InjectionEngine::new(p))
            },
            "ollama" => {
                let mod_name = model.or_else(|| std::env::var("OLLAMA_MODEL").ok())
                    .unwrap_or_else(|| "llama3".to_string());
                let p = OllamaProvider::new(mod_name);
                EngineBackend::Ollama(InjectionEngine::new(p))
            },
            _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Unknown provider: {}", provider))),
        };

        Ok(Engine { backend, runtime: rt })
    }

    fn render(&self, template: &Template) -> PyResult<String> {
        self.runtime.block_on(async {
            match &self.backend {
                EngineBackend::OpenAi(e) => e.render(&template.inner).await,
                EngineBackend::Anthropic(e) => e.render(&template.inner).await,
                EngineBackend::Gemini(e) => e.render(&template.inner).await,
                EngineBackend::Ollama(e) => e.render(&template.inner).await,
            }.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }
}

#[pymodule]
fn aether(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Engine>()?;
    m.add_class::<Template>()?;
    Ok(())
}
