//! Event models for the event-driven architecture

pub mod event_bus;
pub mod model_events;

// Re-export commonly used types
pub use event_bus::{EventBus, ModelEventHandler, PostCommandActionHandler, SimpleEventBus};
pub use model_events::ModelEvent;
