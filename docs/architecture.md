# Aether Architecture

Aether is a high-performance, structured AI code injection framework. It is designed to bridge the gap between creative LLM generation and the strict requirements of source code.

For a detailed comparison on why Aether is superior to raw API calls, see [Aether vs Raw API](./COMPARISON.md).

## Core Concepts

### 1. Template & Slots
Instead of asking AI for a whole file, Aether uses **Templates**.
- **Templates**: Static files containing placeholders like `{{AI:slot_name}}`.
- **Slots**: Discrete injection points with their own prompts, constraints, and **Temperature** settings.

### 2. Injection Engine
The orchestrator that manages the lifecycle of code generation:
1. **Parsing**: Scans templates for slots.
2. **Context Assembly**: Gathers project context and applies **TOON** protocol optimization.
3. **Parallel Generation**: Dispatches slot requests to AI providers concurrently.
4. **Validation**: Runs generated code through constraints and `RustValidator`.
5. **Self-Healing**: If validation fails, it loops back to the AI with error feedback.
6. **Rendering**: Injects the final, verified code back into the template.

---

## Premium Features

### 🧪 Self-Healing (Alpha)
Aether doesn't just "hope" the AI gets it right. 
- Integrated `RustValidator` uses `cargo check` and temporary file compilation.
- If an error is detected, the engine automatically re-prompts the AI with the specific error message.

### 🧠 Semantic Caching
Powered by `fastembed`, Aether maintains a local vector database of previous generations.
- If a new prompt is semantically similar to a cached one, it returns the cached result.
- DRAMATICALLY reduces API costs and provides sub-millisecond responses for repeated patterns.

### 🚀 TOON Protocol (Token-Oriented Object Notation)
A custom serialization format designed for LLMs. 
- Traditional JSON/Markdown adds unnecessary token overhead.
- TOON structures context in a way that maximizes AI "understanding" while minimizing token consumption.

### 🌡️ Per-Slot Temperature
Fine-grained control over AI "creativity" within a single document.
- **Temp 0.0**: For logic, structs, and math. Ensures deterministic, strict output.
- **Temp 1.0**: For marketing text, naming, and storytelling. Encourages variety.

---

## Tech Stack
- **Core**: Rust (Safety & Speed)
- **AI Clients**: Reqwest + Async Trait
- **Caching**: fastembed-rs + DashMap
- **Bindings**: NAPI-RS (Native Node.js support)
- **Templates**: Handlebars + Custom Regex Parser
