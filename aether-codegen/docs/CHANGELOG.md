# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Cross-platform support (macOS, Linux)
- WebAssembly (WASM) support for browser usage
- Streaming code generation
- Custom AI provider plugins
- Template caching and optimization

---

## [0.1.0] - 2026-01-15

### Added

#### Core Framework
- **Template System**: Parse templates with `{{AI:slot_name}}` syntax for defining injection points
- **Slot Types**: Support for multiple slot kinds (`html`, `css`, `js`, `function`, `class`, `component`, `raw`)
- **Slot Constraints**: Validation with max lines, max chars, forbidden patterns
- **Injection Context**: Project, language, and framework context for better code generation
- **Parallel Generation**: Generate multiple slots concurrently with retry logic

#### AI Providers
- **OpenAI Provider**: Full support for GPT-5.2 models (Instant, Thinking, Pro)
- **Anthropic Provider**: Support for Claude Opus 4.5 and Claude Sonnet 4
- **Ollama Provider**: Local LLM support with CodeLlama and other models
- **Provider Trait**: Extensible `AiProvider` trait for custom backends
- **Mock Provider**: Built-in mock provider for testing

#### Rust Crates
- `aether-core`: Core library with template, slot, provider, and engine modules
- `aether-ai`: AI provider implementations (OpenAI, Anthropic, Ollama)
- `aether-macros`: Procedural macros (`ai!`, `ai_slot!`, `ai_template!`, `ai_generate`)
- `aether-node`: Node.js native bindings via NAPI-RS

#### Node.js Bindings
- **Classes**: `Template`, `Slot`, `AetherEngine`
- **Functions**: `generate()`, `renderTemplate()`
- **TypeScript**: Full type definitions included
- **Platform**: Windows x64 support (more platforms coming via CI)

#### Documentation
- Comprehensive README with usage examples
- API documentation with doc comments
- Publishing guide for NPM

### Technical Details

#### Dependencies
- Rust 2021 Edition
- tokio 1.43 (async runtime)
- reqwest 0.12 (HTTP client)
- handlebars 6.0 (template engine)
- serde 1.0 (serialization)
- napi 2.16 (Node.js bindings)

#### Default Models
- OpenAI: `gpt-5.2-thinking`
- Anthropic: `claude-opus-4-5`
- Ollama: `codellama`

---

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| 0.1.0 | 2026-01-15 | Initial release with core framework and Node.js bindings |

---

## Migration Guides

### Upgrading to 0.1.0

This is the initial release. No migration required.

---

## Links

- [GitHub Repository](https://github.com/sjkim1127/aether-codegen)
- [NPM Package](https://www.npmjs.com/package/aether-codegen)
- [Documentation](./docs/)

[Unreleased]: https://github.com/sjkim1127/aether-codegen/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/sjkim1127/aether-codegen/releases/tag/v0.1.0
