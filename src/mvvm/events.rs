//! # Event System for MVVM Architecture
//!
//! Clean event-driven communication between MVVM components.
//! Events decouple components and enable reactive programming patterns.

use crossterm::event::KeyEvent;

/// Logical position in text content (line and column)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LogicalPosition {
    pub line: usize,
    pub column: usize,
}

impl LogicalPosition {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn zero() -> Self {
        Self::new(0, 0)
    }
}

/// Range in logical coordinates for text operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicalRange {
    pub start: LogicalPosition,
    pub end: LogicalPosition,
}

impl LogicalRange {
    pub fn new(start: LogicalPosition, end: LogicalPosition) -> Self {
        Self { start, end }
    }

    pub fn single_char(position: LogicalPosition) -> Self {
        Self {
            start: position,
            end: LogicalPosition::new(position.line, position.column + 1),
        }
    }
}

/// Which pane is currently active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pane {
    Request,
    Response,
}

/// Editor mode (vim-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Normal,
    Insert,
    Command,
}

/// Events emitted when models change
#[derive(Debug, Clone, PartialEq)]
pub enum ModelEvent {
    /// Cursor moved in a pane
    CursorMoved {
        pane: Pane,
        old_pos: LogicalPosition,
        new_pos: LogicalPosition,
    },

    /// Text was inserted
    TextInserted {
        pane: Pane,
        position: LogicalPosition,
        text: String,
    },

    /// Text was deleted
    TextDeleted { pane: Pane, range: LogicalRange },

    /// Editor mode changed
    ModeChanged {
        old_mode: EditorMode,
        new_mode: EditorMode,
    },

    /// Active pane switched
    PaneSwitched { old_pane: Pane, new_pane: Pane },

    /// HTTP request was executed
    RequestExecuted { method: String, url: String },

    /// HTTP response received
    ResponseReceived { status_code: u16, body: String },
}

/// Events emitted when view updates are needed
#[derive(Debug, Clone, PartialEq)]
pub enum ViewEvent {
    /// Full screen redraw required
    FullRedrawRequired,

    /// Specific pane needs redrawing
    PaneRedrawRequired { pane: Pane },

    /// Status bar needs updating
    StatusBarUpdateRequired,

    /// Cursor position needs updating
    CursorUpdateRequired { pane: Pane },

    /// Scroll position changed
    ScrollChanged {
        pane: Pane,
        old_offset: usize,
        new_offset: usize,
    },
}

/// Input events from user or system
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    /// Key pressed
    KeyPressed(KeyEvent),

    /// Terminal resized
    TerminalResized { width: u16, height: u16 },
}

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
    use std::sync::{Arc, Mutex};

    #[test]
    fn logical_position_should_create_correctly() {
        let pos = LogicalPosition::new(5, 10);
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 10);
    }

    #[test]
    fn logical_position_zero_should_be_origin() {
        let pos = LogicalPosition::zero();
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn logical_range_should_create_correctly() {
        let start = LogicalPosition::new(1, 5);
        let end = LogicalPosition::new(2, 3);
        let range = LogicalRange::new(start, end);

        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }

    #[test]
    fn event_bus_should_deliver_model_events() {
        let mut bus = SimpleEventBus::new();
        let received_events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = Arc::clone(&received_events);

        bus.subscribe_to_model_events(Box::new(move |event| {
            events_clone.lock().unwrap().push(event.clone());
        }));

        let test_event = ModelEvent::CursorMoved {
            pane: Pane::Request,
            old_pos: LogicalPosition::zero(),
            new_pos: LogicalPosition::new(1, 0),
        };

        bus.publish_model_event(test_event.clone());

        let events = received_events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], test_event);
    }

    #[test]
    fn event_bus_should_deliver_view_events() {
        let mut bus = SimpleEventBus::new();
        let received_events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = Arc::clone(&received_events);

        bus.subscribe_to_view_events(Box::new(move |event| {
            events_clone.lock().unwrap().push(event.clone());
        }));

        let test_event = ViewEvent::FullRedrawRequired;
        bus.publish_view_event(test_event.clone());

        let events = received_events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], test_event);
    }
}
