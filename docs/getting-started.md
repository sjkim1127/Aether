# Getting Started with Aether Codegen

This guide will help you get started with Aether Codegen, an AI-powered dynamic code injection framework.

## Prerequisites

- **Rust**: 1.75 or later (for Rust usage)
- **Node.js**: 18 or later (for Node.js usage)
- **API Key**: OpenAI, Anthropic, or local Ollama

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
aether-core = { git = "https://github.com/sjkim1127/aether-codegen" }
aether-ai = { git = "https://github.com/sjkim1127/aether-codegen" }
tokio = { version = "1", features = ["full"] }
```

### Node.js

```bash
npm install aether-codegen
```

## Quick Start

### 1. Set Your API Key

```bash
# For OpenAI
export OPENAI_API_KEY=your-api-key

# For Anthropic
export ANTHROPIC_API_KEY=your-api-key

# For Ollama (no key needed, just run ollama locally)
ollama serve
```

### 2. Generate Code

#### Rust

```rust
use aether_ai::{openai, InjectionEngine, Template};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = openai("gpt-5.2-thinking")?;
    let engine = InjectionEngine::new(provider);

    let template = Template::new("<button>{{AI:text}}</button>")
        .with_slot("text", "Generate a creative call-to-action");

    let html = engine.render(&template).await?;
    println!("{}", html);

    Ok(())
}
```

#### Node.js

```javascript
const { generate } = require('aether-codegen');

async function main() {
    const code = await generate("Create a login form with validation");
    console.log(code);
}

main();
```

## Core Concepts

### Templates

Templates are strings with AI slots marked by `{{AI:slot_name}}`:

```html
<div class="card">
    <h1>{{AI:title}}</h1>
    <p>{{AI:description}}</p>
    {{AI:button:html}}
</div>
```

### Slots

Slots are injection points where AI generates code. Each slot has:
- **Name**: Unique identifier
- **Prompt**: Instructions for the AI
- **Kind**: Type of code to generate (html, css, js, etc.)

### Providers

Providers are AI backends that generate the code:
- **OpenAI**: GPT-5.2 models (Instant, Thinking, Pro)
- **Anthropic**: Claude Opus 4.5, Claude Sonnet 4
- **Ollama**: Local models (CodeLlama, etc.)

## Next Steps

- Read the [Template Syntax Guide](./template-syntax.md)
- Explore [AI Providers](./ai-providers.md)
- Check out [Examples](../examples/)
