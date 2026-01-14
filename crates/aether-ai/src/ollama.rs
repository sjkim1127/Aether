//! Ollama local provider implementation.
//!
//! Supports local LLM models through Ollama.

use aether_core::{
    AetherError, AiProvider, Result,
    provider::{GenerationRequest, GenerationResponse},
    SlotKind,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use aether_core::provider::StreamResponse;
use futures::stream::{BoxStream, StreamExt};
use tracing::{debug, instrument};

const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434/api/generate";

/// Ollama provider for local code generation.
#[derive(Debug, Clone)]
pub struct OllamaProvider {
    client: Client,
    model: String,
    base_url: String,
}

/// Ollama generate request.
#[derive(Debug, Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    system: Option<String>,
    stream: bool,
    options: Option<GenerateOptions>,
}

#[derive(Debug, Serialize)]
struct GenerateOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
}

/// Ollama generate response.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GenerateResponse {
    response: String,
    done: bool,
    #[serde(default)]
    eval_count: Option<u32>,
}

impl OllamaProvider {
    /// Create a new Ollama provider with the given model.
    pub fn new(model: impl Into<String>) -> Self {
        Self::with_options(model, DEFAULT_OLLAMA_URL)
    }

    /// Create a provider with a custom URL.
    pub fn with_options(model: impl Into<String>, base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // Local models can be slow
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            model: model.into(),
            base_url: base_url.into(),
        }
    }

    /// Create from environment variables.
    ///
    /// Reads `OLLAMA_MODEL` and optionally `OLLAMA_URL`.
    pub fn from_env() -> Self {
        let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "codellama".to_string());
        let url = std::env::var("OLLAMA_URL").unwrap_or_else(|_| DEFAULT_OLLAMA_URL.to_string());
        Self::with_options(model, url)
    }

    /// Build the system prompt for code generation.
    fn build_system_prompt(&self, kind: &SlotKind, context: Option<&str>) -> String {
        let base = "You are a code generation assistant. Generate only the requested code without explanations or markdown code blocks. Output raw code only.";

        let kind_specific = match kind {
            SlotKind::Html => "\nGenerate valid HTML5 markup.",
            SlotKind::Css => "\nGenerate valid CSS styles.",
            SlotKind::JavaScript => "\nGenerate valid JavaScript code.",
            SlotKind::Function => "\nGenerate a complete function definition.",
            SlotKind::Class => "\nGenerate a complete class/struct definition.",
            SlotKind::Component => "\nGenerate a complete component with HTML, CSS, and JavaScript as needed.",
            _ => "",
        };

        let context_part = context
            .filter(|c| !c.is_empty())
            .map(|c| format!("\n\nContext:\n{}", c))
            .unwrap_or_default();

        format!("{}{}{}", base, kind_specific, context_part)
    }
}

#[async_trait]
impl AiProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    #[instrument(skip(self, request), fields(slot = %request.slot.name))]
    async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse> {
        debug!("Generating code with Ollama for slot: {}", request.slot.name);

        let system = Some(request.system_prompt.unwrap_or_else(|| {
            self.build_system_prompt(&request.slot.kind, request.context.as_deref())
        }));

        let api_request = GenerateRequest {
            model: self.model.clone(),
            prompt: request.slot.prompt.clone(),
            system,
            stream: false,
            options: Some(GenerateOptions {
                temperature: Some(0.7),
                num_predict: Some(2048),
            }),
        };

        let response = self
            .client
            .post(&self.base_url)
            .json(&api_request)
            .send()
            .await
            .map_err(|e| AetherError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AetherError::ProviderError(format!(
                "Ollama error {}: {}",
                status, body
            )));
        }

        let gen_response: GenerateResponse = response
            .json()
            .await
            .map_err(|e| AetherError::ProviderError(e.to_string()))?;

        let code = strip_code_blocks(&gen_response.response);

        Ok(GenerationResponse {
            code,
            tokens_used: gen_response.eval_count,
            metadata: None,
        })
    }

    fn generate_stream(
        &self,
        request: GenerationRequest,
    ) -> BoxStream<'static, Result<StreamResponse>> {
        let client = self.client.clone();
        let model = self.model.clone();
        let base_url = self.base_url.clone();

        let system = Some(request.system_prompt.unwrap_or_else(|| {
            self.build_system_prompt(&request.slot.kind, request.context.as_deref())
        }));

        let api_request = GenerateRequest {
            model: model.clone(),
            prompt: request.slot.prompt.clone(),
            system,
            stream: true,
            options: Some(GenerateOptions {
                temperature: Some(0.7),
                num_predict: Some(2048),
            }),
        };

        let stream = async_stream::stream! {
            let response = client
                .post(&base_url)
                .json(&api_request)
                .send()
                .await
                .map_err(|e| aether_core::AetherError::NetworkError(e.to_string()));

            let response = match response {
                Ok(r) => r,
                Err(e) => {
                    yield Err(e);
                    return;
                }
            };

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                yield Err(aether_core::AetherError::ProviderError(format!(
                    "Ollama error {}: {}",
                    status, body
                )));
                return;
            }

            let mut stream = response.bytes_stream();
            
            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        yield Err(aether_core::AetherError::NetworkError(e.to_string()));
                        break;
                    }
                };

                let text = String::from_utf8_lossy(&chunk);
                for line in text.lines() {
                    let line = line.trim();
                    if line.is_empty() { continue; }
                    
                    if let Ok(gen_resp) = serde_json::from_str::<GenerateResponse>(line) {
                        yield Ok(StreamResponse {
                            delta: gen_resp.response,
                            metadata: None,
                        });
                        if gen_resp.done { break; }
                    }
                }
            }
        };

        Box::pin(stream)
    }

    async fn health_check(&self) -> Result<bool> {
        // Check if Ollama is running
        let response = self
            .client
            .get("http://localhost:11434/api/tags")
            .send()
            .await
            .map_err(|e| AetherError::NetworkError(e.to_string()))?;

        Ok(response.status().is_success())
    }
}

/// Strip markdown code blocks from generated code.
fn strip_code_blocks(code: &str) -> String {
    let code = code.trim();

    if code.starts_with("```") && code.ends_with("```") {
        let lines: Vec<&str> = code.lines().collect();
        if lines.len() >= 2 {
            return lines[1..lines.len() - 1].join("\n");
        }
    }

    code.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = OllamaProvider::new("codellama");
        assert_eq!(provider.model, "codellama");
    }
}
