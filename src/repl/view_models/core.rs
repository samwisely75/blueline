//! # Core ViewModel Structure
//!
//! Contains the main ViewModel struct and basic initialization logic.
//! This is the central coordinator that delegates to specialized managers.

use crate::repl::events::{EditorMode, EventBus, ModelEvent, Pane, ViewEvent};
use crate::repl::models::{ResponseModel, StatusLine};
use crate::repl::view_models::pane_manager::PaneManager;
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
    pub(super) response: ResponseModel,

    // Pane management - encapsulates all pane-related state and operations
    pub(super) pane_manager: PaneManager,

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

        Self {
            response,
            pane_manager: PaneManager::new(terminal_dimensions),
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
        // Update PaneManager's terminal size and pane dimensions
        self.pane_manager.update_terminal_size(
            width,
            height,
            self.response.status_code().is_some(),
        );

        // Resize screen buffers
        self.current_screen_buffer
            .resize(width as usize, height as usize);
        self.previous_screen_buffer
            .resize(width as usize, height as usize);
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
        self.pane_manager.terminal_dimensions
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

    // === Pane Methods (Semantic Operations) ===

    /// Get current active pane (for backward compatibility - prefer semantic operations)
    pub fn get_current_pane(&self) -> Pane {
        self.pane_manager.current_pane_type()
    }

    /// Check if currently in Request pane
    pub fn is_in_request_pane(&self) -> bool {
        self.pane_manager.is_in_request_pane()
    }

    /// Check if currently in Response pane  
    pub fn is_in_response_pane(&self) -> bool {
        self.pane_manager.is_in_response_pane()
    }

    /// Switch to the other pane
    pub fn switch_to_other_pane(&mut self) {
        let events = self.pane_manager.switch_to_other_area();
        if !events.is_empty() {
            // Update status line pane
            self.status_line
                .set_current_pane(self.pane_manager.current_pane_type());
            let _ = self.emit_view_event(events);
        }
    }

    /// Switch to Request pane
    pub fn switch_to_request_pane(&mut self) {
        let events = self.pane_manager.switch_to_request_pane();
        if !events.is_empty() {
            self.status_line.set_current_pane(Pane::Request);
            let _ = self.emit_view_event(events);
        }
    }

    /// Switch to Response pane
    pub fn switch_to_response_pane(&mut self) {
        let events = self.pane_manager.switch_to_response_pane();
        if !events.is_empty() {
            self.status_line.set_current_pane(Pane::Response);
            let _ = self.emit_view_event(events);
        }
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

    /// Get current editor mode from the active pane
    pub fn mode(&self) -> EditorMode {
        self.pane_manager.get_current_pane_mode()
    }

    /// Set editor mode for the active pane, returning event if changed
    pub fn set_mode(&mut self, new_mode: EditorMode) -> Option<ModelEvent> {
        let old_mode = self.pane_manager.get_current_pane_mode();
        if old_mode != new_mode {
            self.pane_manager.set_current_pane_mode(new_mode);
            Some(ModelEvent::ModeChanged { old_mode, new_mode })
        } else {
            None
        }
    }

    /// Get content width (terminal width minus line numbers and padding)
    pub fn get_content_width(&self) -> usize {
        // Use semantic width calculation based on current area
        let current_pane = self.pane_manager.current_pane_type();
        let line_num_width = self.get_line_number_width(current_pane);
        (self.pane_manager.terminal_dimensions.0 as usize).saturating_sub(line_num_width + 1)
    }

    /// Get reference to PaneManager for pane-specific operations
    pub fn pane_manager(&self) -> &PaneManager {
        &self.pane_manager
    }
}

impl Default for ViewModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::events::EditorMode;

    /// Test for Issue #84: Wrap mode cursor positioning bug
    ///
    /// When wrap mode is enabled and exactly enough characters are typed to fill
    /// a display line (e.g., 106 chars in 106-width display), the cursor should
    /// wrap to the beginning of the next display line, not stay at the end of
    /// the current line.
    ///
    /// This test verifies:
    /// 1. No horizontal scrolling occurs in wrap mode
    /// 2. Display cursor properly wraps to next line at exact boundary
    /// 3. Logical cursor position remains correct
    #[test]
    fn test_issue_84_wrap_cursor_positioning() {
        let mut vm = ViewModel::new();

        // Set terminal size to get exactly 106 content width (as per issue #84)
        vm.update_terminal_size(110, 24);

        // Enable wrap mode
        vm.pane_manager.set_wrap_enabled(true);

        // Switch to insert mode
        vm.change_mode(EditorMode::Insert).unwrap();

        let content_width = vm.pane_manager.get_content_width();
        assert_eq!(
            content_width, 106,
            "Test setup should give 106 content width"
        );

        // Type exactly 106 characters (should fill first display line completely)
        let test_line: String = "a".repeat(106);
        for ch in test_line.chars() {
            vm.insert_char(ch).unwrap();
        }

        // Check state after 106 characters
        let logical_cursor = vm.pane_manager.get_current_cursor_position();
        let display_cursor = vm.pane_manager.get_current_display_cursor();
        let scroll_offset = vm.pane_manager.get_current_scroll_offset();

        // Issue #84: After 106 characters, cursor should be at beginning of second display line

        assert_eq!(
            scroll_offset.col, 0,
            "No horizontal scrolling should occur in wrap mode"
        );
        assert_eq!(logical_cursor.line, 0, "Logical line should remain 0");
        assert_eq!(logical_cursor.column, 106, "Logical column should be 106");

        // The fix: display cursor should be at beginning of next display line when content exactly fills line

        // This assertion should pass when the bug is fixed
        assert_eq!(
            display_cursor.row, 1,
            "Display cursor should be on second display line after 106 chars"
        );
        assert_eq!(
            display_cursor.col, 0,
            "Display cursor should be at beginning of second display line"
        );
    }
}
