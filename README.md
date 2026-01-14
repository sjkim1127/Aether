# Aether Codegen

> AI-powered dynamic code injection framework for Rust

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

## Overview

Aether Codegen is a framework that enables AI to dynamically inject source code into templates. With a single line of code, you can call an AI to generate and inject code into predefined slots within your templates.

### Key Features

- 🎯 **One-line AI calls** - Generate code with a single macro invocation
- 🧪 **Self-Healing (Alpha)** - Automatic error detection and self-correction loop
- 🧠 **Semantic Caching** - Local response caching using embeddings to save costs
- � **TOON Protocol** - Token-Oriented Object Notation for extreme token efficiency
- 📡 **Live Streaming** - Real-time code generation for interactive experiences
- 🔌 **Expanded Providers** - OpenAI, Anthropic, Google Gemini, Ollama, and xAI Grok
- ⚡ **Parallel Generation** - Concurrent slot processing for maximum speed
- 🛡️ **Validation & Types** - Integrated constraints and type-safe injection
- 🧩 **Native Node.js** - High-performance C++ bindings for JavaScript ecosystem

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
aether-core = { path = "crates/aether-core" }
aether-ai = { path = "crates/aether-ai" }
aether-macros = { path = "crates/aether-macros" }
tokio = { version = "1.43", features = ["full"] }
```

## Quick Start

### One-Line Code Generation

```rust
use aether_ai::{openai, InjectionEngine, Template};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize provider with one line
    let provider = openai("gpt-5.2-thinking")?;
    
    // Create template and inject code with one line
    let engine = InjectionEngine::new(provider);
    let template = Template::new("<button>{{AI:button_text}}</button>")
        .with_slot("button_text", "Generate a creative call-to-action text");
    
    let result = engine.render(&template).await?;
    println!("{}", result);
    
    Ok(())
}
```

### Using the `ai!` Macro

```rust
use aether_macros::ai;
use aether_ai::OpenAiProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = OpenAiProvider::from_env()?;
    
    // One-line AI code generation
    let code = ai!("Create a login form with email and password fields", provider).await?;
    println!("{}", code);
    
    Ok(())
}
```

### Template Syntax

Templates use the `{{AI:slot_name}}` syntax for defining injection points:

```html
<!DOCTYPE html>
<html>
<head>
    <style>
        {{AI:styles:css}}
    </style>
</head>
<body>
    <header>{{AI:header:html}}</header>
    <main>{{AI:content:html}}</main>
    <footer>{{AI:footer:html}}</footer>
    <script>
        {{AI:script:js}}
    </script>
</body>
</html>
```

### Slot Kinds

You can specify the type of code to generate:

| Slot Kind | Syntax | Description |
|-----------|--------|-------------|
| Raw | `{{AI:name}}` | Raw code injection |
| HTML | `{{AI:name:html}}` | HTML markup |
| CSS | `{{AI:name:css}}` | CSS styles |
| JavaScript | `{{AI:name:js}}` | JavaScript code |
| Function | `{{AI:name:function}}` | Function definition |
| Class | `{{AI:name:class}}` | Class/struct definition |
| Component | `{{AI:name:component}}` | Full component |

## Architecture

```
aether-codegen/
├── crates/
│   ├── aether-core/       # Core Engine & Infrastructure
│   │   ├── validation.rs  # Self-Healing & Validation logic
│   │   ├── cache.rs       # Semantic Caching with fastembed
│   │   ├── toon.rs        # TOON Protocol implementation
│   │   ├── engine.rs      # Main orchestrator
│   │   └── ...
│   ├── aether-ai/         # AI Provider Implementations
│   │   ├── gemini.rs      # Google Gemini (v1beta/v1)
│   │   ├── grok.rs        # xAI Grok (OpenAI-compatible)
│   │   ├── openai.rs      # OpenAI GPT series
│   │   └── ...
│   └── aether-node/       # NAPI-RS Node.js Bindings
└── examples/              # Usage patterns & demos
```

## AI Providers

### Google Gemini

```rust
use aether_ai::GeminiProvider;

// From environment (GOOGLE_API_KEY)
let provider = GeminiProvider::from_env()?;

