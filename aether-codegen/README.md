# Aether Codegen

> AI-powered dynamic code injection framework for Rust

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

## Overview

Aether Codegen is a framework that enables AI to dynamically inject source code into templates. With a single line of code, you can call an AI to generate and inject code into predefined slots within your templates.

### Key Features

- 🎯 **One-line AI calls** - Generate code with a single macro invocation
- 📝 **Template-based injection** - Define slots in templates using `{{AI:slot_name}}` syntax
- 🔌 **Multiple AI providers** - OpenAI, Anthropic Claude, and local Ollama
  - **OpenAI**: GPT-5.2-Instant, GPT-5.2-Thinking, GPT-5.2-Pro
  - **Anthropic**: Claude Opus 4.5, Claude Sonnet 4
  - **Local**: Ollama with CodeLlama and other models
- ⚡ **Parallel generation** - Generate multiple slots concurrently
- 🛡️ **Validation & constraints** - Define constraints on generated code
- 🧩 **Proc macros** - Compile-time markers and runtime helpers

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
│   ├── aether-core/     # Core library
│   │   ├── template.rs  # Template parsing
│   │   ├── slot.rs      # Slot definitions
│   │   ├── provider.rs  # AI provider trait
│   │   ├── context.rs   # Injection context
│   │   └── engine.rs    # Main orchestrator
│   ├── aether-ai/       # AI providers
│   │   ├── openai.rs    # OpenAI integration
│   │   ├── anthropic.rs # Claude integration
│   │   └── ollama.rs    # Local Ollama
│   └── aether-macros/   # Proc macros
│       └── lib.rs       # ai!, ai_slot!, etc.
└── examples/            # Usage examples
```

## AI Providers

### OpenAI

```rust
use aether_ai::OpenAiProvider;

// From environment (OPENAI_API_KEY)
let provider = OpenAiProvider::from_env()?;

// Or with explicit configuration
use aether_core::ProviderConfig;
let config = ProviderConfig::new("your-api-key", "gpt-5.2-thinking")
    .with_temperature(0.7)
    .with_max_tokens(2048);
let provider = OpenAiProvider::new(config)?;
```

### Available OpenAI Models

| Model | Use Case |
|-------|----------|
| `gpt-5.2-instant` | Fast responses, everyday tasks |
| `gpt-5.2-thinking` | Complex coding and logic (default) |
| `gpt-5.2-pro` | Highest quality, hard problems |

### Anthropic Claude

```rust
use aether_ai::AnthropicProvider;

// From environment (ANTHROPIC_API_KEY)
let provider = AnthropicProvider::from_env()?;

// Or with specific model
let provider = aether_ai::anthropic("claude-opus-4-5")?;
// Or use Claude Sonnet 4 for faster responses
let provider = aether_ai::anthropic("claude-sonnet-4")?;
```

### Ollama (Local)

```rust
use aether_ai::OllamaProvider;

// Default (localhost:11434)
let provider = OllamaProvider::new("codellama");

// Or with custom URL
let provider = OllamaProvider::with_options("codellama", "http://custom:11434/api/generate");
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

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENAI_API_KEY` | OpenAI API key | - |
| `AETHER_API_KEY` | Alternative API key | - |
| `AETHER_MODEL` | Default model | `gpt-5.2-thinking` |
| `ANTHROPIC_API_KEY` | Anthropic API key | - |
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

// Using engine with template
const engine = AetherEngine.openai("gpt-5.2-thinking");
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

- `basic_web.rs` - Basic web page generation
- `one_line.rs` - One-line code generation
- `local_ollama.rs` - Local AI with Ollama

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.
