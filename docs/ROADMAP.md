# Aether Roadmap

This document outlines the planned development roadmap for Aether Codegen.

## Vision

Make AI-powered code generation accessible to every developer, for every language and platform.

---

## Current Status: v0.1.0 ✅

**Released: January 15, 2026**

- Core template engine with `{{AI:slot}}` syntax
- AI Providers: OpenAI, Anthropic, Ollama
- Node.js bindings via NAPI-RS
- NPM package: `aether-codegen`
- Basic documentation

---

## Q1 2026: Foundation 🏗️

### v0.2.0 - CLI & Developer Experience

| Feature | Status | Priority |
|---------|--------|----------|
| **CLI Tool** | 🔲 Planned | High |
| Project scaffolding (`aether new`) | 🔲 Planned | High |
| Component generation (`aether generate`) | 🔲 Planned | High |
| Template management (`aether template`) | 🔲 Planned | Medium |
| **Streaming Generation** | 🔲 Planned | High |
| Real-time output while generating | 🔲 Planned | High |
| Progress indicators | 🔲 Planned | Medium |
| **Caching Layer** | 🔲 Planned | Medium |
| In-memory cache | 🔲 Planned | Medium |
| Redis/Disk cache support | 🔲 Planned | Low |
| **AI Protocol Optimization** | 🔲 Planned | Medium |
| TOON Format Support | 🔲 Planned | High |
| Token Usage Optimization | 🔲 Planned | Medium |

### v0.3.0 - Language Bindings

| Feature | Status | Priority |
|---------|--------|----------|
| **Python Bindings (PyO3)** | 🔲 Planned | High |
| pip package: `aether-codegen` | 🔲 Planned | High |
| Async support with asyncio | 🔲 Planned | High |
| **WASM Support** | 🔲 Planned | Medium |
| Browser-based generation | 🔲 Planned | Medium |
| Cloudflare Workers support | 🔲 Planned | Low |

---

## Q2 2026: Ecosystem 🌍

### v0.4.0 - IDE Integration

| Feature | Status | Priority |
|---------|--------|----------|
| **VSCode Extension** | 🔲 Planned | High |
| Syntax highlighting for templates | 🔲 Planned | High |
| Inline AI generation | 🔲 Planned | High |
| Template preview | 🔲 Planned | Medium |
| **JetBrains Plugin** | 🔲 Planned | Medium |
| IntelliJ, WebStorm, PyCharm | 🔲 Planned | Medium |

### v0.5.0 - Template Marketplace

| Feature | Status | Priority |
|---------|--------|----------|
| **Template Registry** | 🔲 Planned | High |
| Publish templates | 🔲 Planned | High |
| Install from registry | 🔲 Planned | High |
| Version management | 🔲 Planned | Medium |
| **Community Templates** | 🔲 Planned | Medium |
| React components | 🔲 Planned | Medium |
| API boilerplates | 🔲 Planned | Medium |
| Test generators | 🔲 Planned | Medium |

---

## Q3 2026: Advanced Features ⚡

### v0.6.0 - AI Pipelines

| Feature | Status | Priority |
|---------|--------|----------|
| **Multi-step Pipelines** | 🔲 Planned | High |
| Chain multiple AI calls | 🔲 Planned | High |
| Conditional branching | 🔲 Planned | Medium |
| Error recovery | 🔲 Planned | Medium |
| **Code Validation** | 🔲 Planned | High |
| Syntax checking | 🔲 Planned | High |
| Linting integration | 🔲 Planned | Medium |
| Security scanning | 🔲 Planned | Medium |

### v0.7.0 - Multimodal Support

| Feature | Status | Priority |
|---------|--------|----------|
| **Image-to-Code** | 🔲 Planned | High |
| Design mockup conversion | 🔲 Planned | High |
| Screenshot to component | 🔲 Planned | High |
| **Figma Integration** | 🔲 Planned | Medium |
| Import designs directly | 🔲 Planned | Medium |
| Component mapping | 🔲 Planned | Medium |

---

## Q4 2026: Enterprise 🏢

### v0.8.0 - Collaboration

| Feature | Status | Priority |
|---------|--------|----------|
| **Team Features** | 🔲 Planned | Medium |
| Shared templates | 🔲 Planned | Medium |
| Usage analytics | 🔲 Planned | Low |
| **Version History** | 🔲 Planned | Medium |
| Generation history | 🔲 Planned | Medium |
| Rollback support | 🔲 Planned | Medium |

### v0.9.0 - Self-Hosted

| Feature | Status | Priority |
|---------|--------|----------|
| **Local LLM Optimization** | 🔲 Planned | High |
| Optimized prompts for local models | 🔲 Planned | High |
| Fine-tuned model support | 🔲 Planned | Medium |
| **Enterprise Features** | 🔲 Planned | Medium |
| SSO integration | 🔲 Planned | Low |
| Audit logging | 🔲 Planned | Low |

---

## v1.0.0 - Production Ready 🎉

**Target: Q4 2026**

- Stable API
- Comprehensive documentation
- Full test coverage
- Production security audit
- LTS support

---

## Language Support Roadmap

| Language | Bindings | Status |
|----------|----------|--------|
| Rust | Native | ✅ Complete |
| Node.js | NAPI-RS | ✅ Complete |
| Python | PyO3 | 🔲 v0.3.0 |
| Go | CGO | 🔲 v0.5.0 |
| Java | JNI | 🔲 v0.6.0 |
| C# | P/Invoke | 🔲 v0.7.0 |
| Ruby | FFI | 🔲 v0.8.0 |

---

## AI Provider Roadmap

| Provider | Status | Notes |
|----------|--------|-------|
| OpenAI | ✅ Complete | GPT-5.2, GPT-4o |
| Anthropic | ✅ Complete | Claude Opus 4.5 |
| Ollama | ✅ Complete | Local models |
| Google Gemini | 🔲 v0.3.0 | |
| Mistral AI | 🔲 v0.4.0 | |
| Cohere | 🔲 v0.5.0 | |
| Custom/Self-hosted | 🔲 v0.6.0 | OpenAI-compatible API |

---

## Contributing

Want to help shape the roadmap? We welcome contributions!

- 🐛 [Report bugs](https://github.com/sjkim1127/aether-codegen/issues)
- 💡 [Request features](https://github.com/sjkim1127/aether-codegen/discussions)
- 🔧 [Submit PRs](https://github.com/sjkim1127/aether-codegen/pulls)

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Complete |
| 🚧 | In Progress |
| 🔲 | Planned |
| ❌ | Cancelled |

---

*Last updated: January 15, 2026*
