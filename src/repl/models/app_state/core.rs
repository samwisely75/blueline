//! # Application State
//!
//! Core application state that represents the entire application's data model.
//! This merges the functionality of ViewModel and PaneManager into a single cohesive state.
//!
//! This follows the MVVM pattern where:
//! - AppState contains all application state and business logic
//! - Views depend only on AppState for rendering

use super::PaneManager;
use crate::repl::models::pane_state::{EditorMode, Pane, PaneState};
use crate::repl::models::{
    ClipboardYankBuffer, LogicalPosition, MemoryYankBuffer, ResponseModel, StatusLine, YankBuffer,
};
use std::collections::HashMap;

/// Type alias for display line rendering data: (content, line_number, is_continuation, logical_start_col, logical_line)
pub type DisplayLineData = (String, Option<usize>, bool, usize, usize);

/// Core application state containing all data models and business logic
///
/// This struct merges the functionality that was previously split between
/// ViewModel and PaneManager, providing a unified state management interface.
pub struct AppState {
    // Core state
    pub(crate) response: ResponseModel,

    // Pane management - encapsulates all pane-related state and operations
    pub(crate) pane_manager: PaneManager,

    // Status line model - encapsulates all status bar state
    pub(crate) status_line: StatusLine,

    // HTTP session configuration
    pub(crate) http_session_headers: HashMap<String, String>,

    // Yank buffer for copy/paste operations
    pub(crate) yank_buffer: Box<dyn YankBuffer>,

    // Whether clipboard integration is enabled
    pub(crate) clipboard_enabled: bool,

    // Whether d/dd/D commands should cut (yank) instead of just delete
    pub(crate) dcut_enabled: bool,
}

impl AppState {
    /// Create a new AppState with default state
    ///
    /// Sets up all specialized managers with sensible defaults:
    /// - 80x24 terminal dimensions for initial layout calculations
    /// - Request pane as default active pane
    /// - Normal editor mode as starting state
    pub fn new() -> Self {
        let response = ResponseModel::new();

        // Default terminal size
        let terminal_dimensions = (80, 24);

        Self {
            response,
            pane_manager: PaneManager::new(terminal_dimensions),
            status_line: StatusLine::new(),
            http_session_headers: HashMap::new(),
            yank_buffer: Box::new(MemoryYankBuffer::new()),
            clipboard_enabled: false,
            dcut_enabled: true, // Default to true for cut behavior
        }
    }

    // Direct access to current PaneState for reduced delegation

    /// Get current active pane state for direct access to editing operations
    pub fn current_pane(&self) -> &PaneState {
        &self.pane_manager.panes[self.pane_manager.current_pane]
    }

    /// Get mutable access to current active pane state
    pub fn current_pane_mut(&mut self) -> &mut PaneState {
        &mut self.pane_manager.panes[self.pane_manager.current_pane]
    }

    /// Get specific pane state by pane type
    pub fn pane(&self, pane_type: Pane) -> &PaneState {
        &self.pane_manager.panes[pane_type]
    }

    /// Get mutable access to specific pane state by pane type
    pub fn pane_mut(&mut self, pane_type: Pane) -> &mut PaneState {
        &mut self.pane_manager.panes[pane_type]
    }

    /// Set Visual Block Insert cursor positions for multi-cursor editing
    pub fn set_visual_block_insert_cursors(&mut self, positions: Vec<LogicalPosition>) {
        self.current_pane_mut()
            .set_visual_block_insert_cursors(positions);
    }

    /// Update only the cursor positions without changing boundaries
    pub fn update_visual_block_insert_cursors(&mut self, positions: Vec<LogicalPosition>) {
        self.current_pane_mut()
            .update_visual_block_insert_cursors(positions);
    }

    /// Get Visual Block Insert cursor positions  
    pub fn get_visual_block_insert_cursors(&self) -> Vec<LogicalPosition> {
        self.current_pane()
            .get_visual_block_insert_cursors()
            .to_vec()
    }

    /// Get Visual Block Insert start column boundaries
    pub fn get_visual_block_insert_start_columns(&self) -> Vec<usize> {
        self.current_pane()
            .get_visual_block_insert_start_columns()
            .to_vec()
    }

    /// Clear Visual Block Insert cursor positions
    pub fn clear_visual_block_insert_cursors(&mut self) {
        self.current_pane_mut().clear_visual_block_insert_cursors();
    }

