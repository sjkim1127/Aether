# Aether Codegen ⚡
>
> **The Infrastructure Layer for AI Code Generation**

[![Crates.io](https://img.shields.io/badge/crates.io-v0.1.5-orange.svg)](https://crates.io/crates/aether-core)
[![NPM](https://img.shields.io/badge/npm-v0.1.5-red.svg)](https://www.npmjs.com/package/@aether/codegen)
[![PyPI](https://img.shields.io/badge/pypi-v0.1.5-blue.svg)](https://pypi.org/project/aether-codegen/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**Aether** is a high-performance, type-safe framework designed to bridge the gap between *Creative LLMs* and *Rigid Software Engineering*.

Unlike simple API wrappers, Aether provides a **structured runtime** for code generation, featuring **semantic caching**, **self-healing TDD loops**, and **protocol optimizations** (TOON) that save 40%+ on token costs.

---

## 🚀 Why Aether?

| Feature | Raw API Calls | Aether Framework |
| :--- | :--- | :--- |
| **Reliability** | "Hope it works" flavor. | **Self-Healing**: Auto-compiles & fixes errors. |
| **Latency** | 1-5s per request. | **<10ms** for cached semantic hits. |
| **Cost** | Pay for every character. | **TOON Protocol**: ~40% token reduction. |
| **Integration** | String concatenation spaghetti. | **Templates**: Clean `{{AI:slot}}` syntax. |
| **Security** | Keys in env/code. | **Remote Resolution**: Keys never touch the binary. |

---

## � Installation & Quick Start

Choose your ecosystem. Aether provides native bindings for maximum performance.

### 🦀 Rust (Core)

Add to `Cargo.toml`:

```toml
[dependencies]
aether-core = "0.1.5"
aether-ai = "0.1.5"
tokio = { version = "1", features = ["full"] }
```

**main.rs**:

```rust
use aether_ai::{openai, InjectionEngine, Template};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = openai("gpt-5.1")?; // Or "claude-opus-4.5", "gemini-2.0-pro"
    let engine = InjectionEngine::new(provider);

    let template = Template::new("fn calculate(n: i32) -> i32 { {{AI:code}} }")
        .with_slot("code", "Return nth fibonacci number recursively");

    let code = engine.render(&template).await?;
    println!("{}", code);
    Ok(())
}
```

### 🐢 Node.js (TypeScript)

```bash
npm install @aether/codegen
```

**index.ts**:

```typescript
import { AetherEngine, Template } from '@aether/codegen';

async function main() {
    // Enable premium features: Caching, Healing, Parallelism
    const engine = AetherEngine.openai("gpt-5.1");
    engine.setCache(true);
    engine.setHeal(true);

    const tmpl = new Template("export const {{AI:name}} = (props) => { {{AI:body}} }");
    tmpl.addSlot("name", "Name for a login component");
    tmpl.addSlot("body", "React component logic with tailwind styles");

    const result = await engine.render(tmpl);
    console.log(result);
}
main();
```

### 🐍 Python

```bash
pip install aether-codegen
```

**app.py**:

```python
import aether
import asyncio

async def main():
    engine = aether.Engine("anthropic", model="claude-sonnet-4.5")
    engine.set_toon(True) # Enable optimization
    
    # Simple one-line generation
    code = await aether.generate("Write a Python decorator for timing functions")
    print(code)

if __name__ == "__main__":
    asyncio.run(main())
```

---

## 🛠️ Core Features

### 1. Templates & Slots

Don't prompt-engineer strings. Use **Templates**.

```html
<div class="card">
  <!-- Raw injection -->
  <h1>{{AI:title}}</h1>
  
  <!-- Kind-aware injection (enforces valid HTML) -->
  <div class="content">{{AI:body:html}}</div>
  
  <!-- Strict logic injection (Temperature 0.0) -->
  <script>
    {{AI:validation_logic:js}}
  </script>
</div>
```

### 2. SCSEM (Semantic Caching)

Aether includes a local Vector Database (using `fastembed`).

- **Hit**: If you ask "Make a login button" and later ask "Create a sign-in button", Aether understands they are semantically identical and returns the cached code instantly (0ms API cost).
- **Miss**: Falls back to the LLM.

### 3. TOON Protocol (Token-Oriented Object Notation)

A custom serialization format that replaces JSON for context injection.

- **JSON**: `[{"id": 1, "name": "foo"}, {"id": 2, "name": "bar"}]` (Tokens: High)
- **TOON**: `{id,name}: 1,foo | 2,bar` (Tokens: Low)
- **Result**: **30-60% cost reduction** on context-heavy prompts.

### 4. Self-Healing (TDD)

When `healing` is enabled:

1. AI generates code.
2. Aether compiles it (Rust/TS/Python/Go) in a sandbox.
3. If it fails, Aether captures the error log.
4. Aether feeds the error back to the AI: *"You made error X on line Y. Fix it."*
5. Repeat until success or `max_retries`.

---

## 📚 Documentation

Detailed documentation is available in the [Wiki Repository](./wiki_repo).

- [**Architecture**](./wiki_repo/architecture.md): How the Injection Engine works.
- [**AI Providers**](./wiki_repo/ai-providers.md): Configuring OpenAI, Claude, Gemini, Ollama, Grok.
- [**Template Syntax**](./wiki_repo/template-syntax.md): Full guide to slots and constraints.
- [**Comparisons**](./wiki_repo/COMPARISON.md): Aether vs LangChain vs Raw API.
- [**Roadmap**](./wiki_repo/ROADMAP.md): Future plans.

---

## 🤝 Supported Providers

| Provider | Key Env Var | Models |
| :--- | :--- | :--- |
| **OpenAI** | `OPENAI_API_KEY` | `gpt-5.1` (flagship), `gpt-5.2` (reasoning) |
| **Anthropic** | `ANTHROPIC_API_KEY` | `claude-opus-4.5`, `claude-sonnet-4.5` |
| **Google** | `GOOGLE_API_KEY` | `gemini-2.0-pro`, `gemini-2.0-flash` |
| **xAI** | `XAI_API_KEY` | `grok-3` |
| **Ollama** | - | `llama-4`, `mistral-large-v3` |

---

## 📜 License

MIT License © 2026 Aether Team.
