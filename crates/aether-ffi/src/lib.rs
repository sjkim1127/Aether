//! # Aether FFI
//!
//! C/C++ bindings for the Aether AI code injection framework.
//!
//! This crate provides a C-compatible API for integrating Aether
//! into C, C++, and other languages that support C FFI.
//!
//! ## Usage (C++)
//!
//! ```cpp
//! #include "aether.h"
//!
//! int main() {
//!     // Initialize a provider
//!     AetherProvider* provider = aether_create_openai_provider("gpt-4o");
//!     if (!provider) {
//!         printf("Error: %s\n", aether_last_error());
//!         return 1;
//!     }
//!
//!     // Create an engine
//!     AetherEngine* engine = aether_create_engine(provider);
//!
//!     // Create a template
//!     AetherTemplate* tmpl = aether_create_template("<button>{{AI:text}}</button>");
//!     aether_template_add_slot(tmpl, "text", "Generate a call-to-action button text");
//!
//!     // Render
//!     char* result = aether_render(engine, tmpl);
//!     if (result) {
//!         printf("Generated: %s\n", result);
//!         aether_free_string(result);
//!     }
//!
//!     // Cleanup
//!     aether_free_template(tmpl);
//!     aether_free_engine(engine);
//!     aether_free_provider(provider);
//!     return 0;
//! }
//! ```

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use std::sync::Arc;

use aether_core::{
    InjectionEngine, Template, AiProvider,
    validation::MultiValidator,
    cache::SemanticCache,
};
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

// Thread-local error message storage
thread_local! {
    static LAST_ERROR: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
}

/// Global Tokio runtime for async operations
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});

fn set_last_error(msg: String) {
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(msg));
}

// ============================================================
// Opaque Handle Types
// ============================================================

/// Opaque provider handle
pub struct AetherProvider {
    inner: Arc<dyn AiProvider + Send + Sync>,
}

/// Opaque engine handle
pub struct AetherEngine {
    inner: InjectionEngine<Arc<dyn AiProvider + Send + Sync>>,
    provider: Arc<dyn AiProvider + Send + Sync>,
    healing_enabled: bool,
    cache_enabled: bool,
    toon_enabled: bool,
    max_retries: usize,
}

impl AetherEngine {
    fn rebuild(&mut self) {
        let mut engine = InjectionEngine::new(self.provider.clone());
        
        if self.healing_enabled {
            engine = engine.with_validator(MultiValidator::new());
        }
        
        if self.cache_enabled {
            if let Ok(cache) = SemanticCache::new() {
                engine = engine.with_cache(cache);
            }
        }
        
        if self.toon_enabled {
            engine = engine.with_toon(true);
        }
        
        if self.max_retries > 0 {
            engine = engine.max_retries(self.max_retries as u32);
        }
        
        self.inner = engine;
    }
}

/// Opaque template handle
pub struct AetherTemplate {
    inner: Template,
}

// ============================================================
// Error Handling
// ============================================================

/// Get the last error message.
/// Returns NULL if no error occurred.
/// The returned string is valid until the next FFI call on the same thread.
#[no_mangle]
pub extern "C" fn aether_last_error() -> *const c_char {
    thread_local! {
        static ERROR_BUF: std::cell::RefCell<Option<CString>> = std::cell::RefCell::new(None);
    }

    LAST_ERROR.with(|e| {
        if let Some(ref msg) = *e.borrow() {
            ERROR_BUF.with(|buf| {
                let cstring = CString::new(msg.clone()).unwrap_or_default();
                let ptr = cstring.as_ptr();
                *buf.borrow_mut() = Some(cstring);
                ptr
            })
        } else {
            ptr::null()
        }
    })
}

// ============================================================
// Provider Creation
// ============================================================

/// Create an OpenAI provider.
///
/// # Arguments
/// * `model` - Model name (e.g., "gpt-4o", "gpt-4-turbo"). Pass NULL for default.
///
/// # Returns
/// Provider handle on success, NULL on failure. Check `aether_last_error()`.
#[no_mangle]
pub extern "C" fn aether_create_openai_provider(model: *const c_char) -> *mut AetherProvider {
    let model_str = if model.is_null() {
        "gpt-4o".to_string()
    } else {
        unsafe { CStr::from_ptr(model) }.to_string_lossy().into_owned()
    };

    match aether_ai::openai(&model_str) {
        Ok(provider) => {
            let handle = Box::new(AetherProvider {
                inner: Arc::new(provider),
            });
            Box::into_raw(handle)
        }
        Err(e) => {
            set_last_error(e.to_string());
            ptr::null_mut()
        }
    }
}

