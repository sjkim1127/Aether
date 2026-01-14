//! Anthropic Claude provider implementation.

use aether_core::{
    AetherError, AiProvider, ProviderConfig, Result,
    provider::{GenerationRequest, GenerationResponse},
    SlotKind,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Anthropic Claude provider for code generation.
#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    client: Client,
    config: ProviderConfig,
}

/// Anthropic message request.
#[derive(Debug, Serialize)]
struct MessageRequest {
    model: String,
    max_tokens: u32,
    system: Option<String>,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

/// Anthropic streaming response event (minimal)
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum StreamEvent {
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        delta: TextDelta,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
struct TextDelta {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// Anthropic message response.
#[derive(Debug, Deserialize)]
struct MessageResponse {
    content: Vec<ContentBlock>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider.
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
    /// Reads `ANTHROPIC_API_KEY`.
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| AetherError::ConfigError("ANTHROPIC_API_KEY not set".to_string()))?;

        let model = std::env::var("ANTHROPIC_MODEL")
            .unwrap_or_else(|_| "claude-opus-4-5".to_string());

        let config = ProviderConfig::new(api_key, model);
        Self::new(config)
    }

    /// Create a provider from environment with a specific model.
    pub fn from_env_with_model(model: &str) -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| AetherError::ConfigError("ANTHROPIC_API_KEY not set".to_string()))?;

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
impl AiProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    #[instrument(skip(self, request), fields(slot = %request.slot.name))]
    async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse> {
        debug!("Generating code with Anthropic for slot: {}", request.slot.name);

        let system = Some(request.system_prompt.unwrap_or_else(|| {
            self.build_system_prompt(&request.slot.kind, request.context.as_deref())
        }));

        let messages = vec![Message {
            role: "user".to_string(),
            content: request.slot.prompt.clone(),
        }];

        let temperature = request.slot.temperature.or(self.config.temperature);
        let api_request = MessageRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(4096),
            system,
            messages,
            temperature,
            stream: None,
        };

        let url = self.config.base_url.as_deref().unwrap_or(ANTHROPIC_API_URL);

        let response = self
            .client
            .post(url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
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

        let msg_response: MessageResponse = response
            .json()
            .await
            .map_err(|e| AetherError::ProviderError(e.to_string()))?;

        let code = msg_response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        // Strip markdown code blocks if present
        let code = strip_code_blocks(&code);

        Ok(GenerationResponse {
            code,
            tokens_used: Some(msg_response.usage.input_tokens + msg_response.usage.output_tokens),
            metadata: None,
        })
    }

    fn generate_stream(
        &self,
        request: GenerationRequest,
    ) -> BoxStream<'static, Result<StreamResponse>> {
        let client = self.client.clone();
        let config = self.config.clone();
        let system = Some(request.system_prompt.unwrap_or_else(|| {
            self.build_system_prompt(&request.slot.kind, request.context.as_deref())
        }));
        let user_prompt = request.slot.prompt.clone();
        let url = config.base_url.as_deref().unwrap_or(ANTHROPIC_API_URL).to_string();

        let temperature = request.slot.temperature.or(config.temperature);
        let api_request = MessageRequest {
            model: config.model.clone(),
            max_tokens: config.max_tokens.unwrap_or(4096),
            system,
            messages: vec![Message {
                role: "user".to_string(),
                content: user_prompt,
            }],
            temperature,
            stream: Some(true),
        };

        let stream = async_stream::stream! {
            let response = client
                .post(&url)
                .header("x-api-key", &config.api_key)
                .header("anthropic-version", ANTHROPIC_VERSION)
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
                        if let Ok(event) = serde_json::from_str::<StreamEvent>(event_data) {
                            if let StreamEvent::ContentBlockDelta { delta } = event {
                                yield Ok(StreamResponse {
                                    delta: delta.text,
                                    metadata: None,
                                });
                            }
                        }
                    }
                }
            }
        };

        Box::pin(stream)
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
    fn test_system_prompt() {
        let config = ProviderConfig::new("test-key", "claude-3-sonnet-20240229");
        let provider = AnthropicProvider::new(config).unwrap();

        let prompt = provider.build_system_prompt(&SlotKind::Html, None);
        assert!(prompt.contains("HTML5"));
    }
}
