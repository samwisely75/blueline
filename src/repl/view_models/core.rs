//! # Core ViewModel Structure
//!
//! Contains the main ViewModel struct and basic initialization logic.
//! This is the central coordinator that delegates to specialized managers.

use crate::repl::events::{EditorMode, EventBus, ModelEvent, Pane, ViewEvent};
use crate::repl::models::{ResponseModel, StatusLine};
use crate::repl::view_models::pane_state::PaneState;
use crate::repl::view_models::screen_buffer::ScreenBuffer;
// use anyhow::Result; // Currently unused
use bluenote::HttpClient;
use std::collections::HashMap;

/// Type alias for event bus option to reduce complexity
type EventBusOption = Option<Box<dyn EventBus>>;

/// Type alias for display line rendering data: (content, line_number, is_continuation, logical_start_col, logical_line)
pub type DisplayLineData = (String, Option<usize>, bool, usize, usize);



/// The central ViewModel that coordinates all business logic
pub struct ViewModel {
    // Core state
    pub(super) editor_mode: EditorMode,
    pub(super) current_pane: Pane,
    pub(super) response: ResponseModel,

    // Pane management - replaces 8 duplicate fields with unified structure
    pub(super) panes: [PaneState; 2],

    // Display coordination (word wrap support)
    pub(super) wrap_enabled: bool,

    // Display state
    pub(super) terminal_dimensions: (u16, u16), // (width, height)
    pub(super) request_pane_height: u16,

    // Status line model - encapsulates all status bar state
    pub(super) status_line: StatusLine,

    // HTTP client and configuration
    pub(super) http_client: Option<HttpClient>,
    pub(super) http_session_headers: HashMap<String, String>,
    pub(super) http_verbose: bool,

    // Event management
    pub(super) event_bus: EventBusOption,
    pub(super) pending_view_events: Vec<ViewEvent>,
    pub(super) pending_model_events: Vec<ModelEvent>,

    // Double buffering state
    pub(super) current_screen_buffer: ScreenBuffer,
    pub(super) previous_screen_buffer: ScreenBuffer,
}

impl ViewModel {
    /// Create a new ViewModel with default state
    pub fn new() -> Self {
        let response = ResponseModel::new();

        // Default terminal size
        let terminal_dimensions = (80, 24);

        // Build initial display caches
        let content_width = (terminal_dimensions.0 as usize).saturating_sub(4); // Account for line numbers

        // Calculate pane heights
        let request_pane_height = (terminal_dimensions.1 / 2) as usize;
        let response_pane_height = (terminal_dimensions.1 as usize)
            .saturating_sub((terminal_dimensions.1 / 2) as usize)
            .saturating_sub(2) // -2 for separator and status
            .max(1); // Ensure minimum height of 1

        // Initialize pane array with proper display caches and dimensions
        let request_pane = PaneState::new(Pane::Request, content_width, request_pane_height, true);
        let response_pane =
            PaneState::new(Pane::Response, content_width, response_pane_height, true);

        Self {
            editor_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            response,
            panes: [request_pane, response_pane],
            wrap_enabled: true,
            terminal_dimensions,
            request_pane_height: terminal_dimensions.1 / 2,
            status_line: StatusLine::new(),
            http_client: None,
            http_session_headers: HashMap::new(),
            http_verbose: false,
            event_bus: None,
            pending_view_events: Vec::new(),
            pending_model_events: Vec::new(),
            current_screen_buffer: ScreenBuffer::new(
                terminal_dimensions.0 as usize,
                terminal_dimensions.1 as usize,
            ),
            previous_screen_buffer: ScreenBuffer::new(
                terminal_dimensions.0 as usize,
                terminal_dimensions.1 as usize,
            ),
        }
    }

    /// Set the event bus for this ViewModel
    pub fn set_event_bus(&mut self, event_bus: Box<dyn EventBus>) {
        self.event_bus = Some(event_bus);
        tracing::debug!("Event bus set for ViewModel");
    }

    /// Update terminal size and resize screen buffers
    pub fn update_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_dimensions = (width, height);

        // Calculate request pane height (split screen when response exists)
        self.request_pane_height = if self.response.status_code().is_some() {
            height / 2
        } else {
            height - 1 // Reserve space for status bar
        };

        // Recalculate pane dimensions
        let content_width = (width as usize).saturating_sub(4); // Account for line numbers
        let request_pane_height = self.request_pane_height as usize;
        let response_pane_height = (height as usize)
            .saturating_sub(self.request_pane_height as usize)
            .saturating_sub(2) // -2 for separator and status
            .max(1); // Ensure minimum height of 1

        // Update pane dimensions
        self.panes[Pane::Request].update_dimensions(content_width, request_pane_height);
        self.panes[Pane::Response].update_dimensions(content_width, response_pane_height);