/// Create an Anthropic (Claude) provider.
#[no_mangle]
pub extern "C" fn aether_create_anthropic_provider(model: *const c_char) -> *mut AetherProvider {
    let model_str = if model.is_null() {
        "claude-3-opus-20240229".to_string()
    } else {
        unsafe { CStr::from_ptr(model) }.to_string_lossy().into_owned()
    };

    match aether_ai::anthropic(&model_str) {
        Ok(provider) => {
            let handle = Box::new(AetherProvider {
                inner: Arc::new(provider),
            });
            Box::into_raw(handle)
        }
        Err(e) => {
            set_last_error(e.to_string());
            ptr::null_mut()
        }
    }
}

/// Create a Google Gemini provider.
#[no_mangle]
pub extern "C" fn aether_create_gemini_provider(model: *const c_char) -> *mut AetherProvider {
    let model_str = if model.is_null() {
        "gemini-1.5-pro".to_string()
    } else {
        unsafe { CStr::from_ptr(model) }.to_string_lossy().into_owned()
    };

    match aether_ai::gemini(&model_str) {
        Ok(provider) => {
            let handle = Box::new(AetherProvider {
                inner: Arc::new(provider),
            });
            Box::into_raw(handle)
        }
        Err(e) => {
            set_last_error(e.to_string());
            ptr::null_mut()
        }
    }
}

/// Create an Ollama (local) provider.
#[no_mangle]
pub extern "C" fn aether_create_ollama_provider(model: *const c_char) -> *mut AetherProvider {
    let model_str = if model.is_null() {
        "llama3".to_string()
    } else {
        unsafe { CStr::from_ptr(model) }.to_string_lossy().into_owned()
    };

    let provider = aether_ai::ollama(&model_str);
    let handle = Box::new(AetherProvider {
        inner: Arc::new(provider),
    });
    Box::into_raw(handle)
}

/// Free a provider handle.
#[no_mangle]
pub extern "C" fn aether_free_provider(provider: *mut AetherProvider) {
    if !provider.is_null() {
        unsafe { drop(Box::from_raw(provider)) };
    }
}

// ============================================================
// Engine Creation
// ============================================================

/// Create an injection engine from a provider.
///
/// # Arguments
/// * `provider` - Provider handle (ownership is NOT transferred)
///
/// # Returns
/// Engine handle on success, NULL on failure.
#[no_mangle]
pub extern "C" fn aether_create_engine(provider: *const AetherProvider) -> *mut AetherEngine {
    if provider.is_null() {
        set_last_error("Provider is null".to_string());
        return ptr::null_mut();
    }

    let provider_ref = unsafe { &*provider };
    let provider_arc = provider_ref.inner.clone();
    let engine = InjectionEngine::new(provider_arc.clone());

    let handle = Box::new(AetherEngine {
        inner: engine,
        provider: provider_arc,
        healing_enabled: false,
        cache_enabled: false,
        toon_enabled: false,
        max_retries: 0,
    });
    Box::into_raw(handle)
}

/// Free an engine handle.
#[no_mangle]
pub extern "C" fn aether_free_engine(engine: *mut AetherEngine) {
    if !engine.is_null() {
        unsafe { drop(Box::from_raw(engine)) };
    }
}

/// Enable Self-Healing on the engine.
/// When enabled, generated code is validated and regenerated on errors.
///
/// # Arguments
/// * `engine` - Engine handle (must be mutable)
///
/// # Returns
/// true on success, false on failure
#[no_mangle]
pub extern "C" fn aether_engine_enable_healing(engine: *mut AetherEngine) -> bool {
    if engine.is_null() {
        set_last_error("Engine is null".to_string());
        return false;
    }

    let engine_ref = unsafe { &mut *engine };
    engine_ref.healing_enabled = true;
    engine_ref.rebuild();
    true
}

/// Enable Semantic Caching on the engine.
/// Reduces API costs by caching similar prompts.
///
/// # Arguments
/// * `engine` - Engine handle (must be mutable)
///
/// # Returns
/// true on success, false on failure
#[no_mangle]
pub extern "C" fn aether_engine_enable_cache(engine: *mut AetherEngine) -> bool {
    if engine.is_null() {
        set_last_error("Engine is null".to_string());
        return false;
    }

    let engine_ref = unsafe { &mut *engine };
    engine_ref.cache_enabled = true;
    engine_ref.rebuild();
    true
}

/// Enable TOON Protocol on the engine.
/// Compresses context for token efficiency.
///
/// # Arguments
/// * `engine` - Engine handle (must be mutable)
/// * `enabled` - Whether to enable TOON
#[no_mangle]
pub extern "C" fn aether_engine_set_toon(engine: *mut AetherEngine, enabled: bool) {
    if engine.is_null() {
        return;
    }

    let engine_ref = unsafe { &mut *engine };
    engine_ref.toon_enabled = enabled;
    engine_ref.rebuild();
}

