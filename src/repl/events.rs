//! Event system for MVVM architecture
//!
//! This module defines the event types and event bus infrastructure for the MVVM transition.
//! Events decouple components by allowing them to communicate without direct references,
//! preventing the view logic from leaking into commands and maintaining clear separation of concerns.

use crate::repl::model::{EditorMode, Pane};
use crossterm::event::KeyEvent;

/// Logical position in text content (line and column)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicalPosition {
    pub line: usize,
    pub column: usize,
}

/// Range in logical coordinates for text operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogicalRange {
    pub start: LogicalPosition,
    pub end: LogicalPosition,
}

/// Events emitted by the Model layer when state changes occur
///
/// These events represent pure business logic changes without any view concerns.
/// The ViewModel subscribes to these events to update display state accordingly.
#[derive(Debug, Clone, PartialEq)]
pub enum ModelEvent {
    /// Cursor position changed in logical coordinates
    CursorMoved { 
        pane: Pane, 
        old_pos: LogicalPosition, 
        new_pos: LogicalPosition 
    },
    
    /// Text was inserted at a specific position
    TextInserted { 
        pane: Pane, 
        position: LogicalPosition, 
        text: String 
    },
    
    /// Text was deleted from a range
    TextDeleted { 
        pane: Pane, 
        range: LogicalRange 
    },
    
    /// A new line was inserted
    LineInserted { 
        pane: Pane, 
        line: usize 
    },
    
    /// A line was deleted
    LineDeleted { 
        pane: Pane, 
        line: usize 
    },
    
    /// Editor mode changed (Normal, Insert, Command, etc.)
    ModeChanged { 
        from: EditorMode, 
        to: EditorMode 
    },
    
    /// Active pane switched
    PaneSwitched { 
        from: Pane, 
        to: Pane 
    },
    
    /// HTTP request was executed
    RequestExecuted,
    
    /// HTTP response was received
    ResponseReceived { 
        status: String, 
        body: String 
    },
}

/// Events emitted by the ViewModel when display state changes
///
/// These events notify the View about what needs to be re-rendered,
/// allowing for efficient partial updates instead of full screen redraws.
#[derive(Debug, Clone, PartialEq)]
pub enum ViewModelEvent {
    /// Display cache was updated for a pane
    DisplayCacheUpdated { pane: Pane },
    
    /// Scroll position changed in display coordinates
    ScrollPositionChanged { 
        pane: Pane, 
        old_offset: usize, 
        new_offset: usize 
    },
    
    /// Full screen redraw is required
    FullRedrawRequired,
    
    /// Specific pane needs redrawing
    PaneRedrawRequired { pane: Pane },
    
    /// Status bar needs updating
    StatusBarUpdateRequired,
    
    /// Cursor needs repositioning on screen
    CursorRepositionRequired,
}

/// Input events from the user or system
///
/// These events represent raw input that gets translated into commands,
/// decoupling input handling from command processing.
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    /// Key was pressed
    KeyPress(KeyEvent),
    
    /// Terminal was resized
    Resize { width: u16, height: u16 },
}

/// Event bus trait for publishing and subscribing to events
///
/// This trait allows different components to communicate through events
/// without direct coupling, enabling the MVVM architecture.
pub trait EventBus {
    /// Publish a model event to all subscribers
    fn publish_model(&mut self, event: ModelEvent);
    
    /// Publish a view model event to all subscribers  
    fn publish_view(&mut self, event: ViewModelEvent);
    
    /// Subscribe to model events with a callback
    fn subscribe_model(&mut self, handler: Box<dyn Fn(&ModelEvent)>);
    
    /// Subscribe to view model events with a callback
    fn subscribe_view(&mut self, handler: Box<dyn Fn(&ViewModelEvent)>);
}