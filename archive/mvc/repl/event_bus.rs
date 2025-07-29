//! Simple event bus implementation for MVVM architecture
//!
//! This module provides a concrete implementation of the EventBus trait,
//! allowing components to communicate through events without direct coupling.

use super::events::{
    EventBus, ModelEvent, ModelEventHandler, ViewModelEvent, ViewModelEventHandler,
};

/// Simple in-memory event bus implementation
///
/// This event bus maintains lists of handlers for different event types
/// and dispatches events synchronously to all registered handlers.
/// This keeps the implementation simple while providing the decoupling needed for MVVM.
pub struct SimpleEventBus {
    model_handlers: Vec<ModelEventHandler>,
    view_handlers: Vec<ViewModelEventHandler>,
}

impl SimpleEventBus {
    /// Create a new empty event bus
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
    fn publish_model(&mut self, event: ModelEvent) {
        // Dispatch to all model event handlers
        for handler in &self.model_handlers {
            handler(&event);
        }
    }

    fn publish_view(&mut self, event: ViewModelEvent) {
        // Dispatch to all view model event handlers
        for handler in &self.view_handlers {
            handler(&event);
        }
    }

    fn subscribe_model(&mut self, handler: ModelEventHandler) {
        self.model_handlers.push(handler);
    }

    fn subscribe_view(&mut self, handler: ViewModelEventHandler) {
        self.view_handlers.push(handler);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::events::LogicalPosition;
    use crate::repl::model::{EditorMode, Pane};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn simple_event_bus_should_create_empty_bus() {
        let bus = SimpleEventBus::new();
        assert_eq!(bus.model_handlers.len(), 0);
        assert_eq!(bus.view_handlers.len(), 0);
    }

    #[test]
    fn simple_event_bus_should_publish_model_events_to_subscribers() {
        let mut bus = SimpleEventBus::new();
        let received_events = Rc::new(RefCell::new(Vec::new()));
        let received_events_clone = received_events.clone();

        bus.subscribe_model(Box::new(move |event| {
            received_events_clone.borrow_mut().push(event.clone());
        }));

        let event = ModelEvent::CursorMoved {
            pane: Pane::Request,
            old_pos: LogicalPosition { line: 0, column: 0 },
            new_pos: LogicalPosition { line: 1, column: 5 },
        };

        bus.publish_model(event.clone());

        let events = received_events.borrow();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event);
    }

    #[test]
    fn simple_event_bus_should_publish_view_events_to_subscribers() {
        let mut bus = SimpleEventBus::new();
        let received_events = Rc::new(RefCell::new(Vec::new()));
        let received_events_clone = received_events.clone();

        bus.subscribe_view(Box::new(move |event| {
            received_events_clone.borrow_mut().push(event.clone());
        }));

        let event = ViewModelEvent::CursorRepositionRequired;

        bus.publish_view(event.clone());

        let events = received_events.borrow();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event);
    }

    #[test]
    fn simple_event_bus_should_handle_multiple_subscribers() {
        let mut bus = SimpleEventBus::new();
        let first_events = Rc::new(RefCell::new(Vec::new()));
        let second_events = Rc::new(RefCell::new(Vec::new()));

        let first_clone = first_events.clone();
        let second_clone = second_events.clone();

        bus.subscribe_model(Box::new(move |event| {
            first_clone.borrow_mut().push(event.clone());
        }));

        bus.subscribe_model(Box::new(move |event| {
            second_clone.borrow_mut().push(event.clone());
        }));

        let event = ModelEvent::ModeChanged {
            from: EditorMode::Normal,
            to: EditorMode::Insert,
        };

        bus.publish_model(event.clone());

        assert_eq!(first_events.borrow().len(), 1);
        assert_eq!(second_events.borrow().len(), 1);
        assert_eq!(first_events.borrow()[0], event);
        assert_eq!(second_events.borrow()[0], event);
    }

    #[test]
    fn simple_event_bus_should_not_interfere_between_event_types() {
        let mut bus = SimpleEventBus::new();
        let model_events = Rc::new(RefCell::new(Vec::new()));
        let view_events = Rc::new(RefCell::new(Vec::new()));

        let model_clone = model_events.clone();
        let view_clone = view_events.clone();

        bus.subscribe_model(Box::new(move |event| {
            model_clone.borrow_mut().push(event.clone());
        }));

        bus.subscribe_view(Box::new(move |event| {
            view_clone.borrow_mut().push(event.clone());
        }));

        let model_event = ModelEvent::RequestExecuted;
        let view_event = ViewModelEvent::FullRedrawRequired;

        bus.publish_model(model_event.clone());
        bus.publish_view(view_event.clone());

        assert_eq!(model_events.borrow().len(), 1);
        assert_eq!(view_events.borrow().len(), 1);
        assert_eq!(model_events.borrow()[0], model_event);
        assert_eq!(view_events.borrow()[0], view_event);
    }
}
