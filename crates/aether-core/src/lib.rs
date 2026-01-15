//! # Aether Core
//!
//! Core library for AI-powered dynamic code injection framework.
//!
//! This crate provides the foundational components for template management,
//! code injection, and transformation logic.
//!
//! ## Features
//!
//! - Template parsing and management
//! - Slot-based code injection
//! - Dynamic code transformation
//! - Extensible provider trait for AI backends
//!
//! ## Example
//!
//! ```rust,ignore
//! use aether_core::{Template, Slot, InjectionContext};
//!
//! let template = Template::new("{{AI:generate_button}}")
//!     .with_slot("generate_button", "Create a submit button");
//!
//! let result = template.render_with_ai(&provider).await?;
//! ```

pub mod error;
pub mod template;
pub mod slot;
pub mod provider;
pub mod context;
pub mod engine;
pub mod validation;
pub mod cache;
pub mod toon;
pub mod runtime;
pub mod observer;
pub mod shield;
pub mod config;
pub mod script;

pub use error::{AetherError, Result};
pub use template::Template;
pub use slot::{Slot, SlotKind, SlotConstraints};
pub use provider::{AiProvider, ProviderConfig};
pub use context::InjectionContext;
pub use engine::{InjectionEngine, RenderSession};
pub use script::{AetherScript, AetherAgenticRuntime};
pub use runtime::AetherRuntime;
pub use config::AetherConfig;
pub use cache::{Cache, ExactCache, SemanticCache, TieredCache};
pub use observer::{EngineObserver, ObserverPtr};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        Template, Slot, SlotKind, SlotConstraints,
        AiProvider, ProviderConfig,
        InjectionContext, InjectionEngine, RenderSession,
        AetherScript, AetherAgenticRuntime,
        AetherError, Result,
    };
}
