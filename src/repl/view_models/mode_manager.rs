//! # Mode Management
//!
//! Handles editor mode transitions, visual mode selection state, and mode-related operations.

use crate::repl::events::{EditorMode, LogicalPosition, Pane, ViewEvent};
use crate::repl::view_models::core::ViewModel;
use anyhow::Result;

/// Type alias for visual selection state to reduce complexity
type VisualSelectionState = (
    Option<LogicalPosition>,
    Option<LogicalPosition>,
    Option<Pane>,
);

impl ViewModel {
    /// Get current editor mode
    pub fn get_mode(&self) -> EditorMode {
        self.mode()
    }

    /// Get previous editor mode
    pub fn get_previous_mode(&self) -> EditorMode {
        self.status_line.previous_mode()
    }

    /// Change editor mode for the current active pane
    pub fn change_mode(&mut self, mode: EditorMode) -> Result<()> {
        // Get current mode from the active pane
        let old_mode = self.pane_manager.get_current_pane_mode();
        tracing::debug!(
            "Changing mode from {:?} to {:?} for current pane",
            old_mode,
            mode
        );

        // Set mode for the current pane
        self.pane_manager.set_current_pane_mode(mode);

        // CURSOR & SCROLL PULLBACK: When switching from Insert to Normal/Visual mode,
        // pull cursor back if it's at the "new character position" (past last character)
        // and also pull back horizontal scrolling if needed
        let mut mode_change_events = Vec::new();
        if old_mode == EditorMode::Insert
            && matches!(
                mode,
                EditorMode::Normal
                    | EditorMode::Visual
                    | EditorMode::VisualLine
                    | EditorMode::VisualBlock
            )
        {
            // Check if cursor needs to be pulled back
            let current_cursor_pos = self.pane_manager.get_current_display_cursor();

            // Get current line to check if cursor is past last character
            if let Some(current_pane_state) = self.pane_manager.get_current_pane_state() {
                if let Some(current_line) = current_pane_state
                    .display_cache
                    .get_display_line(current_cursor_pos.row)
                {
                    let line_display_width = current_line.display_width();

                    // If cursor is at the "new character position" (past last character)
                    if current_cursor_pos.col >= line_display_width && line_display_width > 0 {
                        tracing::debug!(
                            "Insert→Normal: Cursor at new character position (col={}), pulling back cursor (line_width={})",
                            current_cursor_pos.col, line_display_width
                        );

                        // Pull cursor back by moving left
                        let pullback_events = self.pane_manager.move_cursor_left();
                        mode_change_events.extend(pullback_events);

                        // After cursor pullback, check if we need to pull back horizontal scrolling too
                        let new_cursor_pos = self.pane_manager.get_current_display_cursor();
                        let scroll_offset = self.pane_manager.get_current_scroll_offset();
                        let content_width = self.pane_manager.get_content_width();

                        // HORIZONTAL SCROLL PULLBACK: If the cursor is now within the content area
                        // without needing horizontal scroll, pull it back
                        if scroll_offset.col > 0 && new_cursor_pos.col < content_width {
                            tracing::debug!(
                                "Insert→Normal: Cursor at pos {} fits in content_width {}, pulling back horizontal scroll from {}",
                                new_cursor_pos.col, content_width, scroll_offset.col
                            );

                            // Trigger horizontal scroll adjustment by calling ensure_cursor_visible
                            // This will automatically pull back the scroll since we're now in Normal mode
                            let scroll_events = self
                                .pane_manager
                                .ensure_current_cursor_visible(content_width);
                            mode_change_events.extend(scroll_events);
                        }
                    }
                }
            }
        }

        // Update status line mode
        self.status_line.set_editor_mode(mode);

        // Clear command buffer when exiting Command mode (e.g., when pressing Escape)
        if old_mode == EditorMode::Command && mode != EditorMode::Command {
            self.status_line.clear_command_buffer();
            tracing::debug!("Cleared command buffer when exiting Command mode");
        }

        // Handle visual mode selection state using PaneManager
        let mut events = mode_change_events; // Start with any cursor pullback events
        let entering_visual_mode = matches!(
            mode,
            EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock
        );
        let exiting_visual_mode = matches!(
            old_mode,
            EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock
        );

        if entering_visual_mode && !exiting_visual_mode {
            // Entering any visual mode from non-visual mode
            events.extend(self.pane_manager.start_visual_selection());
        } else if exiting_visual_mode && !entering_visual_mode {
            // Exiting visual mode to non-visual mode
            events.extend(self.pane_manager.end_visual_selection());
        }
        // Note: switching between visual modes (v ↔ V ↔ Ctrl+V) maintains selection

        // Add standard mode change events
        events.extend([
            ViewEvent::StatusBarUpdateRequired,
            ViewEvent::ActiveCursorUpdateRequired,
        ]);

        let _ = self.emit_view_event(events);

        tracing::info!(
            "Successfully changed mode from {:?} to {:?} for current pane",
            old_mode,
            mode
        );
        Ok(())
    }

    /// Get visual selection state
    pub fn get_visual_selection(&self) -> VisualSelectionState {
        self.pane_manager.get_visual_selection()
    }

    /// Check if a position is within visual selection
    pub fn is_position_selected(&self, position: LogicalPosition, pane: Pane) -> bool {
        self.pane_manager.is_position_selected(position, pane)
    }
}
