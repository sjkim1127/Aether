#ifndef AETHER_H
#define AETHER_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Opaque engine handle
 */
typedef struct AetherEngine AetherEngine;

/**
 * Opaque provider handle
 */
typedef struct AetherProvider AetherProvider;

/**
 * Opaque template handle
 */
typedef struct AetherTemplate AetherTemplate;

/**
 * Callback type for streaming chunks.
 *
 * # Arguments
 * * `chunk` - The chunk of generated text (null-terminated C string)
 * * `user_data` - User-provided context pointer
 *
 * # Returns
 * Return true to continue streaming, false to abort.
 */
typedef bool (*AetherStreamCallback)(const char *chunk, void *user_data);

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Get the last error message.
 * Returns NULL if no error occurred.
 * The returned string is valid until the next FFI call on the same thread.
 */
const char *aether_last_error(void);

/**
 * Create an OpenAI provider.
 *
 * # Arguments
 * * `model` - Model name (e.g., "gpt-4o", "gpt-4-turbo"). Pass NULL for default.
 *
 * # Returns
 * Provider handle on success, NULL on failure. Check `aether_last_error()`.
 */
struct AetherProvider *aether_create_openai_provider(const char *model);

/**
 * Create an Anthropic (Claude) provider.
 */
struct AetherProvider *aether_create_anthropic_provider(const char *model);

/**
 * Create a Google Gemini provider.
 */
struct AetherProvider *aether_create_gemini_provider(const char *model);

/**
 * Create an Ollama (local) provider.
 */
struct AetherProvider *aether_create_ollama_provider(const char *model);

/**
 * Free a provider handle.
 */
void aether_free_provider(struct AetherProvider *provider);

/**
 * Create an injection engine from a provider.
 *
 * # Arguments
 * * `provider` - Provider handle (ownership is NOT transferred)
 *
 * # Returns
 * Engine handle on success, NULL on failure.
 */
struct AetherEngine *aether_create_engine(const struct AetherProvider *provider);

/**
 * Free an engine handle.
 */
void aether_free_engine(struct AetherEngine *engine);

/**
 * Enable Self-Healing on the engine.
 * When enabled, generated code is validated and regenerated on errors.
 *
 * # Arguments
 * * `engine` - Engine handle (must be mutable)
 *
 * # Returns
 * true on success, false on failure
 */
bool aether_engine_enable_healing(struct AetherEngine *engine);

/**
 * Enable Semantic Caching on the engine.
 * Reduces API costs by caching similar prompts.
 *
 * # Arguments
 * * `engine` - Engine handle (must be mutable)
 *
 * # Returns
 * true on success, false on failure
 */
bool aether_engine_enable_cache(struct AetherEngine *engine);

/**
 * Enable TOON Protocol on the engine.
 * Compresses context for token efficiency.
 *
 * # Arguments
 * * `engine` - Engine handle (must be mutable)
 * * `enabled` - Whether to enable TOON
 */
void aether_engine_set_toon(struct AetherEngine *engine, bool enabled);

/**
 * Set the maximum retry count for Self-Healing.
 *
 * # Arguments
 * * `engine` - Engine handle
 * * `max_retries` - Maximum number of healing attempts (default: 3)
 */
void aether_engine_set_max_retries(struct AetherEngine *engine, uint32_t max_retries);

/**
 * Create a template from content string.
 *
 * # Arguments
 * * `content` - Template content with `{{AI:slot}}` markers
 *
 * # Returns
 * Template handle on success, NULL on failure.
 */
struct AetherTemplate *aether_create_template(const char *content);

/**
 * Add a slot to a template.
 *
 * # Arguments
 * * `template` - Template handle
 * * `name` - Slot name
 * * `prompt` - AI prompt for this slot
 */
void aether_template_add_slot(struct AetherTemplate *template_,
                              const char *name,
                              const char *prompt);

/**
 * Free a template handle.
 */
void aether_free_template(struct AetherTemplate *template_);

/**
 * Render a template using the engine.
 *
 * # Arguments
 * * `engine` - Engine handle
 * * `template` - Template handle
 *
 * # Returns
 * Newly allocated string with the result. Caller must free with `aether_free_string()`.
 * Returns NULL on error. Check `aether_last_error()`.
 */
char *aether_render(const struct AetherEngine *engine, const struct AetherTemplate *template_);

/**
 * One-shot code generation (convenience function).
 *
 * # Arguments
 * * `provider` - Provider handle
 * * `prompt` - The prompt for code generation
 *
 * # Returns
 * Newly allocated string with generated code. Free with `aether_free_string()`.
 */
char *aether_generate(const struct AetherProvider *provider, const char *prompt);

/**
 * Free a string allocated by Aether.
 */
void aether_free_string(char *s);

/**
 * Render a template with streaming output.
 *
 * # Arguments
 * * `engine` - Engine handle
 * * `template` - Template handle
 * * `slot_name` - Name of the slot to stream
 * * `callback` - Function pointer called for each chunk
 * * `user_data` - User context passed to callback (can be NULL)
 *
 * # Returns
 * Newly allocated string with the full result. Caller must free with `aether_free_string()`.
 * Returns NULL on error. Check `aether_last_error()`.
 *
 * # Example (C++)
 * ```cpp
 * bool on_chunk(const char* chunk, void* user_data) {
 *     std::cout << chunk << std::flush;
 *     return true;  // continue streaming
 * }
 *
 * char* result = aether_render_stream(engine, tmpl, "code", on_chunk, nullptr);
 * ```
 */
char *aether_render_stream(const struct AetherEngine *engine,
                           const struct AetherTemplate *template_,
                           const char *slot_name,
                           AetherStreamCallback callback,
                           void *user_data);

/**
 * Get the Aether version string.
 */
const char *aether_version(void);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus

#endif /* AETHER_H */
