//! OpenAI provider implementation.
//!
//! Supports GPT-4, GPT-3.5-turbo, and other OpenAI models.

use aether_core::{
    AetherError, AiProvider, ProviderConfig, Result,
    provider::{GenerationRequest, GenerationResponse},
    SlotKind,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";

/// OpenAI provider for code generation.
#[derive(Debug, Clone)]
pub struct OpenAiProvider {
    client: Client,
    config: ProviderConfig,
}

/// OpenAI chat completion request.
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

/// Chat message.
#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// OpenAI chat completion response.
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct Usage {
    total_tokens: u32,
}

/// OpenAI streaming response chunk.
#[derive(Debug, Deserialize)]
struct ChatStreamResponse {
    choices: Vec<ChatStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatStreamChoice {
    delta: ChatStreamDelta,
    #[allow(dead_code)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatStreamDelta {
    content: Option<String>,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider with the given configuration.
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
    /// Reads `OPENAI_API_KEY` and optionally `OPENAI_MODEL`.
    pub fn from_env() -> Result<Self> {
        let config = ProviderConfig::from_env()?;
        Self::new(config)
    }

    /// Create a provider from environment with a specific model.
    pub fn from_env_with_model(model: &str) -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| AetherError::ConfigError("OPENAI_API_KEY not set".to_string()))?;

        let config = ProviderConfig::new(api_key, model);
        Self::new(config)
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

use aether_core::provider::StreamResponse;
use futures::stream::{BoxStream, StreamExt};

#[async_trait]
impl AiProvider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    #[instrument(skip(self, request), fields(slot = %request.slot.name))]
    async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse> {
        debug!("Generating code with OpenAI for slot: {}", request.slot.name);

        let system_prompt = request.system_prompt.unwrap_or_else(|| {
            self.build_system_prompt(&request.slot.kind, request.context.as_deref())
        });

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: request.slot.prompt.clone(),
            },
        ];

        let api_request = ChatRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            stream: None,
        };

        let url = self.config.base_url.as_deref().unwrap_or(OPENAI_API_URL);

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
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

        let chat_response: ChatResponse = response
            .json()
            .await
            .map_err(|e| AetherError::ProviderError(e.to_string()))?;

        let code = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        // Strip markdown code blocks if present
        let code = strip_code_blocks(&code);

        // Validate against slot constraints
        if let Err(errors) = request.slot.validate(&code) {
            debug!("Generated code failed validation: {:?}", errors);
            // For now, we'll still return the code but log the warning
        }

        Ok(GenerationResponse {
            code,
            tokens_used: chat_response.usage.map(|u| u.total_tokens),
            metadata: None,
        })
    }

    fn generate_stream(
        &self,
        request: GenerationRequest,
    ) -> BoxStream<'static, Result<StreamResponse>> {
        let client = self.client.clone();
        let config = self.config.clone();
        let system_prompt = request.system_prompt.unwrap_or_else(|| {
            self.build_system_prompt(&request.slot.kind, request.context.as_deref())
        });
        let user_prompt = request.slot.prompt.clone();
        let url = config.base_url.as_deref().unwrap_or(OPENAI_API_URL).to_string();

        let api_request = ChatRequest {
            model: config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt,
                },
            ],
            max_tokens: config.max_tokens,
            temperature: config.temperature,
            stream: Some(true),
        };

        let stream = async_stream::stream! {
            let response = client
                .post(&url)
                .header("Authorization", format!("Bearer {}", config.api_key))
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

                // OpenAI stream format is SSE: "data: {...}"
                let text = String::from_utf8_lossy(&chunk);
                for line in text.lines() {
                    let line = line.trim();
                    if line.is_empty() { continue; }
                    if line == "data: [DONE]" { break; }
                    
                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(stream_resp) = serde_json::from_str::<ChatStreamResponse>(data) {
                            if let Some(choice) = stream_resp.choices.first() {
                                if let Some(content) = &choice.delta.content {
                                    yield Ok(StreamResponse {
                                        delta: content.clone(),
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
        // Try a minimal API call
        let response = self
            .client
            .get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .map_err(|e| AetherError::NetworkError(e.to_string()))?;

        Ok(response.status().is_success())
    }
}

/// Strip markdown code blocks from generated code.
fn strip_code_blocks(code: &str) -> String {
    let code = code.trim();

    // Check for ```language\n...\n``` pattern
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
    fn test_strip_code_blocks() {
        let input = "```html\n<div>Hello</div>\n```";
        assert_eq!(strip_code_blocks(input), "<div>Hello</div>");

        let input = "<div>Already clean</div>";
        assert_eq!(strip_code_blocks(input), "<div>Already clean</div>");
    }

    #[test]
    fn test_system_prompt_generation() {
        let config = ProviderConfig::new("test-key", "gpt-4");
        let provider = OpenAiProvider::new(config).unwrap();

        let prompt = provider.build_system_prompt(&SlotKind::Html, None);
        assert!(prompt.contains("HTML5"));
    }
}