/// Set the maximum retry count for Self-Healing.
///
/// # Arguments
/// * `engine` - Engine handle
/// * `max_retries` - Maximum number of healing attempts (default: 3)
#[no_mangle]
pub extern "C" fn aether_engine_set_max_retries(engine: *mut AetherEngine, max_retries: u32) {
    if engine.is_null() {
        return;
    }

    let engine_ref = unsafe { &mut *engine };
    engine_ref.max_retries = max_retries as usize;
    engine_ref.rebuild();
}


// ============================================================
// Template Operations
// ============================================================

/// Create a template from content string.
///
/// # Arguments
/// * `content` - Template content with `{{AI:slot}}` markers
///
/// # Returns
/// Template handle on success, NULL on failure.
#[no_mangle]
pub extern "C" fn aether_create_template(content: *const c_char) -> *mut AetherTemplate {
    if content.is_null() {
        set_last_error("Content is null".to_string());
        return ptr::null_mut();
    }

    let content_str = unsafe { CStr::from_ptr(content) }.to_string_lossy().into_owned();
    let template = Template::new(content_str);

    let handle = Box::new(AetherTemplate { inner: template });
    Box::into_raw(handle)
}

/// Add a slot to a template.
///
/// # Arguments
/// * `template` - Template handle
/// * `name` - Slot name
/// * `prompt` - AI prompt for this slot
#[no_mangle]
pub extern "C" fn aether_template_add_slot(
    template: *mut AetherTemplate,
    name: *const c_char,
    prompt: *const c_char,
) {
    if template.is_null() || name.is_null() || prompt.is_null() {
        return;
    }

    let template_ref = unsafe { &mut *template };
    let name_str = unsafe { CStr::from_ptr(name) }.to_string_lossy().into_owned();
    let prompt_str = unsafe { CStr::from_ptr(prompt) }.to_string_lossy().into_owned();

    template_ref.inner = template_ref.inner.clone().with_slot(name_str, prompt_str);
}

/// Free a template handle.
#[no_mangle]
pub extern "C" fn aether_free_template(template: *mut AetherTemplate) {
    if !template.is_null() {
        unsafe { drop(Box::from_raw(template)) };
    }
}

// ============================================================
// Rendering
// ============================================================

/// Render a template using the engine.
///
/// # Arguments
/// * `engine` - Engine handle
/// * `template` - Template handle
///
/// # Returns
/// Newly allocated string with the result. Caller must free with `aether_free_string()`.
/// Returns NULL on error. Check `aether_last_error()`.
#[no_mangle]
pub extern "C" fn aether_render(
    engine: *const AetherEngine,
    template: *const AetherTemplate,
) -> *mut c_char {
    if engine.is_null() || template.is_null() {
        set_last_error("Engine or template is null".to_string());
        return ptr::null_mut();
    }

    let engine_ref = unsafe { &*engine };
    let template_ref = unsafe { &*template };

    match RUNTIME.block_on(engine_ref.inner.render(&template_ref.inner)) {
        Ok(result) => {
            match CString::new(result) {
                Ok(cstr) => cstr.into_raw(),
                Err(e) => {
                    set_last_error(format!("Invalid result string: {}", e));
                    ptr::null_mut()
                }
            }
        }
        Err(e) => {
            set_last_error(e.to_string());
            ptr::null_mut()
        }
    }
}

/// One-shot code generation (convenience function).
///
/// # Arguments
/// * `provider` - Provider handle
/// * `prompt` - The prompt for code generation
///
/// # Returns
/// Newly allocated string with generated code. Free with `aether_free_string()`.
#[no_mangle]
pub extern "C" fn aether_generate(
    provider: *const AetherProvider,
    prompt: *const c_char,
) -> *mut c_char {
    if provider.is_null() || prompt.is_null() {
        set_last_error("Provider or prompt is null".to_string());
        return ptr::null_mut();
    }

    let provider_ref = unsafe { &*provider };
    let prompt_str = unsafe { CStr::from_ptr(prompt) }.to_string_lossy().into_owned();

    let engine = InjectionEngine::new(provider_ref.inner.clone());
    let template = Template::new("{{AI:gen}}").with_slot("gen", prompt_str);

    match RUNTIME.block_on(engine.render(&template)) {
        Ok(result) => {
            match CString::new(result) {
                Ok(cstr) => cstr.into_raw(),
                Err(e) => {
                    set_last_error(format!("Invalid result: {}", e));
                    ptr::null_mut()
                }
            }
        }
        Err(e) => {
            set_last_error(e.to_string());
            ptr::null_mut()
        }
    }
}

// ============================================================
// Memory Management
// ============================================================

/// Free a string allocated by Aether.
#[no_mangle]
pub extern "C" fn aether_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)) };
    }
}

// ============================================================
// Version Info
// ============================================================

/// Get the Aether version string.
#[no_mangle]
pub extern "C" fn aether_version() -> *const c_char {
    static VERSION: Lazy<CString> = Lazy::new(|| {
        CString::new(env!("CARGO_PKG_VERSION")).unwrap()
    });
    VERSION.as_ptr()
}
