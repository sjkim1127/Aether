# Changelog

All notable changes to this project will be documented in this file.

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
[0.1.1]: https://github.com/sjkim1127/Aether/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/sjkim1127/Aether/releases/tag/v0.1.0
