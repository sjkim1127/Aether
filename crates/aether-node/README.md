# @aether/codegen

High-performance AI code injection for Node.js.

## Installation

```bash
npm install @aether/codegen
```

## Usage

```javascript
const { AetherEngine, Template, generate } = require('@aether/codegen');

// One-line generation
const code = await generate("Create a login form");

// Advanced Usage
const engine = AetherEngine.openai("gpt-4o");
engine.setHeal(true);
engine.setCache(true);
engine.setToon(true);
engine.setParallel(true);
engine.setMaxRetries(3);
engine.setContext("my-app", "javascript", "express");

const template = new Template("{{AI:endpoint}}");
template.setSlot("endpoint", "Create a secure login API endpoint");

const result = await engine.render(template);
```

## Features

- 🚀 **High Performance**: Native Rust binding via NAPI-RS
- ✨ **Self-Healing**: Automatic TDD verification loop
- 🧠 **Semantic Cache**: Local vector-based caching
- 📦 **TOON Protocol**: 30-60% token savings
