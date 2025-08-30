//! Event models for the event-driven architecture

pub mod event_bus;
pub mod model_events;
pub mod view_events;

// Re-export commonly used types
pub use event_bus::{EventBus, ModelEventHandler, SimpleEventBus, ViewEventHandler};
pub use model_events::ModelEvent;
pub use view_events::{InputEvent, ViewEvent};
