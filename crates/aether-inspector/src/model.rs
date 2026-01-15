use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use dashmap::DashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventStatus {
    Starting,
    Generating,
    Success,
    Healed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectorEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub template: String,
    pub slot: String,
    pub prompt: String,
    pub toon_payload: Option<String>,
    pub result: Option<String>,
    pub healing_attempts: u32,
    pub tokens_used: Option<u32>,
    pub status: EventStatus,
}

#[derive(Clone, Default)]
pub struct Inspector {
    pub events: Arc<DashMap<String, InspectorEvent>>,
}

impl Inspector {
    pub fn new() -> Self {
        Self {
            events: Arc::new(DashMap::new()),
        }
    }

    pub fn record(&self, event: InspectorEvent) {
        self.events.insert(event.id.clone(), event);
    }
}
