//! # Core ViewModel Structure
//!
//! Contains the main ViewModel struct and basic initialization logic.
//! This is the central coordinator that delegates to specialized managers.

use crate::repl::events::{EventBus, LogicalPosition, Pane, ViewEvent};
use crate::repl::models::{BufferModel, DisplayCache, EditorModel, ResponseModel};
use crate::repl::view_models::screen_buffer::ScreenBuffer;
// use anyhow::Result; // Currently unused
use bluenote::HttpClient;
use std::collections::HashMap;
use std::ops::{Index, IndexMut, Not};

/// Type alias for event bus option to reduce complexity
type EventBusOption = Option<Box<dyn EventBus>>;

/// Type alias for display line rendering data: (content, line_number, is_continuation)
pub type DisplayLineData = (String, Option<usize>, bool);

/// State container for a single pane (Request or Response)
#[derive(Debug, Clone)]
pub struct PaneState {
    pub buffer: BufferModel,
    pub display_cache: DisplayCache,
    pub display_cursor: (usize, usize), // (display_line, display_column)
    pub scroll_offset: (usize, usize),  // (vertical, horizontal)
}

impl PaneState {
    fn new(pane: Pane) -> Self {
        Self {
            buffer: BufferModel::new(pane),
            display_cache: DisplayCache::new(),
            display_cursor: (0, 0),
            scroll_offset: (0, 0),
        }
    }
}

/// Array indexing for panes to enable clean access patterns
impl Index<Pane> for [PaneState; 2] {
    type Output = PaneState;
    fn index(&self, pane: Pane) -> &Self::Output {
        match pane {
            Pane::Request => &self[0],
            Pane::Response => &self[1],
        }
    }
}

impl IndexMut<Pane> for [PaneState; 2] {
    fn index_mut(&mut self, pane: Pane) -> &mut Self::Output {
        match pane {
            Pane::Request => &mut self[0],
            Pane::Response => &mut self[1],
        }
    }
}

/// Enable pane switching with !current_pane
impl Not for Pane {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Pane::Request => Pane::Response,
            Pane::Response => Pane::Request,
        }
    }
}

/// The central ViewModel that coordinates all business logic
pub struct ViewModel {
    // Core models
    pub(super) editor: EditorModel,
    pub(super) response: ResponseModel,

    // Pane management - replaces 8 duplicate fields with unified structure
    pub(super) panes: [PaneState; 2],
    pub(super) current_pane: Pane,

    // Display coordination (word wrap support)
    pub(super) wrap_enabled: bool,

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

    // Visual mode selection state
    pub(super) visual_selection_start: Option<LogicalPosition>,
    pub(super) visual_selection_end: Option<LogicalPosition>,
    pub(super) visual_selection_pane: Option<Pane>,
}

impl ViewModel {
    /// Create a new ViewModel with default state
    pub fn new() -> Self {
        let editor = EditorModel::new();
        let response = ResponseModel::new();

        // Default terminal size
        let terminal_width = 80;
        let terminal_height = 24;

        // Build initial display caches
        let content_width = (terminal_width as usize).saturating_sub(4); // Account for line numbers

        // Initialize pane array with proper display caches
        let mut request_pane = PaneState::new(Pane::Request);
        let mut response_pane = PaneState::new(Pane::Response);

        let request_lines = request_pane.buffer.content().lines().to_vec();
        let response_lines = response_pane.buffer.content().lines().to_vec();

        request_pane.display_cache =
            crate::repl::models::build_display_cache(&request_lines, content_width, true)
                .unwrap_or_else(|_| DisplayCache::new());
        response_pane.display_cache =
            crate::repl::models::build_display_cache(&response_lines, content_width, true)
                .unwrap_or_else(|_| DisplayCache::new());

        Self {
            editor,
            response,
            panes: [request_pane, response_pane],
            current_pane: Pane::Request,
            wrap_enabled: true,
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
            visual_selection_start: None,
            visual_selection_end: None,
            visual_selection_pane: None,
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

        // Invalidate display caches for both panes
        self.panes[Pane::Request].display_cache.invalidate();
        self.panes[Pane::Response].display_cache.invalidate();

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

    // === Pane Access Methods ===

    /// Get the current active pane
    pub fn current_pane(&self) -> Pane {
        self.current_pane
    }

    /// Switch to the other pane
    pub fn toggle_pane(&mut self) {
        self.current_pane = !self.current_pane;
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
