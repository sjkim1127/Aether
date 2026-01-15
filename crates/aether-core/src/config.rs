//! # Aether Configuration
//! 
//! Central configuration management for the Aether framework.
//! Supports loading from environment variables, files, and programmatic defaults.

use std::env;

/// Global configuration for the Aether engine.
/// 
/// # Example
/// ```rust
/// use aether_core::AetherConfig;
/// 
/// // Load from environment
/// let config = AetherConfig::from_env();
/// 
/// // Or customize
/// let config = AetherConfig::default()
///     .with_toon(true)
///     .with_healing(true);
/// ```
#[derive(Debug, Clone)]
pub struct AetherConfig {
    /// Enable TOON (Token-Oriented Object Notation) for context compression.
    /// Reduces token usage by 30-60% for structured data.
    /// Default: false, Env: AETHER_TOON=true
    pub toon_enabled: bool,

    /// Enable Self-Healing mode (automatic validation and retry on errors).
    /// Default: false, Env: AETHER_HEALING=true
    pub healing_enabled: bool,

    /// Enable Semantic Cache (reduces API costs for similar prompts).
    /// Default: false, Env: AETHER_CACHE=true
    pub cache_enabled: bool,

    /// Enable parallel slot generation.
    /// Default: true, Env: AETHER_PARALLEL=false
    pub parallel: bool,

    /// Whether to enable the Aether Inspector UI.
    /// Default: false, Env: AETHER_INSPECT=true
    pub inspector_enabled: bool,

    /// Port for the Aether Inspector UI.
    /// Default: 3000, Env: AETHER_INSPECT_PORT=8080
    pub inspector_port: u16,

    /// Maximum retries for failed generations.
    /// Default: 2, Env: AETHER_MAX_RETRIES=3
    pub max_retries: u32,

    /// Auto-enable TOON when context exceeds this character count.
    /// If None, TOON is only enabled manually.
    /// Default: Some(2000), Env: AETHER_TOON_THRESHOLD=2000
    pub auto_toon_threshold: Option<usize>,

    /// Cache similarity threshold (0.0 - 1.0).
    /// Higher values require more similar prompts to hit the cache.
    /// Default: 0.90, Env: AETHER_CACHE_THRESHOLD=0.90
    pub cache_threshold: f32,

    /// Prompt header for TOON context block.
    pub prompt_toon_header: String,

    /// Instructional note for the AI about TOON protocol.
    pub prompt_toon_note: String,

    /// Feedback prefix for self-healing retries.
    pub prompt_healing_feedback: String,

    /// Notice added when TDD mode is active.
    pub prompt_tdd_notice: String,

    /// Base delay for retry backoff in milliseconds.
    pub retry_backoff_ms: u64,
}

impl Default for AetherConfig {
    fn default() -> Self {
        Self {
            toon_enabled: false,
            healing_enabled: false,
            cache_enabled: false,
            parallel: true,
            inspector_enabled: false,
            inspector_port: 3000,
            max_retries: 2,
            auto_toon_threshold: Some(2000),
            cache_threshold: 0.90,
            prompt_toon_header: "[CONTEXT:TOON]".to_string(),
            prompt_toon_note: "[TOON Protocol Note]\nTOON is a compact key:value mapping protocol. Each line represents 'key: value'. Use this context to inform your code generation, respecting the framework, language, and architectural constraints defined within.".to_string(),
            prompt_healing_feedback: "[SELF-HEALING FEEDBACK]\nYour previous output had validation errors. Please fix them and output ONLY the corrected code.\nERROR:\n".to_string(),
            prompt_tdd_notice: "\n\nIMPORTANT: The system is running in TDD (Test-Driven Development) mode. Your code will be validated against compiler checks and functional tests. If possible, include unit tests in your response to help self-verify. If validation fails, you will receive feedback to fix the code.".to_string(),
            retry_backoff_ms: 100,
        }
    }
}

