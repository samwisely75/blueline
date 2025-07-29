//! # Core ViewModel Structure
//!
//! Contains the main ViewModel struct and basic initialization logic.
//! This is the central coordinator that delegates to specialized managers.

use crate::repl::events::{EventBus, Pane, ViewEvent};
use crate::repl::models::{BufferModel, DisplayCache, EditorModel, ResponseModel};
use crate::repl::view_models::screen_buffer::ScreenBuffer;
// use anyhow::Result; // Currently unused
use bluenote::HttpClient;
use std::collections::HashMap;

/// Type alias for event bus option to reduce complexity
type EventBusOption = Option<Box<dyn EventBus>>;

/// Type alias for display line rendering data: (content, line_number, is_continuation)
pub type DisplayLineData = (String, Option<usize>, bool);

/// The central ViewModel that coordinates all business logic
pub struct ViewModel {
    // Core models
    pub(super) editor: EditorModel,
    pub(super) request_buffer: BufferModel,
    pub(super) response_buffer: BufferModel,
    pub(super) response: ResponseModel,

    // Display coordination (word wrap support)
    pub(super) request_display_cache: DisplayCache,
    pub(super) response_display_cache: DisplayCache,
    pub(super) wrap_enabled: bool,
    pub(super) request_display_cursor: (usize, usize), // (display_line, display_column)
    pub(super) response_display_cursor: (usize, usize),
    pub(super) request_scroll_offset: (usize, usize), // (vertical, horizontal)
    pub(super) response_scroll_offset: (usize, usize),

    // Display state
    pub(super) terminal_width: u16,
    pub(super) terminal_height: u16,
    pub(super) request_pane_height: u16,

    // Ex command mode state (for :q, :w, etc.)
    pub(super) ex_command_buffer: String,

    // Request execution state
    pub(super) is_executing_request: bool,

    // HTTP client and configuration
    pub(super) http_client: Option<HttpClient>,
    pub(super) session_headers: HashMap<String, String>,
    pub(super) verbose: bool,

    // Profile information
    pub(super) profile_name: String,
    pub(super) profile_path: String,

    // Status message for temporary display
    pub(super) status_message: Option<String>,

    // Event management
    pub(super) event_bus: EventBusOption,
    pub(super) pending_view_events: Vec<ViewEvent>,

    // Double buffering state
    pub(super) current_screen_buffer: ScreenBuffer,
    pub(super) previous_screen_buffer: ScreenBuffer,
}

impl ViewModel {
    /// Create a new ViewModel with default state
    pub fn new() -> Self {
        let editor = EditorModel::new();
        let request_buffer = BufferModel::new(Pane::Request);
        let response_buffer = BufferModel::new(Pane::Response);
        let response = ResponseModel::new();

        // Default terminal size
        let terminal_width = 80;
        let terminal_height = 24;

        // Build initial display caches
        let content_width = (terminal_width as usize).saturating_sub(4); // Account for line numbers
        let request_lines = request_buffer.content().lines().to_vec();
        let response_lines = response_buffer.content().lines().to_vec();

        let request_display_cache =
            crate::repl::models::build_display_cache(&request_lines, content_width, true)
                .unwrap_or_else(|_| DisplayCache::new());
        let response_display_cache =
            crate::repl::models::build_display_cache(&response_lines, content_width, true)
                .unwrap_or_else(|_| DisplayCache::new());

        Self {
            editor,
            request_buffer,
            response_buffer,
            response,
            request_display_cache,
            response_display_cache,
            wrap_enabled: true,
            request_display_cursor: (0, 0),
            response_display_cursor: (0, 0),
            request_scroll_offset: (0, 0),
            response_scroll_offset: (0, 0),
            terminal_width,
            terminal_height,
            request_pane_height: terminal_height / 2,
            ex_command_buffer: String::new(),
            is_executing_request: false,
            http_client: None,
            session_headers: HashMap::new(),
            verbose: false,
            profile_name: "default".to_string(),
            profile_path: "~/.blueline/profile".to_string(),
            status_message: None,
            event_bus: None,
            pending_view_events: Vec::new(),
            current_screen_buffer: ScreenBuffer::new(
                terminal_width as usize,
                terminal_height as usize,
            ),
            previous_screen_buffer: ScreenBuffer::new(
                terminal_width as usize,
                terminal_height as usize,
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
        self.terminal_width = width;
        self.terminal_height = height;

        // Calculate request pane height (split screen when response exists)
        self.request_pane_height = if self.response.status_code().is_some() {
            height / 2
        } else {
            height - 1 // Reserve space for status bar
        };

        // Resize screen buffers
        self.current_screen_buffer
            .resize(width as usize, height as usize);
        self.previous_screen_buffer
            .resize(width as usize, height as usize);

        // Invalidate display caches
        self.request_display_cache.invalidate();
        self.response_display_cache.invalidate();

        tracing::debug!("Terminal size updated: {}x{}", width, height);
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
        (self.terminal_width, self.terminal_height)
    }

    /// Set the profile information for display
    pub fn set_profile_info(&mut self, profile_name: String, profile_path: String) {
        self.profile_name = profile_name;
        self.profile_path = profile_path;
    }

    /// Get the current profile name
    pub fn get_profile_name(&self) -> &str {
        &self.profile_name
    }

    /// Get the current profile path
    pub fn get_profile_path(&self) -> &str {
        &self.profile_path
    }

    /// Set a temporary status message for display
    pub fn set_status_message<S: Into<String>>(&mut self, message: S) {
        self.status_message = Some(message.into());
    }

    /// Clear the status message
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    /// Get the current status message
    pub fn get_status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }
}

impl Default for ViewModel {
    fn default() -> Self {
        Self::new()
    }
}