        // Resize screen buffers
        self.current_screen_buffer
            .resize(width as usize, height as usize);
        self.previous_screen_buffer
            .resize(width as usize, height as usize);

        // Invalidate display caches for both panes
        self.panes[Pane::Request].display_cache.invalidate();
        self.panes[Pane::Response].display_cache.invalidate();

        tracing::debug!(
            "Terminal size updated: {}x{}, pane dimensions: Request={}x{}, Response={}x{}",
            width,
            height,
            content_width,
            request_pane_height,
            content_width,
            response_pane_height
        );
    }

    /// Get current screen buffer dimensions
    pub fn screen_buffer_dimensions(&self) -> (usize, usize) {
        self.current_screen_buffer.dimensions()
    }

    /// Swap screen buffers (for double buffering)
    pub fn swap_screen_buffers(&mut self) {
        std::mem::swap(
            &mut self.current_screen_buffer,
            &mut self.previous_screen_buffer,
        );
        self.current_screen_buffer.clear();
    }

    /// Get changed rows between current and previous screen buffers
    pub fn get_screen_buffer_diff(&self) -> Vec<usize> {
        self.current_screen_buffer
            .diff(&self.previous_screen_buffer)
    }

    /// Get reference to current screen buffer (for rendering)
    pub fn current_screen_buffer(&self) -> &ScreenBuffer {
        &self.current_screen_buffer
    }

    /// Get mutable reference to current screen buffer (for building)
    pub fn current_screen_buffer_mut(&mut self) -> &mut ScreenBuffer {
        &mut self.current_screen_buffer
    }

    /// Get terminal size
    pub fn terminal_size(&self) -> (u16, u16) {
        self.terminal_dimensions
    }

    /// Set the profile information for display
    pub fn set_profile_info(&mut self, profile_name: String, profile_path: String) {
        self.status_line.set_profile(profile_name, profile_path);
    }

    /// Get the current profile name
    pub fn get_profile_name(&self) -> &str {
        self.status_line.profile_name()
    }

    /// Get the current profile path
    pub fn get_profile_path(&self) -> &str {
        self.status_line.profile_path()
    }

    // === Pane Access Methods ===

    /// Get the current active pane
    pub fn current_pane(&self) -> Pane {
        self.current_pane
    }

    /// Switch to the other pane
    pub fn toggle_pane(&mut self) {
        self.current_pane = match self.current_pane {
            Pane::Request => Pane::Response,
            Pane::Response => Pane::Request,
        };
    }

    /// Get immutable reference to request pane
    pub fn request_pane(&self) -> &PaneState {
        &self.panes[Pane::Request]
    }

    /// Get mutable reference to request pane
    pub fn request_pane_mut(&mut self) -> &mut PaneState {
        &mut self.panes[Pane::Request]
    }

    /// Get immutable reference to response pane
    pub fn response_pane(&self) -> &PaneState {
        &self.panes[Pane::Response]
    }

    /// Get mutable reference to response pane
    pub fn response_pane_mut(&mut self) -> &mut PaneState {
        &mut self.panes[Pane::Response]
    }

    /// Get immutable reference to current active pane
    pub fn current_pane_state(&self) -> &PaneState {
        &self.panes[self.current_pane]
    }

    /// Get mutable reference to current active pane
    pub fn current_pane_state_mut(&mut self) -> &mut PaneState {
        &mut self.panes[self.current_pane]
    }

    /// Set a temporary status message for display
    pub fn set_status_message<S: Into<String>>(&mut self, message: S) {
        self.status_line.set_status_message(message);
    }

    /// Clear the status message
    pub fn clear_status_message(&mut self) {
        self.status_line.clear_status_message();
    }

    /// Get the current status message
    pub fn get_status_message(&self) -> Option<&str> {
        self.status_line.status_message()
    }

    /// Check if display cursor position is visible in status bar
    pub fn is_display_cursor_visible(&self) -> bool {
        self.status_line.is_display_cursor_visible()
    }

    // === Editor State Management ===

    /// Get current editor mode
    pub fn mode(&self) -> EditorMode {
        self.editor_mode
    }

    /// Set editor mode, returning event if changed
    pub fn set_mode(&mut self, new_mode: EditorMode) -> Option<ModelEvent> {
        if self.editor_mode != new_mode {
            let old_mode = self.editor_mode;
            self.editor_mode = new_mode;
            Some(ModelEvent::ModeChanged { old_mode, new_mode })
        } else {
            None
        }
    }

    /// Set current pane, returning event if changed
    pub fn set_current_pane(&mut self, new_pane: Pane) -> Option<ModelEvent> {
        if self.current_pane != new_pane {
            let old_pane = self.current_pane;
            self.current_pane = new_pane;
            Some(ModelEvent::PaneSwitched { old_pane, new_pane })
        } else {
            None
        }
    }
}

impl Default for ViewModel {
    fn default() -> Self {
        Self::new()
    }
}
