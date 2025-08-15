//! # Events Module
//!
//! Re-exports all event system components organized by category.
//! This module maintains the same public API while organizing events
//! into logical groups for better maintainability.

// Import event modules
pub mod event_bus;
pub mod event_source;
pub mod model_events;
pub mod terminal_event_source;
pub mod types;
pub mod view_events;

// Re-export all types for easy access
pub use event_bus::{EventBus, ModelEventHandler, SimpleEventBus, ViewEventHandler};
pub use event_source::EventSource;
pub use model_events::ModelEvent;
pub use terminal_event_source::TerminalEventSource;
pub use types::{EditorMode, LogicalPosition, LogicalRange, Pane, PaneCapabilities};
pub use view_events::{InputEvent, ViewEvent};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn all_event_types_should_be_accessible() {
        // Test that all main event types can be imported and used
        let _pos = LogicalPosition::zero();
        let _range = LogicalRange::single_char(LogicalPosition::new(1, 2));
        let _pane = Pane::Request;
        let _mode = EditorMode::Normal;

        let _model_event = ModelEvent::ModeChanged {
            old_mode: EditorMode::Normal,
            new_mode: EditorMode::Insert,
        };

        let _view_event = ViewEvent::FullRedrawRequired;
        let _input_event = InputEvent::TerminalResized {
            width: 80,
            height: 24,
        };

        let _bus = SimpleEventBus::new();
    }

    #[test]
    fn event_bus_integration_should_work() {
        let mut bus = SimpleEventBus::new();
        let received = Arc::new(Mutex::new(false));
        let received_clone = received.clone();

        bus.subscribe_to_model_events(Box::new(move |_| {
            *received_clone.lock().unwrap() = true;
        }));

        let event = ModelEvent::PaneSwitched {
            old_pane: Pane::Request,
            new_pane: Pane::Response,
        };
        bus.publish_model_event(event);

        assert!(*received.lock().unwrap());
    }

    #[test]
    fn types_should_have_consistent_behavior() {
        let pos1 = LogicalPosition::new(1, 2);
        let pos2 = LogicalPosition::new(1, 2);
        let pos3 = LogicalPosition::new(2, 3);

        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);

        let range = LogicalRange::new(pos1, pos3);
        assert_eq!(range.start, pos1);
        assert_eq!(range.end, pos3);
    }
}