    /// Check if we're in multi-cursor Visual Block Insert mode
    pub fn is_in_visual_block_insert_mode(&self) -> bool {
        self.current_pane().is_in_visual_block_insert_mode()
    }

    /// Enable or disable system clipboard integration
    pub fn set_clipboard_enabled(&mut self, enabled: bool) -> anyhow::Result<()> {
        if enabled == self.clipboard_enabled {
            // No change needed
            return Ok(());
        }

        // Save any existing content before switching
        let existing_content = self.yank_buffer.paste().map(|s| s.to_string());

        // Switch yank buffer implementation
        if enabled {
            // Try to create clipboard buffer
            match ClipboardYankBuffer::new() {
                Ok(clipboard_buffer) => {
                    self.yank_buffer = Box::new(clipboard_buffer);
                    self.clipboard_enabled = true;
                    tracing::info!("Switched to system clipboard yank buffer");
                }
                Err(e) => {
                    tracing::error!("Failed to enable clipboard: {}", e);
                    return Err(anyhow::anyhow!("Failed to access system clipboard: {}", e));
                }
            }
        } else {
            // Switch back to memory buffer
            self.yank_buffer = Box::new(MemoryYankBuffer::new());
            self.clipboard_enabled = false;
            tracing::info!("Switched to memory yank buffer");
        }

        // Restore existing content if any
        if let Some(content) = existing_content {
            let _ = self.yank_buffer.yank(content);
        }

        Ok(())
    }

    /// Enable or disable cut behavior for d/dd/D commands
    pub fn set_dcut_enabled(&mut self, enabled: bool) {
        self.dcut_enabled = enabled;
        tracing::info!("DCut mode set to: {}", if enabled { "on" } else { "off" });
    }

    /// Get whether cut behavior is enabled for d/dd/D commands
    pub fn is_dcut_enabled(&self) -> bool {
        self.dcut_enabled
    }

    /// Update terminal size and resize screen buffers
    ///
    /// Ensures all rendering components stay synchronized with terminal dimensions:
    /// 1. Updates PaneManager for layout calculations
    /// 2. Considers response status for pane height calculations
    pub fn update_terminal_size(&mut self, width: u16, height: u16) {
        // Update PaneManager's terminal size and pane dimensions
        self.pane_manager.update_terminal_size(
            width,
            height,
            self.response.status_code().is_some(),
        );
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
        self.pane_manager.switch_to_other_area();
        // Update status line pane
        self.status_line
            .set_current_pane(self.pane_manager.current_pane_type());
    }

    /// Switch to Request pane
    pub fn switch_to_request_pane(&mut self) {
        self.pane_manager.switch_to_request_pane();
        self.status_line.set_current_pane(Pane::Request);
    }

    /// Switch to Response pane
    pub fn switch_to_response_pane(&mut self) {
        self.pane_manager.switch_to_response_pane();
        self.status_line.set_current_pane(Pane::Response);
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
        self.current_pane().get_mode()
    }

    /// Set editor mode for the active pane
    pub fn set_mode(&mut self, new_mode: EditorMode) -> bool {
        let old_mode = self.current_pane().get_mode();
        if old_mode != new_mode {
            self.current_pane_mut().set_mode(new_mode);
            true
        } else {
            false
        }
    }

    /// Get content width (terminal width minus line numbers and padding)
    pub fn get_content_width(&self) -> usize {
        // Use semantic width calculation based on current area
        let line_num_width = self.pane_manager.get_current_line_number_width();
        (self.pane_manager.terminal_dimensions.0 as usize).saturating_sub(line_num_width + 1)
    }

    /// Get reference to PaneManager for pane-specific operations
    pub fn pane_manager(&self) -> &PaneManager {
        &self.pane_manager
    }

    /// Restore the last visual selection (for 'gv' command)
    /// Returns the mode to enter if restoration successful
    pub fn restore_last_visual_selection(&mut self) -> anyhow::Result<Option<EditorMode>> {
        // Delegate to the pane manager to restore selection in current pane
        match self.pane_manager.restore_last_visual_selection() {
            Some(mode) => Ok(Some(mode)),
            None => Ok(None),
        }
    }

    // Event management methods are in rendering_coordinator.rs
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
