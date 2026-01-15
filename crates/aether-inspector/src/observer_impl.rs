use aether_core::{EngineObserver, provider::{GenerationRequest, GenerationResponse}};
use crate::model::{Inspector, InspectorEvent, EventStatus};
use chrono::Utc;

impl EngineObserver for Inspector {
    fn on_start(&self, id: &str, template: &str, slot: &str, request: &GenerationRequest) {
        let event = InspectorEvent {
            id: id.to_string(),
            timestamp: Utc::now(),
            template: template.to_string(),
            slot: slot.to_string(),
            prompt: request.slot.prompt.clone(),
            toon_payload: request.context.as_ref().and_then(|c| {
                if c.contains("[CONTEXT:TOON]") {
                    Some(c.clone())
                } else {
                    None
                }
            }),
            result: None,
            healing_attempts: 0,
            tokens_used: None,
            status: EventStatus::Generating,
        };
        self.record(event);
    }

    fn on_success(&self, id: &str, response: &GenerationResponse) {
        if let Some(mut event) = self.events.get_mut(id) {
            event.result = Some(response.code.clone());
            event.tokens_used = response.tokens_used;
            event.status = EventStatus::Success;
        }
    }

    fn on_healing_step(&self, id: &str, attempt: u32, _error: &str) {
        if let Some(mut event) = self.events.get_mut(id) {
            event.healing_attempts = attempt;
            event.status = EventStatus::Healed;
        }
    }

    fn on_failure(&self, id: &str, error: &str) {
        if let Some(mut event) = self.events.get_mut(id) {
            event.status = EventStatus::Failed;
            event.result = Some(format!("Error: {}", error));
        }
    }
}
