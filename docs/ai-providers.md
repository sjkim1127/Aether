# AI Providers

Aether supports multiple AI providers for code generation.

## OpenAI

The default provider with access to GPT-5.2 models.

### Available Models

| Model | ID | Best For |
|-------|-----|----------|
| GPT-5.2 Instant | `gpt-5.2-instant` | Fast responses, simple tasks |
| GPT-5.2 Thinking | `gpt-5.2-thinking` | Complex coding, logic (default) |
| GPT-5.2 Pro | `gpt-5.2-pro` | Highest quality, hard problems |

### Configuration

**Rust:**
```rust
use aether_ai::OpenAiProvider;
use aether_core::ProviderConfig;

// From environment (OPENAI_API_KEY)
let provider = OpenAiProvider::from_env()?;

// With specific model
let provider = aether_ai::openai("gpt-5.2-pro")?;

// With full configuration
let config = ProviderConfig::new("sk-...", "gpt-5.2-thinking")
    .with_temperature(0.7)
    .with_max_tokens(4096)
    .with_timeout(120);
let provider = OpenAiProvider::new(config)?;
```

**Node.js:**
```javascript
const { AetherEngine } = require('aether-codegen');

const engine = AetherEngine.openai("gpt-5.2-thinking");
engine.setApiKey("sk-...");
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `OPENAI_API_KEY` | Your OpenAI API key |
| `AETHER_MODEL` | Default model (overrides gpt-5.2-thinking) |

---

## Anthropic

Access to Claude models with strong coding capabilities.

### Available Models

| Model | ID | Best For |
|-------|-----|----------|
| Claude Opus 4.5 | `claude-opus-4-5` | Best quality (default) |
| Claude Sonnet 4 | `claude-sonnet-4` | Fast, good for coding |

### Configuration

**Rust:**
```rust
use aether_ai::AnthropicProvider;

// From environment (ANTHROPIC_API_KEY)
let provider = AnthropicProvider::from_env()?;

// With specific model
let provider = aether_ai::anthropic("claude-sonnet-4")?;
```

**Node.js:**
```javascript
const { AetherEngine } = require('aether-codegen');

const engine = AetherEngine.anthropic("claude-opus-4-5");
engine.setApiKey("sk-ant-...");
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `ANTHROPIC_API_KEY` | Your Anthropic API key |
| `ANTHROPIC_MODEL` | Default model |

---

## Ollama (Local)

Run AI models locally without an API key.

### Prerequisites

1. Install Ollama: https://ollama.ai
2. Pull a code model:
   ```bash
   ollama pull codellama
   ollama pull deepseek-coder
   ```
3. Start Ollama:
   ```bash
   ollama serve
   ```

### Available Models

| Model | Best For |
|-------|----------|
| `codellama` | General coding (default) |
| `codellama:13b` | Better quality |
| `codellama:34b` | Best local quality |
| `deepseek-coder` | Alternative coder |

### Configuration

**Rust:**
```rust
use aether_ai::OllamaProvider;

// Default (localhost:11434, codellama)
let provider = OllamaProvider::new("codellama");

// Custom URL
let provider = OllamaProvider::with_options(
    "codellama:13b",
    "http://192.168.1.100:11434/api/generate"
);
```

**Node.js:**
```javascript
const { AetherEngine } = require('aether-codegen');

const engine = AetherEngine.ollama("codellama");
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `OLLAMA_MODEL` | Default model |
| `OLLAMA_URL` | API URL (default: http://localhost:11434/api/generate) |

---

## Google Gemini

Access to Google's powerful Gemini models with large context windows.

### Available Models

| Model | ID | Best For |
|-------|----|----------|
| Gemini 1.5 Pro | `gemini-1.5-pro` | Best quality, huge context (default) |
| Gemini 1.5 Flash | `gemini-1.5-flash` | Fast, cost-effective |

### Configuration

**Rust:**
```rust
use aether_ai::GeminiProvider;

// From environment (GOOGLE_API_KEY)
let provider = GeminiProvider::from_env()?;

// With specific model
let provider = aether_ai::gemini("gemini-1.5-flash")?;
```

**Node.js:**
```javascript
const { AetherEngine } = require('aether-codegen');

// Not yet supported in Node.js binding (coming soon)
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `GOOGLE_API_KEY` | Your Google API key |
| `GEMINI_MODEL` | Default model |

---

## Grok (xAI)

Access to xAI's Grok models via their OpenAI-compatible API.

### Available Models

| Model | Besf For |
|-------|----------|
| `grok-1` | High performance reasoning |
| `grok-beta` | Latest features |

### Configuration

**Rust:**
```rust
// Uses OpenAiProvider internally
let provider = aether_ai::grok("grok-1")?;
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `XAI_API_KEY` | Your xAI API key |

---

## Comparison

| Feature | OpenAI | Anthropic | Gemini | Grok | Ollama |
|---------|--------|-----------|--------|------|--------|
| Quality | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| Speed | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| Cost | Paid | Paid | Free Tier | Paid | Free |
| Privacy | Cloud | Cloud | Cloud | Cloud | Local |
| Offline | ❌ | ❌ | ❌ | ❌ | ✅ |

## Custom Providers

You can implement your own provider by implementing the `AiProvider` trait:

```rust
use aether_core::{AiProvider, provider::{GenerationRequest, GenerationResponse}};
use async_trait::async_trait;

struct MyProvider;

#[async_trait]
impl AiProvider for MyProvider {
    fn name(&self) -> &str {
        "my-provider"
    }

    async fn generate(&self, request: GenerationRequest) -> Result<GenerationResponse> {
        // Your implementation here
    }
}
```
