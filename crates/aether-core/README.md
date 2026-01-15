# aether-core

The core engine for the Aether Codegen framework.

## Features

- **InjectionEngine**: Main orchestrator for template rendering.
- **AetherConfig**: Centralized configuration management.
- **TOON Protocol**: Token-Oriented Object Notation for context compression.
- **Self-Healing**: Automated validation and feedback loops.
- **Semantic Caching**: Vector-based response caching.

## Usage

```rust
use aether_core::{InjectionEngine, Template, AetherConfig};
use aether_ai::OpenAiProvider;

#[tokio::main]
async fn main() {
    let provider = OpenAiProvider::from_env().unwrap();
    let config = AetherConfig::from_env();
    
    let engine = InjectionEngine::with_config(provider, config);
    
    let template = Template::new("{{AI:code}}")
        .with_slot("code", "Generate a hello world function");
        
    let result = engine.render(&template).await.unwrap();
    println!("{}", result);
}
```
