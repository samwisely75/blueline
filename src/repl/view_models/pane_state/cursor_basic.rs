//! Basic cursor movement operations for PaneState
//!
//! This module contains methods for:
//! - Basic directional cursor movement (left, right, up, down)
//! - Capability checking for navigation
//! - Visual selection handling during movement
//! - Virtual column management for Vim-style navigation

use crate::repl::events::{EditorMode, LogicalPosition, PaneCapabilities, ViewEvent};
use crate::repl::geometry::Position;

use super::PaneState;

impl PaneState {
    /// Move cursor left with capability checking and visual selection support
    pub fn move_cursor_left(&mut self, content_width: usize) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        let current_display_pos = self.display_cursor;
        let mut moved = false;

        // Check if we can move left within current display line
        if current_display_pos.col > 0 {
            // Use character-aware left movement
            if let Some(current_line) = self.display_cache.get_display_line(current_display_pos.row)
            {
                let new_col = current_line.move_left_by_character(current_display_pos.col);
                let new_display_pos = Position::new(current_display_pos.row, new_col);
                self.display_cursor = new_display_pos;
                // Update virtual column for horizontal movement
                self.update_virtual_column();
                moved = true;
            }
        } else if current_display_pos.row > 0 {
            // VISUAL BLOCK FIX: In Visual Block mode, prevent moving to previous line
            if self.editor_mode != EditorMode::VisualBlock {
                // Move to end of previous display line
                let prev_display_line = current_display_pos.row - 1;
                if let Some(prev_line) = self.display_cache.get_display_line(prev_display_line) {
                    // Use display width instead of character count for proper multibyte character support
                    let new_col = prev_line.display_width().saturating_sub(1);
                    let new_display_pos = Position::new(prev_display_line, new_col);
                    self.display_cursor = new_display_pos;
                    // Update virtual column for horizontal movement
                    self.update_virtual_column();
                    moved = true;
                }
            }
        }

        if moved {
            // Sync logical cursor with new display position
            let new_display_pos = self.display_cursor;
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.buffer.set_cursor(new_logical_pos);

                // Update visual selection if active
                self.update_visual_selection_on_cursor_move(new_logical_pos);
            }

            let mut events = vec![
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
                ViewEvent::CurrentAreaRedrawRequired,
            ];

            // Ensure cursor is visible and add visibility events
            let visibility_events = self.ensure_cursor_visible_with_events(content_width);
            events.extend(visibility_events);

