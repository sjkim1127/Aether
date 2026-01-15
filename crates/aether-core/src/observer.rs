use crate::provider::{GenerationRequest, GenerationResponse};
use std::sync::Arc;

/// Trait for observing engine events (logging, metrics, UI).
pub trait EngineObserver: Send + Sync {
    /// Called when a generation starts.
    fn on_start(&self, id: &str, template: &str, slot: &str, request: &GenerationRequest);
    
    /// Called when a generation succeeds.
    fn on_success(&self, id: &str, response: &GenerationResponse);
    
    /// Called when a validation/healing attempt occurs.
    fn on_healing_step(&self, id: &str, attempt: u32, error: &str);
    
    /// Called when a generation fails permanently.
    fn on_failure(&self, id: &str, error: &str);

    /// Called to report arbitrary metadata for an event.
    fn on_metadata(&self, _id: &str, _key: &str, _value: serde_json::Value) {}
}

pub type ObserverPtr = Arc<dyn EngineObserver>;
