//! # Application ViewModel
//!
//! Contains all business logic and state management operations.
//! This is the layer between the pure data models (AppState) and the controller.
//!
//! In the MVVM pattern:
//! - AppState (Model) contains pure data
//! - AppViewModel contains business logic and operations
//! - AppController handles user input and coordinates

use crate::repl::models::events::{EventBus, ViewEvent};
use crate::repl::models::pane_state::{EditorMode, Pane};
use crate::repl::models::LogicalPosition;
use crate::repl::models::{AppState, ClipboardYankBuffer};

/// Type alias for display line rendering data
pub type DisplayLineData = (String, Option<usize>, bool, usize, usize);

/// Application ViewModel containing all business logic
///
/// This struct wraps AppState and provides all the operations
/// and business logic needed by the application.
pub struct AppViewModel {
    /// The underlying application state (pure data)
    pub state: AppState,
}

impl AppViewModel {
    /// Create a new AppViewModel with default state
    pub fn new() -> Self {
        Self {
            state: AppState::new(),
        }
    }

    /// Set the event bus for this AppViewModel
    pub fn set_event_bus(&mut self, event_bus: Box<dyn EventBus>) {
        self.state.set_event_bus(event_bus);
    }

    /// Get the current editor mode from the active pane
    pub fn get_mode(&self) -> EditorMode {
        self.state.active_pane_state().editor_mode
    }

    /// Set the editor mode for the active pane
    pub fn set_mode(&mut self, mode: EditorMode) {
        self.state.active_pane_state_mut().editor_mode = mode;

        // Emit status bar update event when mode changes
        if self.state.event_bus.is_some() {
            self.state
                .pending_view_events
                .push(ViewEvent::StatusBarUpdateRequired);
        }
    }

    /// Get the active pane
    pub fn get_active_pane(&self) -> Pane {
        self.state.active_pane
    }

    /// Switch to a different pane
    pub fn switch_pane(&mut self, pane: Pane) {
        if self.state.active_pane != pane {
            self.state.active_pane = pane;

            // Emit focus switch event
            if self.state.event_bus.is_some() {
                self.state
                    .pending_view_events
                    .push(ViewEvent::FocusSwitched);
                self.state
                    .pending_view_events
                    .push(ViewEvent::StatusBarUpdateRequired);
            }
        }
    }

    /// Enable or disable clipboard integration
    pub fn set_clipboard_enabled(&mut self, enabled: bool) {
        if enabled != self.state.clipboard_enabled {
            self.state.clipboard_enabled = enabled;

            // Switch yank buffer implementation
            if enabled {
                // Try to create clipboard yank buffer, fall back to memory if it fails
                match ClipboardYankBuffer::new() {
                    Ok(clipboard_buffer) => {
                        self.state.yank_buffer = Box::new(clipboard_buffer);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to enable clipboard: {e}");
                        self.state.clipboard_enabled = false;
                        self.state.yank_buffer =
                            Box::new(crate::repl::models::MemoryYankBuffer::new());
                    }
                }
            } else {
                self.state.yank_buffer = Box::new(crate::repl::models::MemoryYankBuffer::new());
            }
        }
    }

    /// Get whether clipboard integration is enabled
    pub fn is_clipboard_enabled(&self) -> bool {
        self.state.clipboard_enabled
    }

    /// Set whether d/dd/D commands should cut (yank)
    pub fn set_dcut_enabled(&mut self, enabled: bool) {
        self.state.dcut_enabled = enabled;
    }

    /// Get whether d/dd/D commands should cut
    pub fn is_dcut_enabled(&self) -> bool {
        self.state.dcut_enabled
    }

    /// Update terminal dimensions
    pub fn update_dimensions(&mut self, width: u16, height: u16) {
        // Update pane dimensions
        self.state
            .request_pane
            .update_dimensions(width as usize, height as usize);
        self.state
            .response_pane
            .update_dimensions(width as usize, height as usize);

        // Emit full redraw event for resize
        if self.state.event_bus.is_some() {
            self.state
                .pending_view_events
                .push(ViewEvent::FullRedrawRequired);
        }
    }

    /// Get cursor position for the active pane
    pub fn get_cursor_position(&self) -> LogicalPosition {
        self.state.active_pane_state().buffer.cursor()
    }

    /// Set cursor position for the active pane
    pub fn set_cursor_position(&mut self, position: LogicalPosition) {
        self.state
            .active_pane_state_mut()
            .buffer
            .set_cursor(position);

        // Emit cursor update event
        if self.state.event_bus.is_some() {
            self.state
                .pending_view_events
                .push(ViewEvent::ActiveCursorUpdateRequired);
            self.state
                .pending_view_events
                .push(ViewEvent::PositionIndicatorUpdateRequired);
        }
    }

    /// Process pending events
    pub fn process_events(&mut self) {
        // Process events
        if let Some(ref mut event_bus) = self.state.event_bus {
            for event in self.state.pending_view_events.drain(..) {
                event_bus.publish_view_event(event);
            }

            // Process model events
            for event in self.state.pending_model_events.drain(..) {
                event_bus.publish_model_event(event);
            }
        }
    }

    // Additional business logic methods will be migrated here from ViewModel
    // This includes all the complex operations currently in ViewModel
}

impl Default for AppViewModel {
    fn default() -> Self {
        Self::new()
    }
}