            events
        } else {
            vec![]
        }
    }

    /// Move cursor right with capability checking and visual selection support
    pub fn move_cursor_right(&mut self, content_width: usize) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        let current_display_pos = self.display_cursor;
        let mut moved = false;

        // Check if cursor can move right within current line
        let can_move_right_in_line = if let Some(current_line) =
            self.display_cache.get_display_line(current_display_pos.row)
        {
            let line_display_width = current_line.display_width();

            match self.editor_mode {
                EditorMode::Insert => {
                    // Insert mode: Allow cursor to go one position past end of line
                    current_display_pos.col < line_display_width
                }
                EditorMode::VisualBlock => {
                    // Visual Block mode: Allow cursor to move beyond line content
                    true // Always allow right movement in Visual Block mode
                }
                _ => {
                    // Normal/Visual mode: Stop at last character position
                    if line_display_width == 0 {
                        false // Empty line - no movement allowed
                    } else {
                        let next_pos =
                            current_line.move_right_by_character(current_display_pos.col);
                        next_pos < line_display_width
                    }
                }
            }
        } else {
            false
        };

        // Check if cursor can move to next line
        let can_move_to_next_line = if !can_move_right_in_line {
            // VISUAL BLOCK FIX: In Visual Block mode, prevent moving to next line
            if self.editor_mode == EditorMode::VisualBlock {
                false
            } else {
                let next_display_line = current_display_pos.row + 1;
                self.display_cache
                    .get_display_line(next_display_line)
                    .is_some()
            }
        } else {
            false
        };

        // Perform the actual cursor movement
        if can_move_right_in_line {
            // Move right within current line
            if let Some(current_line) = self.display_cache.get_display_line(current_display_pos.row)
            {
                let new_col = current_line.move_right_by_character(current_display_pos.col);
                self.display_cursor = Position::new(current_display_pos.row, new_col);
                self.update_virtual_column();
                moved = true;
            }
        } else if can_move_to_next_line {
            // Move to beginning of next line
            let next_display_line = current_display_pos.row + 1;
            self.display_cursor = Position::new(next_display_line, 0);
            self.update_virtual_column();
            moved = true;
        }

        if moved {
            // Sync logical cursor with new display position
            let new_display_pos = self.display_cursor;
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.buffer.set_cursor(new_logical_pos);

                // Update visual selection if active
                self.update_visual_selection_on_cursor_move(new_logical_pos);
            }

            let mut events = vec![
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
                ViewEvent::CurrentAreaRedrawRequired,
            ];

            // Ensure cursor is visible and add visibility events
            let visibility_events = self.ensure_cursor_visible_with_events(content_width);
            events.extend(visibility_events);

            events
        } else {
            vec![]
        }
    }

    /// Move cursor up with capability checking and virtual column support
    pub fn move_cursor_up(&mut self, content_width: usize) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        let current_display_pos = self.display_cursor;

        if current_display_pos.row > 0 {
            let new_line = current_display_pos.row - 1;

            // Vim-style virtual column: try to restore the desired column position
            let virtual_col = self.virtual_column;
            let new_col = if let Some(display_line) = self.display_cache.get_display_line(new_line)
            {
                let line_char_count = display_line.char_count();
                let max_col = if self.editor_mode == EditorMode::Insert {
                    line_char_count // Insert mode: can be positioned after last character
                } else {
                    line_char_count.saturating_sub(1) // Normal/Visual: stop at last character
                };
                let clamped_col = virtual_col.min(max_col);
                // Snap to character boundary to handle DBCS characters
                display_line.snap_to_character_boundary(clamped_col)
            } else {
                virtual_col
            };

            let new_display_pos = Position::new(new_line, new_col);
            self.display_cursor = new_display_pos;

            // Sync logical cursor with new display position
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.buffer.set_cursor(new_logical_pos);

                // Update visual selection if active
                self.update_visual_selection_on_cursor_move(new_logical_pos);
            }

            let mut events = vec![
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
                ViewEvent::CurrentAreaRedrawRequired,
            ];

            // Ensure cursor is visible and add visibility events
            let visibility_events = self.ensure_cursor_visible_with_events(content_width);
            events.extend(visibility_events);

            events
        } else {
            vec![]
        }
    }

    /// Move cursor down with capability checking and virtual column support
    pub fn move_cursor_down(&mut self, content_width: usize) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        let current_display_pos = self.display_cursor;
        let next_display_line = current_display_pos.row + 1;

        // Check if the next display line actually exists
        if let Some(display_line) = self.display_cache.get_display_line(next_display_line) {
            // Vim-style virtual column: try to restore the desired column position
            let virtual_col = self.virtual_column;
            let line_char_count = display_line.char_count();
            let max_col = if self.editor_mode == EditorMode::Insert {
                line_char_count // Insert mode: can be positioned after last character
            } else {
                line_char_count.saturating_sub(1) // Normal/Visual: stop at last character
            };
            let clamped_col = virtual_col.min(max_col);
            // Snap to character boundary to handle DBCS characters
            let new_col = display_line.snap_to_character_boundary(clamped_col);
            let new_display_pos = Position::new(next_display_line, new_col);

            self.display_cursor = new_display_pos;

            // Sync logical cursor with new display position
            if let Some(logical_pos) = self
                .display_cache
                .display_to_logical_position(new_display_pos.row, new_display_pos.col)
            {
                let new_logical_pos = LogicalPosition::new(logical_pos.row, logical_pos.col);
                self.buffer.set_cursor(new_logical_pos);

                // Update visual selection if active
                self.update_visual_selection_on_cursor_move(new_logical_pos);
            }

            let mut events = vec![
                ViewEvent::ActiveCursorUpdateRequired,
                ViewEvent::PositionIndicatorUpdateRequired,
                ViewEvent::CurrentAreaRedrawRequired,
            ];

            // Ensure cursor is visible and add visibility events
            let visibility_events = self.ensure_cursor_visible_with_events(content_width);
            events.extend(visibility_events);

            events
        } else {
            vec![]
        }
    }

    /// Set cursor to specific position with capability checking
    pub fn set_current_cursor_position(&mut self, position: LogicalPosition) -> Vec<ViewEvent> {
        // Check if navigation is allowed on this pane
        if !self.capabilities.contains(PaneCapabilities::NAVIGABLE) {
            return vec![]; // Navigation not allowed on this pane
        }

        // Clamp position to valid bounds (same as original implementation)
        let clamped_position = self.buffer.content().clamp_position(position);

        // Update logical cursor
        self.buffer.set_cursor(clamped_position);

        // Sync display cursor with new logical position
        if let Some(display_pos) = self
            .display_cache
            .logical_to_display_position(clamped_position.line, clamped_position.column)
        {
            self.display_cursor = display_pos;
        } else {
            // BUGFIX Issue #89: If logical_to_display_position fails, ensure cursor tracking doesn't break
            tracing::warn!(
                "set_current_cursor_position: logical_to_display_position failed at {:?} - using fallback", 
                clamped_position
            );
            // Fallback: Use logical position as display position (works for non-wrapped content)
            self.display_cursor = Position::new(clamped_position.line, clamped_position.column);
        }

        // BUGFIX: Update virtual column to match new cursor position
        // This ensures that subsequent vertical movements (j/k) preserve the correct column
        self.update_virtual_column();

        // Update visual selection if active
        if self.visual_selection_start.is_some() {
            self.visual_selection_end = Some(clamped_position);
        }

        let mut events = vec![
            ViewEvent::ActiveCursorUpdateRequired,
            ViewEvent::PositionIndicatorUpdateRequired,
        ];

        // Ensure cursor is visible and add visibility events
        let content_width = self.get_content_width();
        let visibility_events = self.ensure_cursor_visible_with_events(content_width);
        events.extend(visibility_events);

        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::events::{Pane, PaneCapabilities};

    #[test]
    fn set_current_cursor_position_should_update_virtual_column() {
        // Create a minimal PaneState for testing
        let mut pane_state = PaneState::new(
            Pane::Request,
            80,
            24,
            false,
            PaneCapabilities::EDITABLE | PaneCapabilities::NAVIGABLE,
        );

        // Insert some test content
        pane_state.buffer.insert_text("line 1\nline 2\nline 3");
        pane_state.build_display_cache(80, false, 4);

        // Set cursor to specific position (line 1, column 4)
        let target_position = LogicalPosition::new(1, 4);
        let _ = pane_state.set_current_cursor_position(target_position);

        // Verify that virtual column matches the cursor position
        assert_eq!(pane_state.virtual_column, 4,
            "Virtual column should be updated to match cursor position after set_current_cursor_position");
    }

    #[test]
    fn set_current_cursor_position_should_preserve_virtual_column_for_vertical_movement() {
        // Create a minimal PaneState for testing
        let mut pane_state = PaneState::new(
            Pane::Request,
            80,
            24,
            false,
            PaneCapabilities::EDITABLE | PaneCapabilities::NAVIGABLE,
        );

        // Insert test content with different line lengths
        pane_state.buffer.insert_text("short\nlonger line\nshort");
        pane_state.build_display_cache(80, false, 4);

        // Set cursor to position on longer line
        let target_position = LogicalPosition::new(1, 7); // "longer |line"
        let _ = pane_state.set_current_cursor_position(target_position);

        // Verify virtual column is set
        assert_eq!(pane_state.virtual_column, 7);

        // Move down to shorter line - should clamp but preserve virtual column intent
        let _ = pane_state.move_cursor_down(80);

        // Virtual column should still be 7 (preserved for future movements)
        assert_eq!(
            pane_state.virtual_column, 7,
            "Virtual column should be preserved during vertical movement"
        );

        // Current cursor should be clamped to shorter line length
        let current_cursor = pane_state.buffer.cursor();
        assert!(
            current_cursor.column <= 5, // "short" = 5 chars
            "Cursor should be clamped to line length but virtual column preserved"
        );
    }
}
