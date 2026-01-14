//! Google Gemini provider implementation.
//!
//! Supports Gemini Pro and other Google AI models.

use aether_core::{
    AetherError, AiProvider, ProviderConfig, Result,
    provider::{GenerationRequest, GenerationResponse},
    SlotKind,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};
use aether_core::provider::StreamResponse;
use futures::stream::{BoxStream, StreamExt};

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";

/// Google Gemini provider for code generation.
#[derive(Debug, Clone)]
pub struct GeminiProvider {
    client: Client,
    config: ProviderConfig,
}

// Request structures
#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Debug, Serialize)]
struct Content {
    parts: Vec<Part>,
    role: String,
}

#[derive(Debug, Serialize)]
struct Part {
    text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
}

// Response structures
#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
    usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: ContentResponse,
}

#[derive(Debug, Deserialize)]
struct ContentResponse {
    parts: Vec<PartResponse>,
}

#[derive(Debug, Deserialize)]
struct PartResponse {
    text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UsageMetadata {
    total_token_count: u32,
}

impl GeminiProvider {
    /// Create a new Gemini provider with the given configuration.
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let timeout = config.timeout_seconds.unwrap_or(60);
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout))
            .build()
            .map_err(|e| AetherError::NetworkError(e.to_string()))?;

        Ok(Self { client, config })
    }

    /// Create a provider from environment variables.
    ///
    /// Reads `GOOGLE_API_KEY` and optionally `GEMINI_MODEL`.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("GOOGLE_API_KEY")
            .map_err(|_| AetherError::ConfigError("GOOGLE_API_KEY not set".to_string()))?;

        let model = std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-1.5-pro".to_string());
        
        // Google API key is query param, not header like OpenAI
        // We store it in config.api_key but will use it in URL
        let config = ProviderConfig::new(api_key, model);
        Self::new(config)
    }

    /// Build the specific prompt for Gemini
    fn build_prompt(&self, kind: &SlotKind, context: Option<&str>, user_prompt: &str) -> String {
        let base_instructions = match kind {
            SlotKind::Html => "Generate valid HTML5 markup.",
            SlotKind::Css => "Generate valid CSS styles.",
            SlotKind::JavaScript => "Generate valid JavaScript code.",
            SlotKind::Function => "Generate a complete function definition.",
            SlotKind::Class => "Generate a complete class/struct definition.",
            SlotKind::Component => "Generate a complete component with HTML, CSS, and JavaScript as needed.",
            _ => "Generate code based on the request.",
        };

        let context_str = context
            .map(|c| format!("\nContext:\n{}", c))
            .unwrap_or_default();

        format!(
            "Role: Code Generator. Task: {}\n{}\nRequest: {}\nOutput only raw code, no markdown.",
            base_instructions, context_str, user_prompt
        )
    }
}

#[async_trait]
impl AiProvider for GeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    #[instrument(skip(self, request), fields(slot = %request.slot.name))]
    async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse> {
        debug!("Generating code with Gemini for slot: {}", request.slot.name);

        // Gemini API is slightly different (no system role in v1beta easily)
        // so we verify robust prompt engineering in the user message
        let full_prompt = self.build_prompt(&request.slot.kind, request.context.as_deref(), &request.slot.prompt);

        let contents = vec![Content {
            role: "user".to_string(),
            parts: vec![Part { text: full_prompt }],
        }];

        let temperature = request.slot.temperature.or(self.config.temperature);
        let api_request = GeminiRequest {
            contents,
            generation_config: Some(GenerationConfig {
                temperature,
                max_output_tokens: self.config.max_tokens,
            }),
        };

        let url = format!(
            "{}/{}:generateContent?key={}",
            GEMINI_API_BASE, self.config.model, self.config.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&api_request)
            .send()
            .await
            .map_err(|e| AetherError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AetherError::ProviderError(format!(
                "API error {}: {}",
                status, body
            )));
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| AetherError::ProviderError(e.to_string()))?;

        // Extract text from the first candidate
        let code = gemini_response
            .candidates
            .as_ref()
            .and_then(|c| c.first())
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| AetherError::ProviderError("No content generated".to_string()))?;

        // Clean up markdown
        let code = code.trim().trim_start_matches("```").trim_end_matches("```");
        // Sometimes it includes the language name like ```rust ... ```
        let code = if let Some(newline_idx) = code.find('\n') {
            if code[..newline_idx].chars().all(char::is_alphanumeric) {
                &code[newline_idx + 1..]
            } else {
                code
            }
        } else {
            code
        };

        Ok(GenerationResponse {
            code: code.to_string(),
            tokens_used: gemini_response.usage_metadata.map(|u| u.total_token_count),
            metadata: None,
        })
    }

    fn generate_stream(
        &self,
        request: GenerationRequest,
    ) -> BoxStream<'static, Result<StreamResponse>> {
        let client = self.client.clone();
        let config = self.config.clone();
        let full_prompt = self.build_prompt(&request.slot.kind, request.context.as_deref(), &request.slot.prompt);
        
        let temperature = request.slot.temperature.or(config.temperature);
        let api_request = GeminiRequest {
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![Part { text: full_prompt }],
            }],
            generation_config: Some(GenerationConfig {
                temperature,
                max_output_tokens: config.max_tokens,
            }),
        };

        let url = format!(
            "{}/{}:streamGenerateContent?alt=sse&key={}",
            GEMINI_API_BASE, config.model, config.api_key
        );

        let stream = async_stream::stream! {
            let response = client
                .post(&url)
                .header("Content-Type", "application/json")
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
                    "API error {}: {}",
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
                    
                    if let Some(event_data) = line.strip_prefix("data: ") {
                        if let Ok(gemini_resp) = serde_json::from_str::<GeminiResponse>(event_data) {
                            if let Some(candidate) = gemini_resp.candidates.as_ref().and_then(|c| c.first()) {
                                if let Some(part) = candidate.content.parts.first() {
                                    yield Ok(StreamResponse {
                                        delta: part.text.clone(),
                                        metadata: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        };

        Box::pin(stream)
    }

    async fn health_check(&self) -> Result<bool> {
        // Minimal check - try to get model info
         let url = format!(
            "{}/{}?key={}",
            GEMINI_API_BASE, self.config.model, self.config.api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AetherError::NetworkError(e.to_string()))?;

        Ok(response.status().is_success())
    }
}