// Or use the latest high-speed model
let provider = aether_ai::gemini("gemini-3-flash-preview")?;
```

### xAI Grok

```rust
// Grok uses the OpenAI-compatible implementation
let provider = aether_ai::grok("grok-1")?;
```

### OpenAI & Anthropic

- **OpenAI**: Supports `gpt-5.2`, `gpt-5.1`, etc.
- **Anthropic**: Supports `claude-4.5-sonnet-latest`, `claude-4.5-opus-latest`.

### Ollama (Local)

```rust
use aether_ai::OllamaProvider;
let provider = OllamaProvider::new("codellama");
``` 

## Advanced Usage

### Context-Aware Generation

```rust
use aether_core::{InjectionContext, InjectionEngine, Template};

let context = InjectionContext::new()
    .with_project("my-app")
    .with_language("typescript")
    .with_framework("react");

let engine = InjectionEngine::new(provider)
    .with_context(context);

let template = Template::new("{{AI:component}}")
    .with_slot("component", "Create a user profile card component");

let result = engine.render(&template).await?;
```

### Slot Constraints

```rust
use aether_core::{Slot, SlotConstraints, SlotKind};

let slot = Slot::new("code", "Generate a helper function")
    .with_kind(SlotKind::Function)
    .with_constraints(
        SlotConstraints::new()
            .max_lines(50)
            .language("rust")
            .forbid_pattern(r"unsafe\s*\{")  // No unsafe blocks
    );
```

### Parallel Generation

```rust
let engine = InjectionEngine::new(provider)
    .parallel(true)  // Enable parallel slot generation
    .max_retries(3); // Retry failed generations

let template = Template::new("{{AI:header}} {{AI:content}} {{AI:footer}}");
let result = engine.render(&template).await?; // All slots generated in parallel
```

### Advanced Workflow (Healing + Cache + TOON)

For complex use cases requiring high reliability and cost-efficiency:

```rust
let engine = InjectionEngine::new(provider)
    .with_cache(SemanticCache::new()?) 
    .with_validator(RustValidator)    
    .with_toon(true);

let template = Template::new("...")
    .configure_slot(
        Slot::new("logic", "Math logic")
            .with_temperature(0.0) // Be strict
    );
```

See [examples/advanced_workflow.rs](./examples/advanced_workflow.rs) for a complete implementation.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENAI_API_KEY` | OpenAI API key | - |
| `ANTHROPIC_API_KEY` | Anthropic API key | - |
| `GOOGLE_API_KEY` | Google Gemini API key | - |
| `XAI_API_KEY` | xAI Grok API key | - |
| `OLLAMA_MODEL` | Ollama model name | `codellama` |
| `OLLAMA_URL` | Ollama API URL | `http://localhost:11434/api/generate` |

## Node.js Bindings

Aether is also available as a native Node.js module via NAPI-RS.

### Installation

```bash
npm install @aether/codegen
```

### Quick Start (JavaScript/TypeScript)

```javascript
const { AetherEngine, Template, generate } = require('@aether/codegen');

// One-line code generation
const code = await generate("Create a login form with validation");
console.log(code);

// Enable Premium Features
const engine = AetherEngine.openai("gpt-4o");
engine.setHeal(true);  // Enable Self-Healing
engine.setCache(true); // Enable Semantic Cache
engine.setToon(true);  // Enable TOON Protocol

const template = new Template("<div>{{AI:content}}</div>");
template.setSlot("content", "Generate a welcome message");
const result = await engine.render(template);
```

### TypeScript Support

Full TypeScript definitions are included:

```typescript
import { AetherEngine, Template, generate, renderTemplate } from '@aether/codegen';

// Type-safe API
const engine: AetherEngine = AetherEngine.anthropic("claude-opus-4-5");
engine.setContext("my-app", "typescript", "react");

const html: string = await renderTemplate(
  "<header>{{AI:nav}}</header>",
  { nav: "Generate a responsive navigation bar" }
);
```

### Building from Source

```bash
cd crates/aether-node
npm install
npm run build
```

## Examples

See the [examples](./examples) directory for more usage patterns:

- `advanced_workflow.rs` - Full suite of premium features (Cache, Heal, TOON)
- `basic_web.rs` - Basic web page generation
- `one_line.rs` - One-line code generation
- `local_ollama.rs` - Local AI with Ollama

## Documentation

- [Architecture Overview](./docs/ARCHITECTURE.md) - How Aether works internally
- [Changelog](./docs/CHANGELOG.md) - Recent updates and version history
- [API Reference (Node.js)](./crates/aether-node/index.d.ts) - TypeScript definitions

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.
