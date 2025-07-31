//! # Event Bus
//!
//! Central event distribution system for decoupled communication
//! between REPL components using the observer pattern.

use super::model_events::ModelEvent;
use super::view_events::ViewEvent;

/// Type alias for model event handlers to reduce complexity
pub type ModelEventHandler = Box<dyn Fn(&ModelEvent) + Send + Sync>;

/// Type alias for view event handlers to reduce complexity
pub type ViewEventHandler = Box<dyn Fn(&ViewEvent) + Send + Sync>;

/// Event bus for decoupled communication between components
pub trait EventBus: Send + Sync {
    /// Publish a model event
    fn publish_model_event(&mut self, event: ModelEvent);

    /// Publish a view event
    fn publish_view_event(&mut self, event: ViewEvent);

    /// Subscribe to model events
    fn subscribe_to_model_events(&mut self, handler: ModelEventHandler);

    /// Subscribe to view events
    fn subscribe_to_view_events(&mut self, handler: ViewEventHandler);
}

/// Simple in-memory event bus implementation
pub struct SimpleEventBus {
    model_handlers: Vec<ModelEventHandler>,
    view_handlers: Vec<ViewEventHandler>,
}

impl SimpleEventBus {
    pub fn new() -> Self {
        Self {
            model_handlers: Vec::new(),
            view_handlers: Vec::new(),
        }
    }
}

impl Default for SimpleEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus for SimpleEventBus {
    fn publish_model_event(&mut self, event: ModelEvent) {
        for handler in &self.model_handlers {
            handler(&event);
        }
    }

    fn publish_view_event(&mut self, event: ViewEvent) {
        for handler in &self.view_handlers {
            handler(&event);
        }
    }

    fn subscribe_to_model_events(&mut self, handler: ModelEventHandler) {
        self.model_handlers.push(handler);
    }

    fn subscribe_to_view_events(&mut self, handler: ViewEventHandler) {
        self.view_handlers.push(handler);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::events::types::{EditorMode, LogicalPosition, Pane};
    use std::sync::{Arc, Mutex};

    #[test]
    fn event_bus_should_deliver_model_events() {
        let mut bus = SimpleEventBus::new();
        let received_events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = received_events.clone();

        bus.subscribe_to_model_events(Box::new(move |event| {
            events_clone.lock().unwrap().push(event.clone());
        }));

        let event = ModelEvent::ModeChanged {
            old_mode: EditorMode::Normal,
            new_mode: EditorMode::Insert,
        };
        bus.publish_model_event(event.clone());

        let received = received_events.lock().unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0], event);
    }

    #[test]
    fn event_bus_should_deliver_view_events() {
        let mut bus = SimpleEventBus::new();
        let received_events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = received_events.clone();

        bus.subscribe_to_view_events(Box::new(move |event| {
            events_clone.lock().unwrap().push(event.clone());
        }));

        let event = ViewEvent::CurrentAreaRedrawRequired;
        bus.publish_view_event(event.clone());

        let received = received_events.lock().unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0], event);
    }

    #[test]
    fn event_bus_should_handle_multiple_subscribers() {
        let mut bus = SimpleEventBus::new();
        let received_events_1 = Arc::new(Mutex::new(Vec::new()));
        let received_events_2 = Arc::new(Mutex::new(Vec::new()));
        let events_clone_1 = received_events_1.clone();
        let events_clone_2 = received_events_2.clone();

        bus.subscribe_to_model_events(Box::new(move |event| {
            events_clone_1.lock().unwrap().push(event.clone());
        }));

        bus.subscribe_to_model_events(Box::new(move |event| {
            events_clone_2.lock().unwrap().push(event.clone());
        }));

        let event = ModelEvent::CursorMoved {
            pane: Pane::Request,
            old_pos: LogicalPosition::zero(),
            new_pos: LogicalPosition::new(1, 2),
        };
        bus.publish_model_event(event.clone());

        let received_1 = received_events_1.lock().unwrap();
        let received_2 = received_events_2.lock().unwrap();
        assert_eq!(received_1.len(), 1);
        assert_eq!(received_2.len(), 1);
        assert_eq!(received_1[0], event);
        assert_eq!(received_2[0], event);
    }
}
