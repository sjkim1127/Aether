# Changelog

All notable changes to this project will be documented in this file.

## [0.1.4] - 2026-01-15

### Added

- **TDD Self-Healing**: Enhanced validation module to execute unit tests against AI-generated code.
  - New `TddValidator` that runs test harnesses in Rust, JavaScript, and Python.
  - Added `test_harness` and `test_command` constraints to `Slot`.
- **Incremental Rendering**: Optimized rendering by skipping generation for unchanged slots.
  - Added `RenderSession` for bit-exact change detection using slot and context fingerprints.
  - Implemented `Hash`, `Eq`, and `PartialEq` for core types (`Slot`, `InjectionContext`, etc.).
- **Aether Script (.ae)**: Introduced a specialized DSL for agentic workflows.
  - New `@ai` directive for first-class AI calls within scripts.
  - `AetherAgenticRuntime` for executing scripts with integrated AI capabilities.
- **Aether Shield Encryption**: Compile-time prompt encryption for enhanced security.
  - Encrypts AI prompts using AES-256-GCM at compile time based on `AETHER_SHIELD_KEY`.
- **Examples**:
  - `tdd_demo.rs` & `tdd_js_demo.rs`: Demonstrating self-healing with unit tests.
  - `incremental_demo.rs`: Showcasing performance gains with incremental rendering.
  - `script_demo.rs`: Exploring agentic workflows with Aether Script.
  - `shield_demo.rs`: Demonstrating prompt encryption.

### Fixed

- Improved MultiValidator to support slot-specific validation.
- Fixed various unused variable and import warnings in `aether-core` and `aether-ai`.
- Resolved provider type mismatches in examples.
- Corrected AES-GCM nonce length (12 bytes).

## [0.1.3] - 2026-01-15

- Multi-language validators
- AetherConfig initialization from env

## [0.1.2] - 2026-01-15

- Python bindings
- Schrödinger's Vault example

## [0.1.1] - 2026-01-15

- Gemini/Grok provider support
- Self-Healing basics
- TOON Protocol

## [0.1.0] - 2026-01-15

- Initial release
