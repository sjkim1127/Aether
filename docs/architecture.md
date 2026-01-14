# Architecture

This document describes the architecture of the Aether Codegen framework.

## Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        User Application                          │
├──────────────────┬──────────────────┬────────────────────────────┤
│   Rust (Native)  │  Node.js (NAPI)  │  Future: WASM, Python, etc │
├──────────────────┴──────────────────┴────────────────────────────┤
│                                                                   │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                    aether-core                               │ │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌─────────────┐  │ │
│  │  │ Template  │ │   Slot    │ │  Context  │ │   Engine    │  │ │
│  │  │  Parser   │ │ Validator │ │  Manager  │ │ Orchestrator│  │ │
│  │  └───────────┘ └───────────┘ └───────────┘ └─────────────┘  │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                              │                                    │
│                              ▼                                    │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                     aether-ai                                │ │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌─────────────┐  │ │
│  │  │  OpenAI   │ │ Anthropic │ │  Ollama   │ │    Mock     │  │ │
│  │  │ Provider  │ │ Provider  │ │ Provider  │ │  Provider   │  │ │
│  │  └───────────┘ └───────────┘ └───────────┘ └─────────────┘  │ │
│  └─────────────────────────────────────────────────────────────┘ │
│                              │                                    │
│                              ▼                                    │
│                     ┌───────────────┐                            │
│                     │   AI APIs     │                            │
│                     │ (HTTP/REST)   │                            │
│                     └───────────────┘                            │
└─────────────────────────────────────────────────────────────────┘
```

## Crate Structure

```
aether-codegen/
├── Cargo.toml                 # Workspace root
├── crates/
│   ├── aether-core/           # Core library
│   │   ├── src/
│   │   │   ├── lib.rs         # Module exports
│   │   │   ├── error.rs       # Error types
│   │   │   ├── template.rs    # Template parsing
│   │   │   ├── slot.rs        # Slot definitions
│   │   │   ├── provider.rs    # AiProvider trait
│   │   │   ├── context.rs     # Injection context
│   │   │   └── engine.rs      # Main orchestrator
│   │   └── Cargo.toml
│   │
│   ├── aether-ai/             # AI provider implementations
│   │   ├── src/
│   │   │   ├── lib.rs         # Convenience functions
│   │   │   ├── error.rs       # AI-specific errors
│   │   │   ├── openai.rs      # OpenAI provider
│   │   │   ├── anthropic.rs   # Anthropic provider
│   │   │   └── ollama.rs      # Ollama provider
│   │   └── Cargo.toml
│   │
│   ├── aether-macros/         # Procedural macros
│   │   ├── src/
│   │   │   └── lib.rs         # ai!, ai_slot!, etc.
│   │   └── Cargo.toml
│   │
│   └── aether-node/           # Node.js bindings
│       ├── src/
│       │   └── lib.rs         # NAPI bindings
│       ├── index.js           # JS loader
│       ├── index.d.ts         # TypeScript types
│       └── package.json
│
├── examples/                  # Usage examples
├── docs/                      # Documentation
└── .github/workflows/         # CI/CD
```

## Core Components

### Template

Parses template content and extracts slots:

```rust
pub struct Template {
    pub content: String,
    pub name: String,
    pub slots: HashMap<String, Slot>,
    pub metadata: TemplateMetadata,
}
```

**Key Methods:**
- `new(content)` - Create template from string
- `with_slot(name, prompt)` - Add/configure slot
- `render(injections)` - Render with generated code
- `find_locations()` - Get slot positions

### Slot

Represents an injection point:

```rust
pub struct Slot {
    pub name: String,
    pub prompt: String,
    pub kind: SlotKind,
    pub constraints: Option<SlotConstraints>,
    pub required: bool,
    pub default: Option<String>,
}
```

### InjectionEngine

Orchestrates the code generation process:

```rust
pub struct InjectionEngine<P: AiProvider> {
    provider: Arc<P>,
    global_context: InjectionContext,
    parallel: bool,
    max_retries: u32,
}
```

**Key Methods:**
- `render(template)` - Generate and inject code
- `render_with_context(template, context)` - With additional context
- `generate_slot(template, slot_name)` - Single slot generation

### AiProvider Trait

Interface for AI backends:

```rust
#[async_trait]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    
    async fn generate(&self, request: GenerationRequest) 
        -> Result<GenerationResponse>;
    
    async fn generate_batch(&self, requests: Vec<GenerationRequest>) 
        -> Result<Vec<GenerationResponse>>;
    
    async fn health_check(&self) -> Result<bool>;
}
```

## Data Flow

1. **Template Parsing**
   ```
   "Hello {{AI:greeting}}" → Template { slots: {"greeting": Slot {...}} }
   ```

2. **Slot Configuration**
   ```
   template.with_slot("greeting", "Generate a friendly greeting")
   ```

3. **Generation Request**
   ```
   GenerationRequest {
       slot: Slot { name: "greeting", prompt: "..." },
       context: Some("Language: HTML"),
       system_prompt: None,
   }
   ```

4. **AI Response**
   ```
   GenerationResponse {
       code: "Hello, World!",
       tokens_used: Some(10),
   }
   ```

5. **Template Rendering**
   ```
   "Hello {{AI:greeting}}" + {"greeting": "World!"} → "Hello World!"
   ```

## Async Execution

The framework uses Tokio for async execution:

- **Parallel Generation**: Multiple slots generated concurrently
- **Retry Logic**: Automatic retry on transient failures
- **Timeout Handling**: Configurable request timeouts

## Node.js Integration

NAPI-RS provides zero-copy bindings:

```
Rust (aether-node) ←→ NAPI ←→ Node.js (JavaScript)
```

- `Template` → JS class with async methods
- `AetherEngine` → Factory methods for providers
- `generate()` → Top-level async function
