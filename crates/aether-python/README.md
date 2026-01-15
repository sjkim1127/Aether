# aether-codegen

High-performance AI code injection for Python, powered by Rust.

## Installation

```bash
pip install aether-codegen
```

## Usage

```python
import aether

# Initialize Engine
engine = aether.Engine("anthropic", model="claude-3-opus-20240229")

# Configure Options
engine.set_healing(True)
engine.set_cache(True)
engine.set_toon(True)
engine.set_parallel(True)
engine.set_max_retries(3)
engine.set_context(project="my-app", language="python", framework="flask")

# Create Template
template = aether.Template("def calculate(): {{AI:code}}")
template.add_slot("code", "Calculate prime numbers up to n")

# Render
result = engine.render(template)
print(result)
```

## Features

- 🚀 **High Performance**: Native Rust core via PyO3
- ✨ **Self-Healing**: Automatic TDD verification loop
- 🧠 **Semantic Cache**: Local vector-based caching
- 📦 **TOON Protocol**: 30-60% token savings
