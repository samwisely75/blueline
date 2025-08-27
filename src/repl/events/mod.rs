//! # Events Module
//!
//! Re-exports event system components from their new locations.
//! This module acts as a facade to maintain backward compatibility.

// Re-export from models/events
pub use crate::repl::models::events::event_bus::{
    EventBus, ModelEventHandler, SimpleEventBus, ViewEventHandler,
};
pub use crate::repl::models::events::model_events::ModelEvent;
pub use crate::repl::models::events::view_events::{InputEvent, ViewEvent};

// Re-export from io
pub use crate::repl::io::event_source::EventSource;
pub use crate::repl::io::terminal_event_source::TerminalEventSource;

// Re-export from models/pane_state
pub use crate::repl::models::pane_state::{
    EditorMode, LogicalPosition, LogicalRange, Pane, PaneCapabilities,
};

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
