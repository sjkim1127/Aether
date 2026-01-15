//! # Aether Script
//! 
//! Aether Script (.ae) is a specialized DSL built on top of Rhai,
//! optimized for AI-agentic workflows. It introduces first-class 
//! AI directives and data-flow operators.

use crate::{Result, AetherError, AiProvider, InjectionEngine};
use rhai::{Engine, Dynamic, Scope};
use std::sync::Arc;
use tracing::debug;

/// Pre-processor for Aether Script syntax.
pub struct AetherScript;

impl AetherScript {
    /// Compiles Aether Script syntax into valid Rhai code.
    /// 
    /// Supported Directives:
    /// - `@ai("prompt")` -> Shortcut for AI generation.
    /// - `@json { ... }` -> Auto-decoding JSON responses.
    pub fn preprocess(script: &str) -> String {
        let mut processed = script.to_string();

        // Placeholder for regex-based transformation
        // In a production system, this would be a proper lexer/parser
        
        // Example: Transform @ai("prompt") -> __aether_ask("prompt")
        // We use a simple replacement for now to demonstrate the concept
        processed = processed.replace("@ai", "__aether_ask");
        
        processed
    }
}

/// Aether-enhanced runtime that supports agentic functions.
pub struct AetherAgenticRuntime<P: AiProvider> {
    engine: Engine,
    _provider: Arc<P>,
}

impl<P: AiProvider + 'static> AetherAgenticRuntime<P> {
    /// Create a new agentic runtime with the given AI provider.
    pub fn new(provider: P) -> Self {
        let mut engine = Engine::new();
        let provider = Arc::new(provider);
        let p_clone = Arc::clone(&provider);

        // Register __aether_ask function
        // This allows scripts to call AI directly!
        engine.register_fn("__aether_ask", move |prompt: String| -> Dynamic {
            let p = Arc::clone(&p_clone);
            
            // SAFETY: Rhai is synchronous, but InjectionEngine is asynchronous.
            // Calling block_on directly within a Tokio runtime causes a panic.
            // We spawn a separate thread and a dedicated single-threaded runtime 
            // to safely bridge the sync/async gap.
            let result = std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| AetherError::InjectionError(e.to_string()))?;
                
                rt.block_on(async move {
                    let engine = InjectionEngine::new_raw(p);
                    engine.inject_raw(&prompt).await
                })
            }).join();

            match result {
                Ok(Ok(code)) => Dynamic::from(code),
                Ok(Err(e)) => Dynamic::from(format!("Error: {}", e)),
                Err(_) => Dynamic::from("Error: AI thread panicked".to_string()),
            }
        });

        Self { engine, _provider: provider }
    }

    /// Execute an Aether Script.
    pub fn execute(&self, script: &str, scope: &mut Scope) -> Result<Dynamic> {
        let processed = AetherScript::preprocess(script);
        debug!("Executing preprocessed script: {}", processed);
        
        self.engine.eval_with_scope(scope, &processed)
            .map_err(|e| AetherError::ConfigError(format!("Script execution failed: {}", e)))
    }
}
