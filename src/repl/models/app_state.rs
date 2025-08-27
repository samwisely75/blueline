//! # Application State
//!
//! Core application state that represents the entire application's data model.
//! This is pure data with no business logic - all logic belongs in AppViewModel.
//!
//! This follows the MVVM pattern where:
//! - Models contain pure data structures
//! - ViewModels contain business logic and state management
//! - Views handle presentation and user interaction

use crate::repl::models::events::{EventBus, ModelEvent, ViewEvent};
use crate::repl::models::{MemoryYankBuffer, PaneState, ResponseModel, StatusLine, YankBuffer};
use std::collections::HashMap;

/// Type alias for event bus option to reduce complexity
type EventBusOption = Option<Box<dyn EventBus>>;

/// Core application state containing all data models
///
/// This struct contains ONLY data - no business logic.
/// All operations and logic should be in AppViewModel.
pub struct AppState {
    // Core models
    pub response: ResponseModel,

    // Pane states
    pub request_pane: PaneState,
    pub response_pane: PaneState,
    pub active_pane: crate::repl::models::pane_state::Pane,

    // Status line model
    pub status_line: StatusLine,

    // HTTP session configuration
    pub http_session_headers: HashMap<String, String>,

    // Event management
    pub event_bus: EventBusOption,
    pub pending_view_events: Vec<ViewEvent>,
    pub pending_model_events: Vec<ModelEvent>,

    // Yank buffer for copy/paste operations
    pub yank_buffer: Box<dyn YankBuffer>,

    // Configuration flags
    pub clipboard_enabled: bool,
    pub dcut_enabled: bool,
}

impl AppState {
    /// Create a new AppState with default values
    pub fn new() -> Self {
        use crate::repl::models::pane_state::{Pane, PaneCapabilities};

        // Default terminal size
        let (width, height) = (80, 24);

        Self {
            response: ResponseModel::new(),
            request_pane: PaneState::new(
                Pane::Request,
                width,
                height,
                false,
                PaneCapabilities::FULL_ACCESS,
            ),
            response_pane: PaneState::new(
                Pane::Response,
                width,
                height,
                false,
                PaneCapabilities::READ_ONLY | PaneCapabilities::SELECTABLE,
            ),
            active_pane: Pane::Request,
            status_line: StatusLine::new(),
            http_session_headers: HashMap::new(),
            event_bus: None,
            pending_view_events: Vec::new(),
            pending_model_events: Vec::new(),
            yank_buffer: Box::new(MemoryYankBuffer::new()),
            clipboard_enabled: false,
            dcut_enabled: true,
        }
    }

    /// Set the event bus for this AppState
    pub fn set_event_bus(&mut self, event_bus: Box<dyn EventBus>) {
        self.event_bus = Some(event_bus);
        tracing::debug!("Event bus set for AppState");
    }

    /// Get the active pane state
    pub fn active_pane_state(&self) -> &PaneState {
        match self.active_pane {
            crate::repl::models::pane_state::Pane::Request => &self.request_pane,
            crate::repl::models::pane_state::Pane::Response => &self.response_pane,
        }
    }

    /// Get the active pane state mutably
    pub fn active_pane_state_mut(&mut self) -> &mut PaneState {
        match self.active_pane {
            crate::repl::models::pane_state::Pane::Request => &mut self.request_pane,
            crate::repl::models::pane_state::Pane::Response => &mut self.response_pane,
        }
    }

    /// Get pane state by pane type
    pub fn get_pane_state(&self, pane: crate::repl::models::pane_state::Pane) -> &PaneState {
        match pane {
            crate::repl::models::pane_state::Pane::Request => &self.request_pane,
            crate::repl::models::pane_state::Pane::Response => &self.response_pane,
        }
    }

    /// Get pane state mutably by pane type
    pub fn get_pane_state_mut(
        &mut self,
        pane: crate::repl::models::pane_state::Pane,
    ) -> &mut PaneState {
        match pane {
            crate::repl::models::pane_state::Pane::Request => &mut self.request_pane,
            crate::repl::models::pane_state::Pane::Response => &mut self.response_pane,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
