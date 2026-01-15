# Aether FFI - C/C++ Bindings

C/C++ FFI bindings for the Aether AI code injection framework.

## Building

```bash
# From workspace root
cargo build -p aether-ffi --release
```

This produces:

- **Windows**: `target/release/aether.dll` and `target/release/aether.lib`
- **Linux**: `target/release/libaether.so` and `target/release/libaether.a`
- **macOS**: `target/release/libaether.dylib` and `target/release/libaether.a`

The C header is automatically generated at `crates/aether-ffi/include/aether.h`.

## Usage (C++)

```cpp
#include <cstdio>
#include "aether.h"

int main() {
    // Initialize provider (requires OPENAI_API_KEY env var)
    AetherProvider* provider = aether_create_openai_provider("gpt-4o");
    if (!provider) {
        printf("Error: %s\n", aether_last_error());
        return 1;
    }

    // Create an engine
    AetherEngine* engine = aether_create_engine(provider);

    // Enable advanced features
    aether_engine_enable_healing(engine);   // Auto-validate & retry
    aether_engine_enable_cache(engine);     // Cache similar prompts
    aether_engine_set_toon(engine, true);   // Compress for token efficiency
    aether_engine_set_max_retries(engine, 3);

    // Create a template with AI slots
    AetherTemplate* tmpl = aether_create_template("<button>{{AI:text}}</button>");
    aether_template_add_slot(tmpl, "text", "Generate a compelling call-to-action text");

    // Render (AI generates the slot content)
    char* result = aether_render(engine, tmpl);
    if (result) {
        printf("Generated: %s\n", result);
        aether_free_string(result);
    } else {
        printf("Error: %s\n", aether_last_error());
    }

    // Cleanup
    aether_free_template(tmpl);
    aether_free_engine(engine);
    aether_free_provider(provider);
    return 0;
}
```

## Compiling Your C++ Project

### Windows (MSVC)

```bash
cl /EHsc main.cpp /I path\to\aether-ffi\include path\to\target\release\aether.dll.lib
```

### Linux / macOS

```bash
g++ -o main main.cpp -I/path/to/aether-ffi/include -L/path/to/target/release -laether -lpthread -ldl
```

### CMake Integration

```cmake
find_library(AETHER_LIB aether PATHS "${CMAKE_SOURCE_DIR}/lib")
target_include_directories(your_target PRIVATE "${CMAKE_SOURCE_DIR}/include")
target_link_libraries(your_target PUBLIC ${AETHER_LIB})
```

## API Reference

| Function | Description |
|----------|-------------|
| `aether_create_openai_provider(model)` | Create OpenAI provider |
| `aether_create_anthropic_provider(model)` | Create Anthropic (Claude) provider |
| `aether_create_gemini_provider(model)` | Create Google Gemini provider |
| `aether_create_ollama_provider(model)` | Create Ollama (local) provider |
| `aether_free_provider(provider)` | Free provider handle |
| `aether_create_engine(provider)` | Create injection engine |
| `aether_engine_enable_healing(engine)` | **Enable Self-Healing** (validates & retries) |
| `aether_engine_enable_cache(engine)` | **Enable Semantic Caching** (reduces API costs) |
| `aether_engine_set_toon(engine, enabled)` | **Enable TOON Protocol** (token compression) |
| `aether_engine_set_max_retries(engine, n)` | Set max healing retry count |
| `aether_free_engine(engine)` | Free engine handle |
| `aether_create_template(content)` | Create template from string |
| `aether_template_add_slot(template, name, prompt)` | Add slot to template |
| `aether_free_template(template)` | Free template handle |
| `aether_render(engine, template)` | Render template (AI generation) |
| `aether_generate(provider, prompt)` | One-shot code generation |
| `aether_free_string(s)` | Free string allocated by Aether |
| `aether_last_error()` | Get last error message |
| `aether_version()` | Get Aether version string |

## Thread Safety

- All provider creation functions are thread-safe
- Engine and template handles are NOT thread-safe; use separate handles per thread
- Error messages are thread-local

## Memory Management

- All handles must be freed using their corresponding `aether_free_*` functions
- All returned strings must be freed using `aether_free_string()`
- Provider handles can be shared across multiple engines