impl AetherConfig {
    /// Create a new config from environment variables.
    /// Falls back to defaults for missing variables.
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(v) = env::var("AETHER_TOON") {
            config.toon_enabled = v.to_lowercase() == "true" || v == "1";
        }
        if let Ok(v) = env::var("AETHER_HEALING") {
            config.healing_enabled = v.to_lowercase() == "true" || v == "1";
        }
        if let Ok(v) = env::var("AETHER_CACHE") {
            config.cache_enabled = v.to_lowercase() == "true" || v == "1";
        }
        if let Ok(v) = env::var("AETHER_PARALLEL") {
            config.parallel = v.to_lowercase() != "false" && v != "0";
        }
        if let Ok(v) = env::var("AETHER_INSPECT") {
            config.inspector_enabled = v.to_lowercase() == "true" || v == "1";
        }
        if let Ok(v) = env::var("AETHER_INSPECT_PORT") {
            if let Ok(n) = v.parse() {
                config.inspector_port = n;
            }
        }
        if let Ok(v) = env::var("AETHER_MAX_RETRIES") {
            if let Ok(n) = v.parse() {
                config.max_retries = n;
            }
        }
        if let Ok(v) = env::var("AETHER_TOON_THRESHOLD") {
            if let Ok(n) = v.parse() {
                config.auto_toon_threshold = Some(n);
            }
        }
        if let Ok(v) = env::var("AETHER_CACHE_THRESHOLD") {
            if let Ok(n) = v.parse() {
                config.cache_threshold = n;
            }
        }
        if let Ok(v) = env::var("AETHER_PROMPT_TOON_HEADER") {
            config.prompt_toon_header = v;
        }
        if let Ok(v) = env::var("AETHER_PROMPT_TOON_NOTE") {
            config.prompt_toon_note = v;
        }
        if let Ok(v) = env::var("AETHER_PROMPT_HEALING_FEEDBACK") {
            config.prompt_healing_feedback = v;
        }
        if let Ok(v) = env::var("AETHER_PROMPT_TDD_NOTICE") {
            config.prompt_tdd_notice = v;
        }
        if let Ok(v) = env::var("AETHER_RETRY_BACKOFF") {
            if let Ok(n) = v.parse() {
                config.retry_backoff_ms = n;
            }
        }

        config
    }

    /// Builder: Enable or disable TOON protocol.
    pub fn with_toon(mut self, enabled: bool) -> Self {
        self.toon_enabled = enabled;
        self
    }

    /// Builder: Enable or disable Self-Healing.
    pub fn with_healing(mut self, enabled: bool) -> Self {
        self.healing_enabled = enabled;
        self
    }

    /// Builder: Enable or disable Semantic Cache.
    pub fn with_cache(mut self, enabled: bool) -> Self {
        self.cache_enabled = enabled;
        self
    }

    /// Builder: Enable or disable parallel generation.
    pub fn with_parallel(mut self, enabled: bool) -> Self {
        self.parallel = enabled;
        self
    }

    /// Builder: Enable or disable Aether Inspector.
    pub fn with_inspector(mut self, enabled: bool) -> Self {
        self.inspector_enabled = enabled;
        self
    }

    /// Builder: Set Aether Inspector port.
    pub fn with_inspector_port(mut self, port: u16) -> Self {
        self.inspector_port = port;
        self
    }

    /// Builder: Set maximum retries.
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Builder: Set auto TOON threshold.
    pub fn with_auto_toon_threshold(mut self, threshold: Option<usize>) -> Self {
        self.auto_toon_threshold = threshold;
        self
    }

    /// Check if TOON should be used for a given context length.
    pub fn should_use_toon(&self, context_length: usize) -> bool {
        if self.toon_enabled {
            return true;
        }
        if let Some(threshold) = self.auto_toon_threshold {
            return context_length >= threshold;
        }
        false
    }

    /// Create a recommended default cache for the engine.
    /// Returns a `TieredCache` (Hybrid Exact + Semantic).
    pub fn default_cache(&self) -> crate::Result<crate::cache::TieredCache> {
        crate::cache::TieredCache::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AetherConfig::default();
        assert!(!config.toon_enabled);
        assert!(!config.healing_enabled);
        assert!(config.parallel);
        assert_eq!(config.max_retries, 2);
    }

    #[test]
    fn test_builder_pattern() {
        let config = AetherConfig::default()
            .with_toon(true)
            .with_healing(true)
            .with_max_retries(5);

        assert!(config.toon_enabled);
        assert!(config.healing_enabled);
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_auto_toon() {
        let config = AetherConfig::default();
        assert!(!config.should_use_toon(1000)); // Below threshold
        assert!(config.should_use_toon(3000));  // Above threshold
    }
}
