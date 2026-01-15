# Changelog

All notable changes to this project will be documented in this file.

## [0.1.3] - 2026-01-15

### Added

- 🌍 **Multi-Language Validators**: New `JsValidator` (Node.js), `PythonValidator` (py_compile + ruff), and `MultiValidator` with auto-detection.
- ⚙️ **AetherConfig Module**: Centralized configuration with environment variable support (`AETHER_TOON`, `AETHER_HEALING`, `AETHER_CACHE`).
- 🚀 **Auto-TOON Threshold**: Automatically enable TOON compression when context exceeds threshold (default: 2000 chars).
- 🛡️ **Binding Shield Integration**: Added `execute_script` method to Python and Node.js bindings for direct Rhai execution.
- 🔧 **Python Binding Upgrades**: Added `set_healing()`, `set_cache()`, `set_toon()` methods to Python Engine class.

### Changed

- 📦 **Dependency Updates (2026-01)**:
  - `reqwest`: 0.12 → 0.13.1
  - `regex`: 1.11 → 1.12
  - `rhai`: 1.20 → 1.23
  - `dashmap`: 5.5.3 → 6.1.0
  - `tempfile`: 3.17 → 3.24
  - `serde_yaml` → `serde_yaml_ng` 0.10 (deprecated crate replacement)
- 🟢 **Node.js Bindings**:
  - Upgraded `@napi-rs/cli` from 2.x to 3.5.1
  - Updated NAPI config format (`binaryName`, `targets`)
- 🏗️ **InjectionEngine Refactor**: Now accepts `AetherConfig` via `with_config()` constructor.

### Fixed

- Fixed Windows cross-drive issue with `rustc` output path in RustValidator.
- Internal crate version sync (all workspace crates now at 0.1.2).

## [0.1.2] - 2026-01-15

### Added

- 🐍 **Python Bindings**: Official `aether-python` crate allowing the Aether engine to be used directly from Python. Published on PyPI as `aether-codegen`.
- 🔐 **Schrödinger's Vault Demo (Python)**: A new Python GUI demo showcasing Aether Shield for Python, protecting logic with an AI Oracle.
- 🛠️ **Maturin Integration**: Added support for building Python wheels using `maturin`.

### Changed

- 🔄 **Consistent Provider Config**: Refactored `aether-python` to properly handle different provider configurations (OpenAI, Anthropic, Gemini, Ollama).
- 🛡️ **Prompt Hardening**: Improved `aether_secure` macro prompts for better resilience against answer leakage.

### Fixed

- Fixed type mismatches in Python bindings for provider initialization.

## [0.1.1] - 2026-01-15

### Added

- 🚀 **New AI Providers**: Integrated **Google Gemini** (v1beta/v1) and **xAI Grok** (OpenAI-compatible) into the framework.
- 📡 **Real-time Streaming**: Added `generate_stream` support for OpenAI, Anthropic, Gemini, and Ollama providers.
- 🧪 **Self-Healing (Alpha)**: Implementation of `RustValidator` and automatic error correction loops in `InjectionEngine`.
- 🧠 **Semantic Caching**: Added local vector-based caching using `fastembed` and `dashmap` to reduce API latency and costs.
- 💎 **TOON Protocol**: Introduced **Token-Oriented Object Notation** for highly efficient token-safe context injection.
- 💻 **Aether CLI**: A new command-line interface for testing templates and AI providers directly from the terminal.
- 🎨 **Coding Challenge Demo**: Added a premium React/Express demo showcasing dynamic problem generation and premium UI effects.

### Changed

- 📦 **Dependency Upgrade**: Upgraded core dependencies including `tokio` (1.49), `thiserror` (2.0.17), `handlebars` (6.4), and `fastembed` (5.8).
- ⚙️ **Refinement**: Improved NAPI-RS bindings for Node.js to support premium features (`setHeal`, `setCache`, `setToon`).
- 📝 **Documentation**: Major update to `README.md` and added architectural documentation.

### Fixed

- Fixed unclosed code blocks and formatting issues in AI-generated templates.
- Resolved various unused import warnings and clarified error messages in high-latency environments.

## [0.1.0] - 2026-01-15

### Added

- Initial release of Aether Codegen framework.
- Basic template injection system with `{{AI:slot}}` syntax.
- Support for OpenAI, Anthropic Claude, and local Ollama.
- Core `InjectionEngine` with parallel slot generation.
- Basic Node.js bindings via NAPI-RS.

---
[0.1.3]: https://github.com/sjkim1127/Aether/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/sjkim1127/Aether/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/sjkim1127/Aether/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/sjkim1127/Aether/releases/tag/v0.1.0
